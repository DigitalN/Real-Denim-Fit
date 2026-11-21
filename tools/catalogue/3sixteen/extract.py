#!/usr/bin/env python3
"""Extract the 3sixteen catalogue from 3sixteen.com.

Usage (from the repo root; Python 3.9+ standard library only):

    python3 tools/catalogue/3sixteen/extract.py                 # fetch, then write catalogue/3sixteen/
    python3 tools/catalogue/3sixteen/extract.py --offline       # rebuild from the cache, no network
    python3 tools/catalogue/3sixteen/extract.py --only cs-100x-classic-straight-indigo-selvedge

Options: --date YYYY-MM-DD (the "seen" date, default today), --cache DIR (downloads,
default <tmp>/real-denim-fit/3sixteen/<date>), --out DIR (default catalogue/3sixteen),
--only HANDLE (print one product's JSON and write nothing).

The chart is the page's "Measurements" modal. The script fetches it straight from Shopify's
section endpoint (/products/<handle>?sections=sizing-modal), so no browser is needed. It writes
brand.json and one file per cut, but never EXTRACTING.md. Then it prints the fabric facts it
inferred, so you can check them, and anything odd in the charts.
"""
import argparse
import datetime
import html
import json
import os
import re
import statistics
import sys
import tempfile
import time
import urllib.error
import urllib.request

SITE = "https://www.3sixteen.com"
UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) real-denim-fit catalogue script"
PAUSE = 2.0

CONVENTIONS = {"waist": "around", "seat": "flat", "thigh": "flat", "knee": "flat", "leg_opening": "flat"}
BRAND = {
    "id": "3sixteen",
    "name": "3sixteen",
    "site": SITE,
    "currency": "USD",
    "sizing": "waist",
    "tolerance": None,
    "conventions": CONVENTIONS,
    "measuring_points": {"thigh": "1in-below-crotch", "knee": "14in-below-crotch", "seat": "unknown"},
}
CUTS = {  # cut id -> (name, brand_label)
    "cs": ("CS", "Classic Straight"),
    "ct": ("CT", "Classic Tapered"),
    "rs": ("RS", "Relaxed Straight"),
    "nt": ("NT", "Narrow Tapered"),
    "st": ("ST", "Slim Tapered"),
    "big-jean": ("Big Jean", None),
    "sexsx3s27-02": ("Self Edge x Samurai x 3sixteen SEXSX3s27-02", "Straight Tapered"),
}
# Collaborations that don't follow the "CS-100x ~ ..." title pattern: handle prefix -> (cut, id, name).
COLLABS = {
    "throwing-fits-big-jean": ("big-jean", "big-jean-103xs", 'Throwing Fits "Big Jean", Stonewashed 103xs'),
    "self-edge-x-samurai": ("sexsx3s27-02", "sexsx3s27-02", "Self Edge x Samurai x 3sixteen, 15oz Unsanforized Selvedge"),
}
ROWS = {"Waist": "waist", "Front Rise": "front_rise", "Back Rise": "back_rise", "Inseam": "inseam",
        "Thigh": "thigh", "Knee": "knee", "Opening": "leg_opening"}
FRAC = {"½": 0.5, "¼": 0.25, "¾": 0.75, "⅛": 0.125, "⅜": 0.375, "⅝": 0.625, "⅞": 0.875}
# Hand decisions the rules can't make, by product id. Keep each with its reason.
OVERRIDES = {}


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
        batch = json.loads(fetch.get(f"{SITE}/products.json?limit=250&page={page}", f"products-{page}.json"))["products"]
        if not batch:
            return out
        out += batch
        page += 1


def text_of(fragment):
    return re.sub(r"\s+", " ", html.unescape(re.sub(r"<[^>]+>", " ", fragment))).strip()


def identify(p):
    """(cut, product id, name) for a jeans product, or None if it isn't one."""
    title = re.sub(r"\s+", " ", p["title"]).strip()
    if "short" in title.lower():
        return None
    for prefix, ident in COLLABS.items():
        if p["handle"].startswith(prefix):
            return ident
    m = re.match(r"([A-Z]{2})-(\w+) ~ [^~-]+?\s*[~-]\s*(.+)$", title)
    if m:
        cut, code, rest = m.group(1).lower(), m.group(2), m.group(3).strip()
        return cut, f"{cut}-{code.lower()}", f"{code} {rest}"
    if "jean" in title.lower():
        return "?", p["handle"], title  # a new kind of jean: needs a cut decision
    return None


# ---------- the chart ----------

def cell_value(cell):
    c = re.sub(r"\s+", "", cell)
    if c == "":
        return None
    m = re.fullmatch(r"(\d+)?([½¼¾⅛⅜⅝⅞])?", c)
    if m and (m.group(1) or m.group(2)):
        v = float(m.group(1) or 0) + (FRAC[m.group(2)] if m.group(2) else 0)
    elif re.fullmatch(r"\d+(\.\d+)?", c):
        v = float(c)
    else:
        raise ValueError(cell)
    return int(v) if v.is_integer() else v


