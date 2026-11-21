#!/usr/bin/env python3
"""Rogue Territory: fetch the products and chart images, compare the image reads, build the catalogue.

Rogue Territory's charts are JPEGs, so the numbers are read by eye (Claude, twice, independently),
not by this script. Python 3.9+ standard library only; on macOS it also uses the built-in `sips`
to save an enlarged copy of each image for reading. Run from the repo root:

  1. python3 catalogue/rogue-territory/tools/extract.py fetch
       Downloads products.json and every chart image into the work folder (default
       <tmp>/real-denim-fit/rogue-territory/<date>, or --work DIR). Writes manifest.json and
       images/, plus images/large/ (enlarged copies), and prints each chart to read.
  2. Read each chart twice, independently, into reads/<chart>-a.json and reads/<chart>-b.json:
       {"columns": ["TAG SIZE", "WAIST", ...headings as printed...],
        "rows": [["29", "15.5", ...], ...]}      # every cell as printed; "?" if unsure
  3. python3 catalogue/rogue-territory/tools/extract.py compare
       Compares A and B cell by cell (and C, from reads/<chart>-c.json, for cells that disagree).
       Writes agreed/<chart>.json, prints the tally and the disputed cells (headings only, no values).
  4. python3 catalogue/rogue-territory/tools/extract.py build
       Writes brand.json and one file per cut to catalogue/rogue-territory/ (or --out DIR), from the
       agreed cells only, and lists every cell it left out and why. It never writes EXTRACTING.md.

Common options: --date YYYY-MM-DD (the "seen" date, default today), --work DIR. fetch also takes
--offline (use what's already in the work folder) and --no-enlarge.
"""
import argparse
import datetime
import hashlib
import html
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request

SITE = "https://www.rogueterritory.com"
UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) real-denim-fit catalogue script"
PAUSE = 1.5

BRAND = {
    "id": "rogue-territory",
    "name": "Rogue Territory",
    "site": SITE,
    "currency": "USD",
    "sizing": "waist",
    "tolerance": {"value": 0.25, "unit": "in"},
    "conventions": {"waist": "flat", "seat": "flat", "thigh": "flat", "knee": "flat", "leg_opening": "flat"},
    "measuring_points": {"thigh": "1in-below-crotch", "knee": "13in-below-crotch", "seat": "unknown"},
}
CUT_LABELS = {  # cut id -> (name, brand_label); a new cut takes its label from the product's FIT line
    "stanton": ("Stanton", "Slim Straight, Mid-Rise"),
    "strider": ("Strider", "Straight Leg, High-Rise"),
    "relaxed-strider": ("Relaxed Strider", "Relaxed Straight Leg, High-Rise"),
    "standard-issue": ("Standard Issue", "Relaxed Slight Tapered Leg, Mid-Rise"),
    "silveridge": ("Silveridge", "Slim Straight, High-Rise"),
    "strong-taper": ("Strong Taper", "Hard Taper, Mid-Rise"),
    "sk": ("SK", "Skinny Leg, Mid-Rise"),
}
CUT_ALIASES = {"double knee strider": "strider"}  # the brand: "Canvas Double Knees ... in the Strider fit"
COLUMNS = {"TAG SIZE": "label", "WAIST": "waist", "FRONT RISE": "front_rise", "BACK RISE": "back_rise",
           "UPPER THIGH": "thigh", "KNEE": "knee", "LEG OPENING": "leg_opening", "INSEAM": "inseam"}


def slug(s):
    return re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-")


def text_of(fragment):
    return re.sub(r"\s+", " ", html.unescape(re.sub(r"<[^>]+>", " ", fragment))).strip()


