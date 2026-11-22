#!/usr/bin/env python3
"""Extract the Iron Heart catalogue from ironheart.co.uk.

Usage (from the repo root; Python 3.9+ standard library only):

    python3 catalogue/iron-heart/tools/extract.py                  # fetch, then write catalogue/iron-heart/
    python3 catalogue/iron-heart/tools/extract.py --offline        # rebuild from the cache, no network
    python3 catalogue/iron-heart/tools/extract.py --only ih-634s   # print one product's JSON, write nothing

Options: --date YYYY-MM-DD (the "seen" date, default today), --cache DIR (downloads,
default <tmp>/real-denim-fit/iron-heart/<date>), --out DIR (default catalogue/iron-heart).

It writes brand.json and one file per cut (555.json, 634.json, ...), but never EXTRACTING.md.
Then it prints what it skipped and why, plus any inch/cm disagreements in the brand's tables.
Read catalogue/iron-heart/EXTRACTING.md for what each step relies on.
"""
import argparse
import datetime
import html
import json
import os
import re
import sys
import tempfile
import time
import urllib.error
import urllib.request

SITE = "https://www.ironheart.co.uk"
UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) real-denim-fit catalogue script"
PAUSE = 2.0  # seconds between requests to the brand's site

# Products whose inch table has a typo the cm table doesn't share: record the cm table instead.
USE_CM = {
    "ih-666s-uhr": "inch table gives size 35 a 37.8 waist, above size 36's 37.1; the cm table's 92.5 fits",
}

# Fabric Focus heading -> (sanforized, sold, finish). No Iron Heart jean is faded at the factory,
# so the one-wash (overdyed or not) is "rinsed".
FABRIC_FOCUS = {
    "Sanforized & One-Washed": (True, "one-washed", "rinsed"),
    "Sanforized, One-Washed & Overdyed": (True, "one-washed", "rinsed"),
    "Unsanforized but One-Washed": (False, "one-washed", "rinsed"),
    "Sanforized but Unwashed": (True, "raw", "raw"),
    "Unsanforized & Unwashed": (False, "raw", "raw"),
}
# Shrinkage the brand states in the Fabric Focus block, in inches.
SHRINKAGE = {
    "Unsanforized & Unwashed": (1, 2, 'Approximately 1" shrinkage in the waist and up to 2" shrinkage on the inseam'),
    "Sanforized but Unwashed": (1, 1, 'Expect approximately 1" shrinkage in the waist and inseam'),
}
ROWS = {"Waist": "waist", "Front Rise": "front_rise", "Rear Rise": "back_rise", "Thigh": "thigh",
        "Knee": "knee", "Hem": "leg_opening", "Inseam": "inseam"}
BRAND = {
    "id": "iron-heart",
    "name": "Iron Heart",
    "site": SITE,
    "currency": "GBP",
    "sizing": "waist",
    "tolerance": None,
    "conventions": {"waist": "around", "seat": "flat", "thigh": "flat", "knee": "flat", "leg_opening": "flat"},
    "measuring_points": {"thigh": "at-crotch", "knee": "13in-below-crotch", "seat": "unknown"},
}


# ---------- fetching ----------

class Fetcher:
    def __init__(self, cache, offline):
        self.cache, self.offline, self.last = cache, offline, 0.0

    def get(self, url, name):
        path = os.path.join(self.cache, name)
        if os.path.exists(path) and os.path.getsize(path) > 0:
            with open(path, "rb") as f:
                return f.read().decode("utf-8", "replace")
        if self.offline:
            raise SystemExit(f"--offline: {path} isn't in the cache")
        wait = PAUSE - (time.time() - self.last)
        if wait > 0:
            time.sleep(wait)
        req = urllib.request.Request(url, headers={"User-Agent": UA})
        for attempt in range(4):  # retry transient server and network errors, backing off
            try:
                with urllib.request.urlopen(req, timeout=60) as r:
                    data = r.read()
                break
            except (urllib.error.URLError, TimeoutError) as e:
                if attempt == 3 or (isinstance(e, urllib.error.HTTPError) and e.code < 500 and e.code != 429):
                    raise SystemExit(f"couldn't download {url}: {e}")
                time.sleep(5 * (attempt + 1))
        self.last = time.time()
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb") as f:
            f.write(data)
        return data.decode("utf-8", "replace")


