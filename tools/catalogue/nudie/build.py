"""Build catalogue/nudie/*.json from what fetch_products.py and fetch_virtusize.py saved.

Usage (Python 3, standard library only):
    python3 tools/catalogue/nudie/build.py WORKDIR [--out catalogue/nudie] [--seen YYYY-MM-DD] [--only URI,...]

    WORKDIR  the folder holding products.json, fits.json, vs/ and vs-sp/
    --out    where to write brand.json and one file per cut (default: catalogue/nudie)
    --seen   the date the pages were read (default: today)
    --only   build only these product URIs (for checks; write to a scratch --out)

Output: brand.json plus one file per fit (lean-dean.json, wide-heidi.json, ...), in the format of
docs/catalogue-format.md. It prints a report: product and size counts, products with no
Virtusize data (skipped), sizes sold but missing from Virtusize, and Virtusize names that
differ from Nudie's (harmless abbreviations when the externalId matches).

Rules applied here (see EXTRACTING.md):
  measurements  Virtusize mm / 10 = the cm the widget shows; hip -> seat, legOpening -> leg_opening
  sanforized    true for all ("Yes, all Nudie Jeans are sanforized", help centre)
  shrinkage     a figure in the product's own description beats the care line "May shrink up to N%"
  finish/sold   from fetch_products.py (raw -> sold raw; rinsed -> sold one-washed)
  weight_oz     null when the page states two different weights
  SKIP          products left out on purpose, with the reason
  chart check   a chart identical in every size to a *different* cut's is reported under
                "chart_same_as_other_cut" (with a WARNING until the wrong product is in SKIP);
                identical charts within one cut are normal, Nudie shares them across fabrics
"""
import argparse, datetime, json, os, re, sys
from collections import OrderedDict

ap = argparse.ArgumentParser(description="Build catalogue/nudie")
ap.add_argument("workdir")
ap.add_argument("--out", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "catalogue", "nudie"))
ap.add_argument("--seen", default=datetime.date.today().isoformat())
ap.add_argument("--only", default=None)
args = ap.parse_args()
HERE, OUT, SEEN = args.workdir, args.out, args.seen
ONLY = set(args.only.split(",")) if args.only else None
SITE = "https://www.nudiejeans.com"

products = json.load(open(os.path.join(HERE, "products.json")))
if ONLY:
    products = [p for p in products if p["uri"] in ONLY]
# Nudie's own fit records (Storyblok "global/fits/<fit>"): [gender, fit_style, chips].
FITS = json.load(open(os.path.join(HERE, "fits.json")))

DESC_SHRINK = re.compile(r"([^.]*?inseam shrinkage of (\d+(?:\.\d+)?)% and (\d+(?:\.\d+)?)(?:\s*[-\u2013]\s*(\d+(?:\.\d+)?))?% in width\.)", re.I)


def brand_label(fit):
    # Nudie's chips for the fit, minus the waist and fly ones (rise is worked out from the numbers).
    chips = [c.strip() for c in FITS[fit][2] if not re.search(r"waist|fly", c, re.I)]
    return ", ".join(re.sub(r"\s+Jeans$", "", c) for c in chips) or None


def line(fit):
    g = FITS[fit][0]
    return g if g in ("men", "women", "unisex") else None


def num(x):
    x = float(x)
    return int(x) if x == int(x) else x


def shrinkage(p, url):
    desc = " ".join(t for t in [p.get("desc"), (p.get("ld") or {}).get("description")] if t)
    m = DESC_SHRINK.search(desc)
    if m:  # e.g. "Account for inseam shrinkage of 2% and 4.5-6% in width." -> 2 and the midpoint 5.25
        lo, hi = float(m.group(3)), float(m.group(4) or m.group(3))
        s = {"length": num(m.group(2)), "width": num((lo + hi) / 2), "note": m.group(1).strip()}
    else:
        care = [w for w in p["attrs"].get("ci_washText", []) if re.search(r"shrink", w, re.I)]
        if not care:
            return None
        m = re.fullmatch(r"May shrink up to (\d+(?:\.\d+)?)%", care[0].strip())
        if not m or len(care) > 1:
            raise SystemExit(f"unexpected shrink text on {p['uri']}: {care}")
        s = {"length": num(m.group(1)), "width": num(m.group(1)), "note": care[0].strip()}
    return OrderedDict([("kind", "percent"), ("length", s["length"]), ("width", s["width"]),
                        ("note", s["note"]), ("source", OrderedDict([("url", url), ("seen", SEEN)]))])


VS_KEYS = OrderedDict([("waist", "waist"), ("frontRise", "front_rise"), ("backRise", "back_rise"),
                       ("hip", "seat"), ("thigh", "thigh"), ("knee", "knee"),
                       ("legOpening", "leg_opening"), ("inseam", "inseam")])