def download(url, path, offline, state):
    if os.path.exists(path) and os.path.getsize(path) > 0:
        return True
    if offline:
        return False
    wait = PAUSE - (time.time() - state.get("last", 0))
    if wait > 0:
        time.sleep(wait)
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    data = None
    for attempt in range(4):  # retry transient server and network errors, backing off
        try:
            with urllib.request.urlopen(req, timeout=60) as r:
                data = r.read()
            break
        except (urllib.error.URLError, TimeoutError) as e:
            if attempt == 3 or (isinstance(e, urllib.error.HTTPError) and e.code < 500 and e.code != 429):
                print(f"couldn't download {url}: {e}", file=sys.stderr)
                break
            time.sleep(5 * (attempt + 1))
    state["last"] = time.time()
    if data is None:
        return False
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as f:
        f.write(data)
    return True


# ---------- fetch ----------

def product_info(p):
    body = p["body_html"] or ""
    text = text_of(body)
    fit = re.search(r"FIT //\s*(.*?)\s*(?:DENIM|FABRIC) //", text)
    fabric = re.search(r"(?:DENIM|FABRIC) //\s*(.*?)\s*(?:–|HOW WE MEASURE|FABRIC NOTE|$)", text)
    tol = re.search(r"Measurements \(in inches\) taken flat, \+/- ([\d.]+)″", text)
    imgs = re.findall(r'<img[^>]+src="([^"]+)"', body)
    return {
        "handle": p["handle"],
        "title": re.sub(r"\s+", " ", p["title"]).strip(),
        "url": f"{SITE}/products/{p['handle']}",
        "fit": fit.group(1).strip() if fit else None,
        "fabric": fabric.group(1).strip() if fabric else None,
        "tolerance": tol.group(1) if tol else None,
        "images": imgs,
        "prices": [(v["price"], v.get("compare_at_price")) for v in p["variants"]],
    }