def all_products(fetch):
    out, page = [], 1
    while True:
        text = fetch.get(f"{SITE}/products.json?limit=250&page={page}", f"products-{page}.json")
        batch = json.loads(text)["products"]
        if not batch:
            return out
        out += batch
        page += 1


# ---------- parsing ----------

def is_jean_candidate(p):
    if p["product_type"] not in ("Bottoms", "Future Releases", "", "OUTLET"):
        return False
    skus = {(v.get("sku") or "").split("/")[0] for v in p["variants"]}
    if re.search(r"jean|cut -|cut jean", p["title"], re.I):
        return True
    return any(re.match(r"IH-?\d{3}", s) and not re.match(r"IH-?(7[1-4]\d|8[0-3]\d|72\d)", s) for s in skus)


def text_of(fragment):
    return re.sub(r"\s+", " ", html.unescape(re.sub(r"<[^>]+>", " ", fragment))).strip()


def chart_table(body, marker):
    """The table after [SIZE-GUIDE-INCH] or [SIZE-GUIDE-CM] in body_html, as (sizes, {row: cells})."""
    i = body.find(marker)
    if i < 0:
        return None
    nxt = body.find("[SIZE-GUIDE-", i + len(marker))
    part = body[i:nxt if nxt > 0 else len(body)]
    tables = re.findall(r"<table.*?</table>", part, re.S)
    if not tables:
        return None
    rows = []
    for tr in re.findall(r"<tr.*?</tr>", tables[0], re.S):
        rows.append([text_of(c) for c in re.findall(r"<t[hd][^>]*>(.*?)</t[hd]>", tr, re.S)])
    return rows[0][1:], {r[0]: r[1:] for r in rows[1:]}


def page_lines(page_html):
    t = re.sub(r"<script.*?</script>", "", page_html, flags=re.S)
    t = re.sub(r"<style.*?</style>", "", t, flags=re.S)
    t = html.unescape(re.sub(r"<[^>]+>", "\n", t))
    return [l.strip() for l in t.split("\n") if l.strip()]


STOPS = {"Model Sizing", "Fabric Focus", "Stretch Details", "Care Guidelines", "View in Hall of Fades"}


def section(lines, start):
    out = []
    for l in lines[start + 1:]:
        if l in STOPS:
            break
        out.append(l)
    return out


def read_page(page_html):
    """Description lines and the Fabric Focus block from the rendered product page."""
    L = page_lines(page_html)
    heads = [k for k, l in enumerate(L) if l.lower() in ("product description", "product details")]
    desc = section(L, heads[0]) if heads else []
    ff = section(L, L.index("Fabric Focus")) if "Fabric Focus" in L else None
    return desc, ff


def fabric_info(desc):
    cut_line = next((l for l in desc if re.fullmatch(r"\d{3,4} [A-Za-z /-]+Cut", l)), None)
    j = next((i for i, l in enumerate(desc) if not l.startswith("-") and re.match(r"\d+(?:\.\d+)?(?:/\d+)?oz", l)), None)
    if cut_line is None or j is None:
        return cut_line, None, None, None
    heading = desc[j]
    m = re.match(r"(\d+(?:\.\d+)?)(?:/\d+(?:\.\d+)?)?oz", heading)  # "21/23oz" UHR -> 21, its raw weight
    weight = num(m.group(1))
    comp = None
    for l in desc[j + 1:]:
        if not l.startswith("-"):
            break
        t = l.lstrip("- ").strip()
        if re.fullmatch(r"100% (organic )?cotton", t, re.I):
            comp = [{"fibre": "cotton", "percent": 100}]
            break
        m2 = re.fullmatch(r"(\d+)% cotton / (\d+)% (\w+)", t, re.I)
        if m2:
            comp = [{"fibre": "cotton", "percent": int(m2.group(1))},
                    {"fibre": m2.group(3).lower(), "percent": int(m2.group(2))}]
            break
    return cut_line, heading, weight, comp