def kebab(s):
    return re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-")


def external_id(p):
    # Same rule as Nudie's Size Guide component: first part of the SKU (plus a
    # non-length second part), cut to 6 characters.
    sku = p["items"][0].split("|")[1]
    parts = sku.split("-")
    t = parts[0]
    if len(parts) > 1 and not re.match(r"L\d{2}", parts[1]):
        t = t + parts[1]
    return t[:6]


def weight(p):
    texts = [p["attrs"].get("prd_bullet_list", [""])[0], p.get("short") or "", p.get("desc") or ""]
    found = set()
    for t in texts:
        for m in re.finditer(r"(\d+(?:[.,]\d+)?)\s*oz", t or "", re.I):
            found.add(float(m.group(1).replace(",", ".")))
    if len(found) == 1:
        w = found.pop()
        return int(w) if w == int(w) else w
    return None  # none stated, or conflicting figures


def composition(p):
    raw = (p["attrs"].get("pr_composition") or [None])[0]
    if not raw:
        return None
    parts = re.findall(r"(\d+(?:\.\d+)?)\s*%\s*([A-Za-z][A-Za-z ]*?)(?=\s*\d|\s*,|$)", raw)
    comp = [{"fibre": f.strip().lower(), "percent": (int(float(n)) if float(n) == int(float(n)) else float(n))}
            for n, f in parts]
    if not comp or abs(sum(c["percent"] for c in comp) - 100) > 0.01:
        raise SystemExit(f"composition parse failed for {p['uri']}: {raw!r}")
    return comp


def cm(mm):
    v = mm / 10
    return int(v) if v == int(v) else round(v, 1)


def vs_product(ext):
    """The Virtusize store product for an externalId: inline in the product check, or
    loaded by its productDataId (what the widget does when the check doesn't inline it)."""
    path = os.path.join(HERE, "vs", f"check-{ext}.json")
    if not os.path.exists(path):
        return None, "no check"
    chk = json.load(open(path))
    pd = chk.get("productData")
    if not (pd and pd.get("sizes")):
        pid = (chk.get("data") or {}).get("productDataId")
        if not pid:
            return None, "not in Virtusize (validProduct false)"
        sp = os.path.join(HERE, "vs-sp", f"{pid}.json")
        if not os.path.exists(sp):
            return None, "store product not fetched"
        pd = json.load(open(sp))
    if str(pd.get("externalId")) != ext:
        raise SystemExit(f"externalId mismatch for {ext}: {pd.get('externalId')}")
    return pd, None


def sizes(p, pd):
    by_name = {}
    for s in pd["sizes"]:
        m = re.fullmatch(r"\s*W?(\d+)\s*/\s*L?(\d+)\s*", s["name"])  # "30/32" or "W30/32"
        if not m:
            raise SystemExit(f"unexpected Virtusize size {s['name']!r} for {p['uri']}")
        key = f"{int(m.group(1))}/{int(m.group(2))}"
        if key in by_name:
            raise SystemExit(f"duplicate Virtusize size {key} for {p['uri']}")
        by_name[key] = s["measurements"]
    out, missing = [], []
    for it in p["items"]:
        label = it.split("|")[0]
        m = re.fullmatch(r"W(\d+)/L(\d+)", label)
        if not m:
            raise SystemExit(f"unexpected size label {label!r} on {p['uri']}")
        w, l = int(m.group(1)), int(m.group(2))
        meas = by_name.get(f"{w}/{l}")
        if meas is None:
            missing.append(label)
            continue
        mm = OrderedDict()
        for k, key in VS_KEYS.items():
            v = meas.get(k)
            if isinstance(v, (int, float)) and v > 0:
                mm[key] = cm(v)
        out.append(OrderedDict([("label", label), ("waist_tag", w), ("length_tag", l), ("measurements", mm)]))
    return out, missing


# Products left out on purpose, with the reason (also listed in EXTRACTING.md).
SKIP = {
    "slim-jim-dry-two": "its Virtusize chart is identical, in all 65 sizes, to Regular Alf's (Dry One and "
                        "Dry Selvage), which can't be right for a slim straight cut",
}


def chart_signature(szs):
    return tuple((s["label"], tuple(sorted(s["measurements"].items()))) for s in szs)