def parse_modal(section_html):
    """Returns (sizes, {row label: [cell text]}) from either format of the Measurements modal."""
    body = section_html[section_html.find("measurements-detail"):section_html.find("additional-info")]
    if "<table" in body:
        head = [text_of(h) for h in re.findall(r"<th>(.*?)</th>", body, re.S)]
        rows = {}
        for tr in re.findall(r"<tr>(.*?)</tr>", re.search(r"<tbody>(.*?)</tbody>", body, re.S).group(1), re.S):
            cells = [text_of(c) for c in re.findall(r"<td>(.*?)</td>", tr, re.S)]
            rows[cells[0]] = cells[1:]
        return head[1:], rows
    cols = []
    for ul in re.findall(r"<ul>(.*?)</ul>", body, re.S):
        cells = []
        for li in re.findall(r"<li[^>]*>(.*?)</li>", ul, re.S):
            # A <br> inside one <li> renders as two stacked cells (seen on CT-120x).
            cells.extend(text_of(part) for part in re.split(r"<br\s*/?>", li))
        if cells:
            cols.append(cells)
    labels = cols[0][1:]
    return [c[0] for c in cols[1:]], {lab: [c[k] if k < len(c) else "" for c in cols[1:]] for k, lab in enumerate(labels, start=1)}


# ---------- fabric facts, from the page's own words ----------

def page_text(p):
    parts = (p["body_html"] or "").split("!split!")
    desc = text_of(parts[0])
    details = ""
    for part in parts:
        if "!details!" in part:
            details = part.split("!details!", 1)[1]
    detail_lines = [text_of(li) for li in re.findall(r"<li[^>]*>(.*?)</li>", details, re.S)]
    return desc, detail_lines


def fabric_facts(p):
    desc, details = page_text(p)
    fabric_lines = [l for l in details if re.search(r"denim|selvedge", l, re.I) and not re.search(r"pocket", l, re.I)]
    weight = None
    for l in fabric_lines:
        m = re.search(r"(\d+(?:\.\d+)?)\s*oz", l)
        if m:
            weight = float(m.group(1))
            break
    if weight is None or weight < 8:  # missing, or broken markup (CT-BF1x renders "1" and "3.4oz")
        m = re.search(r"(\d+(?:\.\d+)?)\s*oz", desc)
        weight = float(m.group(1)) if m else None
    if weight is not None and weight.is_integer():
        weight = int(weight)
    cotton = any(re.search(r"100% cotton", l, re.I) for l in fabric_lines)
    text = " ".join([desc] + details).lower()
    if re.search(r"unsanfori[sz]ed", text):
        sanforized, how = False, "page"
    elif re.search(r"sanfori[sz]ed", text):
        sanforized, how = True, "page"
    else:
        sanforized, how = True, "faq"  # FAQ: "Our denim is sanforized"
    if re.search(r"stone ?wash|vintage wash|bleach", text):
        finish = "washed"
    elif re.search(r"\brins(e|ed)\b|garment wash", text):
        finish = "rinsed"
    else:
        finish = "raw"
    return weight, cotton, sanforized, how, finish


def list_price(p):
    amounts = []
    for v in p["variants"]:
        price = float(v["price"])
        cmp = float(v["compare_at_price"]) if v.get("compare_at_price") else 0.0
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