def product_name(title):
    t = re.sub(r"\s+", " ", title).strip()
    t = re.sub(r' ?(?:"1955" )?(?:Medium/High Rise |Mid/High Rise |Relaxed |Slim |Vintage )?'
               r'(?:Straight |Tapered |Slim )?(?:Straight |Tapered )?Cut(?: Jeans)?(?= -)', "", t)
    t = re.sub(r' "1955" Vintage Tapered(?: Cut)? Jeans(?= -)', "", t)
    return t.replace(" Jeans -", " -")


def num(s):
    v = float(s)
    return int(v) if v.is_integer() and "." not in str(s) else v


def list_price(p):
    amounts = []
    for v in p["variants"]:
        price = float(v["price"])
        cmp = float(v["compare_at_price"]) if v.get("compare_at_price") else 0.0
        amounts.append(cmp if cmp > price else price)  # compare_at is the list price on sale items
    common = max(set(amounts), key=amounts.count)
    return (int(common) if common.is_integer() else common), len(set(amounts)) > 1


# ---------- writing ----------

def dumps(v, indent=0, width=230):
    """Indented JSON, but any object or array that fits on one line is kept on one line."""
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


# ---------- main ----------

def build_product(p, page_html, seen, notes):
    h = p["handle"]
    url = f"{SITE}/products/{h}"
    body = p["body_html"] or ""
    inch = chart_table(body, "[SIZE-GUIDE-INCH]")
    cm = chart_table(body, "[SIZE-GUIDE-CM]")
    if inch is None:
        notes["skipped"].append((h, p["title"], "no size chart" + (" (tagged future-release)" if "future-release" in p["tags"] else "")))
        return None, None
    desc, ff = read_page(page_html)
    cut_line, heading, weight, comp = fabric_info(desc)
    if cut_line is None or heading is None:
        notes["problems"].append((h, "couldn't find the cut heading or the fabric heading on the page"))
        return None, None
    if re.search(r"\bDuck\b", heading):
        notes["skipped"].append((h, p["title"], f"not denim ({heading})"))
        return None, None
    if not ff or ff[0] not in FABRIC_FOCUS:
        notes["problems"].append((h, f"unknown Fabric Focus heading {ff[0] if ff else None!r}: add it to FABRIC_FOCUS"))
        return None, None
    sanforized, sold, finish = FABRIC_FOCUS[ff[0]]
    shrinkage = None
    if ff[0] in SHRINKAGE:
        w, i, note = SHRINKAGE[ff[0]]
        shrinkage = {"kind": "amount", "unit": "in", "waist": w, "inseam": i, "note": note,
                     "source": {"url": url, "seen": seen}}
    # Cross-check every inch cell against the cm table (1 in = 2.54 cm).
    if cm and cm[0] == inch[0]:
        for row in inch[1]:
            for size, a, c in zip(inch[0], inch[1][row], cm[1].get(row, [])):
                try:
                    if abs(float(a) - float(c) / 2.54) > 0.2:
                        notes["inch_cm"].append(f"{h} size {size} {row}: inch {a}, cm {c} (= {float(c) / 2.54:.2f} in)")
                except ValueError:
                    pass
    use_cm = h in USE_CM
    sizes_row, rows = cm if use_cm else inch
    sizes = []
    for i, label in enumerate(sizes_row):
        meas = {}
        for row, key in ROWS.items():
            cell = rows.get(row, [""] * len(sizes_row))[i] if i < len(rows.get(row, [])) else ""
            if cell == "":
                notes["blank"].append(f"{h} size {label} {row}")
                continue
            meas[key] = float(cell)  # Iron Heart prints one decimal everywhere ("35.0")
        sizes.append({"label": label, "waist_tag": int(label), "measurements": meas})
    amount, varies = list_price(p)
    if varies:
        notes["problems"].append((h, "list price differs between sizes; used the most common one"))
    product = {
        "id": h[3:] if h.startswith("ih-") else h,
        "name": product_name(p["title"]),
        "url": url,
        "status": "current",
        "status_seen": seen,
        "fabric": {"weight_oz": weight, "composition": comp, "sanforized": sanforized},
        "sold": sold,
        "finish": finish,
        "list_price": {"amount": amount, "currency": "GBP", "url": url, "seen": seen},
        "shrinkage": shrinkage,
        "charts": [{
            "state": "as-sold",
            "source": {"url": url, "seen": seen},
            "unit": "cm" if use_cm else "in",
            "from_image": False,
            "sizes": sizes,
        }],
    }
    cut_key, label = cut_line.split(" ", 1)
    return (cut_key, label), product