def cmd_fetch(a):
    work, state = a.work, {}
    path = lambda *x: os.path.join(work, *x)  # noqa: E731
    products, page = [], 1
    while True:
        f = path(f"products-{page}.json")
        if not download(f"{SITE}/products.json?limit=250&page={page}", f, a.offline, state):
            raise SystemExit(f"couldn't get {f}")
        batch = json.load(open(f))["products"]
        if not batch:
            break
        products += batch
        page += 1
    manifest = {"date": a.date, "products": [], "excluded": [], "charts": {}}
    by_hash = {}
    for p in products:
        if "DENIM" not in p["tags"]:
            continue
        info = product_info(p)
        if not info["fabric"] or "denim" not in info["fabric"].lower():
            manifest["excluded"].append({"handle": p["handle"], "why": f"not denim: {info['fabric']!r}"})
            continue
        if len(info["images"]) != 1:
            manifest["excluded"].append({"handle": p["handle"], "why": f"{len(info['images'])} images on the page; expected one chart"})
            continue
        url = info["images"][0]
        name = re.sub(r"\?.*$", "", url.rsplit("/", 1)[1])
        local = path("images", name)
        if not download(url, local, a.offline, state):
            manifest.setdefault("problems", []).append(f"{p['handle']}: couldn't download its chart image {url}")
            continue
        # A "_480x480"-style rendition: also get the original upload, which reads more easily.
        best = local
        m = re.match(r"(.+)_\d+x\d+(\.\w+)$", name)
        if m:
            orig = path("images", m.group(1) + m.group(2))
            if download(re.sub(r"_\d+x\d+(\.\w+)", r"\1", url), orig, a.offline, state):
                best = orig
        digest = hashlib.sha256(open(local, "rb").read()).hexdigest()
        chart = by_hash.setdefault(digest, os.path.splitext(name)[0])  # byte-identical uploads share a chart
        c = manifest["charts"].setdefault(chart, {"image_urls": [], "file": os.path.relpath(best, work), "products": []})
        if url not in c["image_urls"]:
            c["image_urls"].append(url)
        c["products"].append(p["handle"])
        info["chart"] = chart
        manifest["products"].append(info)
    if not a.no_enlarge and shutil.which("sips"):
        os.makedirs(path("images", "large"), exist_ok=True)
        for chart, c in manifest["charts"].items():
            out = path("images", "large", chart + ".jpg")
            if not os.path.exists(out):
                subprocess.run(["sips", "--resampleWidth", "1400", path(c["file"]), "--out", out],
                               check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            c["large"] = os.path.relpath(out, work)
    os.makedirs(path("reads"), exist_ok=True)
    with open(path("manifest.json"), "w") as f:
        json.dump(manifest, f, indent=1, ensure_ascii=False)
    print(f"{len(manifest['products'])} denim products, {len(manifest['charts'])} distinct chart images, work folder {work}")
    for e in manifest["excluded"]:
        print(f"excluded {e['handle']}: {e['why']}")
    for line in manifest.get("problems", []):
        print(f"PROBLEM {line} (run fetch again; finished downloads are kept)")
    print("\nRead each chart twice, independently (see the docstring for the JSON shape):")
    for chart, c in manifest["charts"].items():
        print(f"  {c.get('large', c['file'])}  ->  reads/{chart}-a.json, reads/{chart}-b.json   ({', '.join(c['products'])})")
    if manifest.get("problems"):
        sys.exit(1)


# ---------- compare ----------

def norm(s):
    return re.sub(r"\s+", " ", str(s)).strip()


def load_read(path):
    d = json.load(open(path))
    cols = [norm(c).upper() for c in d["columns"]]
    cells = {}
    for row in d["rows"]:
        label = norm(row[0])
        for heading, cell in zip(cols[1:], row[1:]):
            cells[(label, heading)] = norm(cell)
    return cols, [norm(r[0]) for r in d["rows"]], cells


def cmd_compare(a):
    manifest = json.load(open(os.path.join(a.work, "manifest.json")))
    os.makedirs(os.path.join(a.work, "agreed"), exist_ok=True)
    tally = {"cells": 0, "agreed": 0, "settled_by_c": 0, "left_out": 0}
    missing, disputed = [], []
    for chart in manifest["charts"]:
        ra, rb, rc = (os.path.join(a.work, "reads", f"{chart}-{x}.json") for x in "abc")
        if not (os.path.exists(ra) and os.path.exists(rb)):
            missing.append(chart)
            continue
        cols_a, rows_a, A = load_read(ra)
        _, rows_b, B = load_read(rb)
        C = load_read(rc)[2] if os.path.exists(rc) else {}
        rows = rows_a + [r for r in rows_b if r not in rows_a]
        kept, left_out = {}, []
        for key in sorted(set(A) | set(B), key=lambda k: (rows.index(k[0]), cols_a.index(k[1]) if k[1] in cols_a else 99)):
            tally["cells"] += 1
            va, vb, vc = A.get(key), B.get(key), C.get(key)
            if va is not None and va == vb and va != "?":
                kept[key] = va
                tally["agreed"] += 1
            elif vc not in (None, "?") and vc in (va, vb):
                kept[key] = vc
                tally["settled_by_c"] += 1
            else:
                left_out.append(list(key))
                tally["left_out"] += 1
                if vc is None:
                    disputed.append(f"{chart}: row {key[0]}, column {key[1]}")
        out = {"columns": cols_a, "rows": {}, "left_out": left_out}
        for (label, heading), v in kept.items():
            out["rows"].setdefault(label, {})[heading] = v
        with open(os.path.join(a.work, "agreed", chart + ".json"), "w") as f:
            json.dump(out, f, indent=1, ensure_ascii=False)
    print(f"cells read: {tally['cells']}, agreed at once: {tally['agreed']}, settled by read C: {tally['settled_by_c']}, left out: {tally['left_out']}")
    if disputed:
        print("\nCells for read C (give the reader the image and these positions only, never the values):")
        for d in disputed:
            print("  " + d)
    if missing:
        print("\nMissing read A or B for: " + ", ".join(missing))
        sys.exit(1)


# ---------- build ----------

def cell_number(text):
    t = norm(text)
    m = re.fullmatch(r"(\d+)(?:\s+(\d+)/(\d+))?", t) or re.fullmatch(r"(\d+\.\d+)", t)
    if not m:
        return None
    if "." in t:
        v = float(t)
    else:
        v = float(m.group(1)) + (int(m.group(2)) / int(m.group(3)) if m.group(2) else 0)
    return int(v) if v.is_integer() else v


def fabric_facts(fabric):
    f = fabric.lower()
    m = re.search(r"(\d+(?:\.\d+)?)\s*oz", f)
    weight = float(m.group(1)) if m else None
    if weight is not None and weight.is_integer():
        weight = int(weight)
    if "unsanforized" in f:
        sanforized, how = False, "page"
    elif "sanforized" in f:
        sanforized, how = True, "page"
    else:
        sanforized, how = True, "faq"  # FAQ: "Our denim is sanforized (pre-shrunk) unless otherwise noted"
    finish = "washed" if re.search(r"\bwashed\b", f) else "rinsed" if re.search(r"\brinsed\b", f) else "raw"
    return weight, "100% cotton" in f, sanforized, how, finish


def list_price(prices):
    amounts = []
    for price, cmp in prices:
        price = float(price)
        cmp = float(cmp) if cmp else 0.0
        amounts.append(cmp if cmp > price else price)
    common = max(set(amounts), key=amounts.count)
    return (int(common) if common.is_integer() else common), len(set(amounts)) > 1


def dumps(v, indent=0, width=230):
    pad = "  " * indent
    one = json.dumps(v, ensure_ascii=False, separators=(", ", ": "))
    if not isinstance(v, (dict, list)) or len(one) + len(pad) <= width:
        return one
    inner = "  " * (indent + 1)
    if isinstance(v, dict):
        items = [f"{inner}{json.dumps(k, ensure_ascii=False)}: {dumps(x, indent + 1)}" for k, x in v.items()]
        return "{\n" + ",\n".join(items) + "\n" + pad + "}"
    return "[\n" + ",\n".join(f"{inner}{dumps(x, indent + 1)}" for x in v) + "\n" + pad + "]"


def write_json(path, v):
    with open(path, "w", encoding="utf-8") as f:
        f.write(dumps(v) + "\n")


def cmd_build(a):
    manifest = json.load(open(os.path.join(a.work, "manifest.json")))
    seen = a.date
    cuts, left_out, problems, faq = {}, [], [], []
    for info in manifest["products"]:
        agreed_path = os.path.join(a.work, "agreed", info["chart"] + ".json")
        if not os.path.exists(agreed_path):
            problems.append(f"{info['handle']}: no agreed reading for chart {info['chart']}: run compare first")
            continue
        agreed = json.load(open(agreed_path))
        unknown = [c for c in agreed["columns"] if c not in COLUMNS]
        if unknown:
            problems.append(f"{info['chart']}: unknown column headings {unknown}: add them to COLUMNS")
            continue
        left, right = [x.strip() for x in info["title"].split("//", 1)]
        cut = CUT_ALIASES.get(left.lower(), slug(left))
        pid = slug(f"{left} {right}")
        tag_heading = next(c for c in agreed["columns"] if COLUMNS[c] == "label")
        for key in agreed["left_out"]:
            left_out.append(f"{pid} row {key[0]} {key[1]}: image reads disagreed or were unsure")
        sizes = []
        for label, cells in agreed["rows"].items():
            tag = cell_number(cells.get(tag_heading, label))
            if not isinstance(tag, int):
                left_out.append(f"{pid} row {label}: tag size unreadable; whole row left out")
                continue
            meas = {}
            for heading, text in cells.items():
                key = COLUMNS[heading]
                if key == "label":
                    continue
                v = cell_number(text)
                if v is None:
                    left_out.append(f"{pid} size {tag} {key}: printed {text!r}, not a number")
                elif key == "waist" and abs(2 * v - tag) > 4:
                    left_out.append(f"{pid} size {tag} waist: printed {text!r}, implausible for a flat waist on tag {tag}")
                else:
                    meas[key] = v
            sizes.append({"label": str(tag), "waist_tag": tag, "measurements": meas})
        weight, cotton, sanforized, how, finish = fabric_facts(info["fabric"])
        if how == "faq":
            faq.append(pid)
        if info["tolerance"] not in (None, "0.25"):
            problems.append(f"{pid}: page states a tolerance of ±{info['tolerance']}″, not ±0.25″")
        amount, varies = list_price(info["prices"])
        if varies:
            problems.append(f"{pid}: list price differs between sizes; used the most common one")
        prod = {
            "id": pid,
            "name": right if left.lower() not in CUT_ALIASES else f"{left.replace(CUT_LABELS[cut][0], '').strip()}, {right}",
            "url": info["url"],
            "status": "current",
            "status_seen": seen,
            "fabric": {"weight_oz": weight, "composition": [{"fibre": "cotton", "percent": 100}] if cotton else None,
                       "sanforized": sanforized},
            "sold": "raw" if finish == "raw" else "one-washed",
            "finish": finish,
            "list_price": {"amount": amount, "currency": "USD", "url": info["url"], "seen": seen},
            "shrinkage": None,  # no product figures; the FAQ's "add 0.5in to your inseam" is hemming advice
            "charts": [{"state": "as-sold", "source": {"url": info["url"], "seen": seen}, "unit": "in",
                        "from_image": True, "sizes": sizes}],
        }
        if cut not in CUT_LABELS:
            CUT_LABELS[cut] = (left, info["fit"])
            problems.append(f"{pid}: new cut {left!r}: check its brand_label {info['fit']!r} and add it to CUT_LABELS")
        cuts.setdefault(cut, []).append(prod)
    os.makedirs(a.out, exist_ok=True)
    for key, prods in cuts.items():
        name, label = CUT_LABELS[key]
        write_json(os.path.join(a.out, f"{key}.json"), {
            "id": key, "brand": "rogue-territory", "name": name, "brand_label": label,
            "line": None,  # the site doesn't file its jeans as men's, women's or unisex
            "products": sorted(prods, key=lambda x: x["id"]),
        })
    write_json(os.path.join(a.out, "brand.json"), dict(BRAND, method_source={"url": f"{SITE}/pages/sizing-guide", "seen": seen}))
    err = sys.stderr
    print(f"{sum(len(v) for v in cuts.values())} products, {sum(len(p['charts'][0]['sizes']) for v in cuts.values() for p in v)} sizes, "
          "cuts: " + ", ".join(f"{k} ({len(v)})" for k, v in cuts.items()), file=err)
    print(f"sanforized from the FAQ: {', '.join(faq) or 'none'}", file=err)
    for line in left_out:
        print("left out: " + line, file=err)
    for line in problems:
        print("PROBLEM " + line, file=err)
    if problems:
        sys.exit(1)


def main():
    root = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("step", choices=["fetch", "compare", "build"])
    ap.add_argument("--date", default=datetime.date.today().isoformat())
    ap.add_argument("--work")
    ap.add_argument("--out", default=os.path.join(root, "catalogue", "rogue-territory"))
    ap.add_argument("--offline", action="store_true")
    ap.add_argument("--no-enlarge", action="store_true")
    a = ap.parse_args()
    a.work = a.work or os.path.join(tempfile.gettempdir(), "real-denim-fit", "rogue-territory", a.date)
    os.makedirs(a.work, exist_ok=True)
    {"fetch": cmd_fetch, "compare": cmd_compare, "build": cmd_build}[a.step](a)


if __name__ == "__main__":
    main()
