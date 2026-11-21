"""Helpers shared by the brand extractors used by /update-catalogue.

Standard library only. Fetches politely (one request a second, an honest
user agent), caches pages for the run, and writes a brand file that keeps
products that have gone from the brand's site, marked so.
"""

import datetime
import html
import json
import os
import re
import sys
import time
import subprocess

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
BRANDS = os.path.join(ROOT, "catalogue", "brands")
CACHE = os.environ.get("RDF_CACHE", os.path.join(ROOT, "target", "catalogue-cache"))
TODAY = os.environ.get("RDF_SEEN", datetime.date.today().isoformat())
AGENT = "RealDenimFit-catalogue/1.0 (+https://github.com/DigitalN/Real-Denim-Fit)"

_last = [0.0]
# Seconds between requests.
GAP = float(os.environ.get("RDF_GAP", "2"))


def fetch(url, cache=True):
    """Returns the body of url as text, caching it under target/."""
    os.makedirs(CACHE, exist_ok=True)
    key = os.path.join(CACHE, re.sub(r"[^A-Za-z0-9]+", "_", url)[-180:])
    if cache and os.path.exists(key):
        with open(key, encoding="utf-8") as f:
            return f.read()
    for attempt in range(6):
        wait = GAP - (time.time() - _last[0])
        if wait > 0:
            time.sleep(wait)
        # curl rather than urllib: some brands' bot protection turns away
        # Python's TLS client but not curl's.
        r = subprocess.run(
            ["curl", "-sSL", "--max-time", "40", "-A", AGENT, "-w", "\n%{http_code}", url],
            capture_output=True,
        )
        _last[0] = time.time()
        out = r.stdout.decode("utf-8", "replace")
        body, _, code = out.rpartition("\n")
        if r.returncode == 0 and code == "200":
            break
        if code != "429" or attempt == 5:
            raise RuntimeError(f"{url}: HTTP {code} {r.stderr.decode()[:200]}")
        time.sleep(15 * (attempt + 1))
    with open(key, "w", encoding="utf-8") as f:
        f.write(body)
    return body


def text_of(fragment):
    """Visible text of an HTML fragment, one line per block."""
    t = re.sub(r"<script.*?</script>|<style.*?</style>", "", fragment, flags=re.S | re.I)
    t = re.sub(r"<br\s*/?>|</(p|div|li|tr|h\d)>", "\n", t, flags=re.I)
    t = html.unescape(re.sub(r"<[^>]+>", " ", t))
    lines = [re.sub(r"[ \t ]+", " ", ln).strip() for ln in t.split("\n")]
    return "\n".join(ln for ln in lines if ln)


def num(s):
    """A number from a table cell, or None for blanks and dashes."""
    if s is None:
        return None
    s = str(s).strip().replace(",", ".")
    m = re.fullmatch(r"(\d+)\s+(\d)/(\d)", s)  # "12 1/2"
    if m:
        return int(m[1]) + int(m[2]) / int(m[3])
    fracs = {"½": 0.5, "¼": 0.25, "¾": 0.75, "⅛": 0.125, "⅜": 0.375, "⅝": 0.625, "⅞": 0.875}
    for k, v in fracs.items():
        if s.endswith(k):
            head = s[: -len(k)].strip()
            return (float(head) if head else 0) + v
    try:
        return float(re.sub(r"[^0-9.]", "", s)) if re.search(r"\d", s) else None
    except ValueError:
        return None


def slug(s):
    return re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-")


def write_brand(brand):
    """Writes catalogue/brands/<id>.json, keeping products that have gone
    from the brand's site and marking them so (they're never deleted)."""
    path = os.path.join(BRANDS, brand["id"] + ".json")
    fresh_ids = {p["id"] for p in brand["products"]}
    gone = []
    if os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            old = json.load(f)
        for p in old.get("products", []):
            if p["id"] not in fresh_ids:
                p["on_brands_site"] = False
                brand["products"].append(p)
                gone.append(p["id"])
    brand["products"].sort(key=lambda p: (p["cut"], p["id"]))
    with open(path, "w", encoding="utf-8") as f:
        json.dump(brand, f, indent=1, ensure_ascii=False)
        f.write("\n")
    print(f"wrote {path}: {len(brand['products'])} products", file=sys.stderr)
    if gone:
        print("no longer on the brand's site: " + ", ".join(gone), file=sys.stderr)
    return path


def compact_rows(path):
    """Rewrites a brand file so each size row sits on one line, which keeps
    catalogue PRs readable."""
    with open(path, encoding="utf-8") as f:
        text = f.read()

    def squash(m):
        return re.sub(r"\s*\n\s*", " ", m.group(0))

    text = re.sub(r"\{\s*\n\s*\"size\":[^{}]*\}", squash, text)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)