def main():
    root = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--date", default=datetime.date.today().isoformat())
    ap.add_argument("--cache")
    ap.add_argument("--out", default=os.path.join(root, "catalogue", "iron-heart"))
    ap.add_argument("--offline", action="store_true")
    ap.add_argument("--only", help="a product handle: print its JSON and write nothing")
    a = ap.parse_args()
    cache = a.cache or os.path.join(tempfile.gettempdir(), "real-denim-fit", "iron-heart", a.date)
    fetch = Fetcher(cache, a.offline)

    products = [p for p in all_products(fetch) if is_jean_candidate(p)]
    if a.only:
        products = [p for p in products if p["handle"] == a.only]
        if not products:
            raise SystemExit(f"{a.only} isn't an Iron Heart jean in products.json")
    notes = {"skipped": [], "problems": [], "inch_cm": [], "blank": []}
    cuts = {}
    for p in products:
        page = fetch.get(f"{SITE}/products/{p['handle']}", os.path.join("pages", p["handle"] + ".html"))
        cut, product = build_product(p, page, a.date, notes)
        if product:
            cuts.setdefault(cut, []).append(product)

    if a.only:
        for prods in cuts.values():
            for prod in prods:
                print(dumps(prod))
    else:
        os.makedirs(a.out, exist_ok=True)
        for (key, label), prods in sorted(cuts.items(), key=lambda kv: int(kv[0][0])):
            write_json(os.path.join(a.out, f"{key}.json"), {
                "id": key, "brand": "iron-heart", "name": key, "brand_label": label,
                "line": None,  # the site doesn't file its jeans as men's, women's or unisex
                "products": sorted(prods, key=lambda x: x["id"]),
            })
        write_json(os.path.join(a.out, "brand.json"),
                   dict(BRAND, method_source={"url": f"{SITE}/pages/how-we-measure-bottoms", "seen": a.date}))

    total = sum(len(v) for v in cuts.values())
    sizes = sum(len(x["charts"][0]["sizes"]) for v in cuts.values() for x in v)
    print(f"{total} products, {sizes} sizes, cuts: " + ", ".join(f"{k[0]} ({len(v)})" for k, v in sorted(cuts.items(), key=lambda kv: int(kv[0][0]))), file=sys.stderr)
    print(f"cache: {cache}", file=sys.stderr)
    for h, title, why in notes["skipped"]:
        print(f"skipped {h}: {why}", file=sys.stderr)
    for h, why in notes["problems"]:
        print(f"PROBLEM {h}: {why}", file=sys.stderr)
    for line in notes["blank"]:
        print(f"blank cell: {line}", file=sys.stderr)
    for line in notes["inch_cm"]:
        tag = " (recorded from cm)" if line.split(" ")[0] in USE_CM else ""
        print(f"inch/cm disagree: {line}{tag}", file=sys.stderr)
    if notes["problems"]:
        sys.exit(1)


if __name__ == "__main__":
    main()