report = {"missing_sizes": {}, "no_vs": [], "skipped": {}, "chart_same_as_other_cut": [], "products": 0, "sizes": 0}
signatures = {}  # chart -> [(fit, uri)], for every product that has a chart, skipped or not
cuts = OrderedDict()
for p in sorted(products, key=lambda p: (p["attrs"]["fit"][0], p["name"])):
    fit = p["attrs"]["fit"][0]
    ext = external_id(p)
    pd, why = vs_product(ext)
    if pd is None:
        report["no_vs"].append(f"{p['uri']} ({ext}): {why}")
        continue
    if pd.get("name") != p["name"]:
        report.setdefault("name_mismatch", []).append(f"{p['uri']}: Virtusize name {pd.get('name')!r}")
    szs, missing = sizes(p, pd)
    if missing:
        report["missing_sizes"][p["uri"]] = missing
    if not szs:
        report["no_vs"].append(p["uri"])
        continue
    signatures.setdefault(chart_signature(szs), []).append((fit, p["uri"]))
    if p["uri"] in SKIP:
        report["skipped"][p["uri"]] = SKIP[p["uri"]]
        continue
    url = f"{SITE}/en-US/product/{p['uri']}"
    shrink = shrinkage(p, url)
    price = p["orig"]["value"] if p.get("orig") else p["price"]["value"]
    name = p["name"]
    short = name[len(fit):].strip() if name.startswith(fit) else name
    prod = OrderedDict([
        ("id", p["uri"]),
        ("name", short),
        ("url", url),
        ("status", "current"),
        ("status_seen", SEEN),
        ("fabric", OrderedDict([("weight_oz", weight(p)), ("composition", composition(p)), ("sanforized", True)])),
        ("sold", "raw" if p.get("finish", "raw") == "raw" else "one-washed"),
        ("finish", p.get("finish", "raw")),
        ("list_price", OrderedDict([("amount", price), ("currency", "USD"), ("url", url), ("seen", SEEN)])),
        ("shrinkage", shrink),
        ("charts", [OrderedDict([
            ("state", "as-sold"),
            ("source", OrderedDict([("url", url), ("seen", SEEN)])),
            ("unit", "cm"),
            ("from_image", False),
            ("sizes", szs),
        ])]),
    ])
    cuts.setdefault(fit, []).append(prod)
    report["products"] += 1
    report["sizes"] += len(szs)

# Within a cut, Nudie shares one chart across fabrics, so identical charts are normal there.
# A chart identical to a *different* cut's is a copy-paste error in Nudie's/Virtusize's data:
# flag it, and leave the wrong product out by adding it to SKIP.
for group in signatures.values():
    if len({fit for fit, _ in group}) > 1:
        names = ", ".join(f"{uri} ({fit}{', skipped' if uri in SKIP else ''})" for fit, uri in group)
        report["chart_same_as_other_cut"].append(names)
        if len({fit for fit, uri in group if uri not in SKIP}) > 1:  # not yet resolved by SKIP
            print(f"WARNING: identical chart across cuts: {names}. Find the wrong one and add it to SKIP.",
                  file=sys.stderr)

INLINE_LISTS = {"composition", "sizes"}


def fmt(obj, indent=0, key=None):
    pad = "  " * indent
    if isinstance(obj, dict):
        if not obj:
            return "{}"
        items = [f'{pad}  {json.dumps(k)}: {fmt(v, indent + 1, k)}' for k, v in obj.items()]
        return "{\n" + ",\n".join(items) + "\n" + pad + "}"
    if isinstance(obj, list):
        if not obj:
            return "[]"
        if key in INLINE_LISTS:
            items = [pad + "  " + json.dumps(v, ensure_ascii=False, separators=(", ", ": ")) for v in obj]
        else:
            items = [pad + "  " + fmt(v, indent + 1) for v in obj]
        return "[\n" + ",\n".join(items) + "\n" + pad + "]"
    return json.dumps(obj, ensure_ascii=False)


def dump(obj, path):
    with open(path, "w") as f:
        f.write(fmt(obj) + "\n")
    assert json.load(open(path)) == json.loads(json.dumps(obj))


os.makedirs(OUT, exist_ok=True)
for fit, prods in cuts.items():
    cid = kebab(fit)
    dump(OrderedDict([("id", cid), ("brand", "nudie"), ("name", fit),
                      ("brand_label", brand_label(fit)), ("line", line(fit)), ("products", prods)]),
         os.path.join(OUT, cid + ".json"))

dump(OrderedDict([
    ("id", "nudie"),
    ("name", "Nudie Jeans"),
    ("site", SITE),
    ("currency", "USD"),
    ("sizing", "waist-and-length"),
    ("tolerance", None),
    ("conventions", OrderedDict([("waist", "flat"), ("seat", "flat"), ("thigh", "flat"), ("knee", "flat"),
                                 ("leg_opening", "flat")])),
    ("measuring_points", OrderedDict([("thigh", "at-crotch"), ("knee", "half-inseam"),
                                      ("seat", "16cm-below-waistband")])),
    ("method_source", OrderedDict([("url", f"{SITE}/en-US/help-center/topic/fits-products"), ("seen", SEEN)])),
]), os.path.join(OUT, "brand.json"))

print(json.dumps(report, indent=1))