def build_product(p, modal_json, seen, notes):
    cut, pid, name = identify(p)
    url = f"{SITE}/products/{p['handle']}"
    section_html = json.loads(modal_json).get("sizing-modal") or ""
    if "measurements-detail" not in section_html:
        notes["skipped"].append(f"{pid}: no Measurements chart")
        return None, None
    labels, rows = parse_modal(section_html)
    missing = [r for r in ROWS if r not in rows]
    if missing:
        notes["problems"].append(f"{pid}: chart has no {', '.join(missing)} row")
    sizes = []
    for i, label in enumerate(labels):
        meas = {}
        for row, key in ROWS.items():
            cells = rows.get(row)
            if cells is None:
                continue
            if len(cells) != len(labels):
                if i == 0:
                    notes["omitted"].append(f"{pid}: {row} row has {len(cells)} cells for {len(labels)} sizes; left out")
                continue
            try:
                v = cell_value(cells[i])
            except ValueError:
                notes["problems"].append(f"{pid} size {label} {row}: can't read {cells[i]!r}")
                continue
            if v is None:
                notes["omitted"].append(f"{pid} size {label} {row}: blank on the brand's chart")
                continue
            meas[key] = v
        sizes.append({"label": label, "waist_tag": int(label), "measurements": meas})
    chart = {"state": "as-sold", "source": {"url": url, "seen": seen}, "unit": "in", "from_image": False}
    ratios = [s["measurements"]["waist"] / s["waist_tag"] for s in sizes if "waist" in s["measurements"]]
    if ratios and statistics.median(ratios) < 0.75:  # this chart publishes a flat half-waist
        chart["conventions"] = dict(CONVENTIONS, waist="flat")
        notes["flat"].append(pid)
    chart["sizes"] = sizes
    weight, cotton, sanforized, how, finish = fabric_facts(p)
    o = OVERRIDES.get(pid, {})
    weight, sanforized, finish = o.get("weight_oz", weight), o.get("sanforized", sanforized), o.get("finish", finish)
    if how == "faq":
        notes["faq"].append(pid)
    notes["facts"].append(f"{pid}: {weight}oz, {'100% cotton' if cotton else 'composition not stated'}, "
                          f"sanforized={sanforized} ({how}), finish={finish}")
    amount, varies = list_price(p)
    if varies:
        notes["problems"].append(f"{pid}: list price differs between sizes; used the most common one")
    product = {
        "id": pid,
        "name": name,
        "url": url,
        "status": "current",
        "status_seen": seen,
        "fabric": {"weight_oz": weight, "composition": [{"fibre": "cotton", "percent": 100}] if cotton else None,
                   "sanforized": sanforized},
        "sold": "raw" if finish == "raw" else "one-washed",
        "finish": finish,
        "list_price": {"amount": amount, "currency": "USD", "url": url, "seen": seen},
        "shrinkage": None,  # 3sixteen gives no shrinkage figures ("minimal" / "no shrinkage")
        "charts": [chart],
    }
    return cut, product


def main():
    root = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--date", default=datetime.date.today().isoformat())
    ap.add_argument("--cache")
    ap.add_argument("--out", default=os.path.join(root, "catalogue", "3sixteen"))
    ap.add_argument("--offline", action="store_true")
    ap.add_argument("--only")
    a = ap.parse_args()
    cache = a.cache or os.path.join(tempfile.gettempdir(), "real-denim-fit", "3sixteen", a.date)
    fetch = Fetcher(cache, a.offline)

    jeans = [p for p in all_products(fetch) if identify(p)]
    if a.only:
        jeans = [p for p in jeans if p["handle"] == a.only]
        if not jeans:
            raise SystemExit(f"{a.only} isn't a 3sixteen jean in products.json")
    notes = {k: [] for k in ("skipped", "problems", "omitted", "flat", "faq", "facts")}
    cuts = {}
    for p in jeans:
        cut = identify(p)[0]
        if cut not in CUTS:
            notes["problems"].append(f"{p['handle']}: new jean '{p['title']}': decide its cut and add it to CUTS/COLLABS")
            continue
        modal = fetch.get(f"{SITE}/products/{p['handle']}?sections=sizing-modal", os.path.join("modal", p["handle"] + ".json"))
        cut, product = build_product(p, modal, a.date, notes)
        if product:
            cuts.setdefault(cut, []).append(product)

    if a.only:
        for prods in cuts.values():
            for prod in prods:
                print(dumps(prod))
    else:
        os.makedirs(a.out, exist_ok=True)
        for key, prods in cuts.items():
            name, label = CUTS[key]
            write_json(os.path.join(a.out, f"{key}.json"), {
                "id": key, "brand": "3sixteen", "name": name, "brand_label": label,
                "line": None,  # the site doesn't file its jeans as men's, women's or unisex
                "products": sorted(prods, key=lambda x: x["id"]),
            })
        write_json(os.path.join(a.out, "brand.json"),
                   dict(BRAND, method_source={"url": f"{SITE}/pages/measuring-guide-jeans", "seen": a.date}))

    err = sys.stderr
    total = sum(len(v) for v in cuts.values())
    print(f"{total} products, {sum(len(x['charts'][0]['sizes']) for v in cuts.values() for x in v)} sizes, cuts: "
          + ", ".join(f"{k} ({len(v)})" for k, v in cuts.items()), file=err)
    print(f"cache: {cache}", file=err)
    print("fabric facts as inferred (check new products against their pages):", file=err)
    for line in notes["facts"]:
        print("  " + line, file=err)
    print(f"flat-waist charts: {', '.join(notes['flat']) or 'none'}", file=err)
    print(f"sanforized from the FAQ: {', '.join(notes['faq']) or 'none'}", file=err)
    for line in notes["omitted"] + notes["skipped"]:
        print("left out: " + line, file=err)
    for line in notes["problems"]:
        print("PROBLEM " + line, file=err)
    if notes["problems"]:
        sys.exit(1)


if __name__ == "__main__":
    main()
