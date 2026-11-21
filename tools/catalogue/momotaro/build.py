"""Build catalogue/momotaro/*.json from the output of scrape.js.

Usage (Python 3, standard library only):
    python3 tools/catalogue/momotaro/build.py momotaro-raw.json [--out catalogue/momotaro] [--seen YYYY-MM-DD]

    momotaro-raw.json  what scrape.js produced (window.__momotaro.raw) in a tab on momotaro-jeans.com
    --out              where to write brand.json and one file per cut (default: catalogue/momotaro)
    --seen             the date the pages were read (default: today)
    --only PID,...     build only these product ids (for checks; write to a scratch --out)

Output: brand.json plus 100.json, 200.json, ... (one per cut), in the format of
docs/catalogue-format.md. It prints the product and size counts, and warnings for:
  - products in the table below that the scrape didn't find (gone from the site?)
  - scraped products with an inseam chart that aren't in the table or in SKIP (new?)

What's hand-curated here, and must be reviewed on every run (see EXTRACTING.md):
  PRODUCTS  which page is which cut and fabric, with weight, composition, sold, finish
  DROP      numbers left out because the brand's own chart is impossible or out of order
  SKIP      scraped products deliberately left out, with the reason
"""
import argparse, datetime, json, os, sys
from collections import OrderedDict

ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
ap.add_argument("raw")
ap.add_argument("--out", default=os.path.join(os.path.dirname(__file__), "..", "..", "..", "catalogue", "momotaro"))
ap.add_argument("--seen", default=datetime.date.today().isoformat())
ap.add_argument("--only", default=None)
args = ap.parse_args()
OUT, SEEN = args.out, args.seen
ONLY = set(args.only.split(",")) if args.only else None
BASE = "https://www.momotaro-jeans.com/en_US/product/"

raw = json.load(open(args.raw))
for pid, v in raw.items():
    if not v["p"].split("|")[1].startswith("$"):
        sys.exit(f"{pid} is priced {v['p']!r}, not USD: set Ship to: United States and scrape again")

# Scraped products with an inseam chart that are deliberately not in the catalogue.
SKIP = {
    "MMJE6401": "#100 Kyotomontsuki Dyed: its page shows the #400 Wide chart",
    "MMPT6400": "corduroy", "MMPT6401": "smooth leather", "MMPT6402": "suede leather",
    "MXPT6000": "denim trousers, not jeans",
    "MXSP6000": "shorts", "MXSP6001": "shorts", "MMSP6000": "shorts", "MMSP6001": "shorts",
    "MMPT6004": "duck canvas", "MMPT6006": "satin", "MMPT6007": "trousers", "MMPT6008": "sashiko",
    "MMPT6009": "cotton silk cargo", "MMPT6010": "corduroy", "MMPT6011": "cargo", "MMPT6012": "duck",
    "MMPT6015": "wabash trousers", "MMPT6016": "cargo", "MXPT1021": "sashiko", "MXPT1023": "sashiko",
}

URLS = {
    "M100CA": "-100-cashmere-straight-13.7oz-zipper-fly/M100CA.html",
    "M100CL": "-100-classic-straight-15.7oz/M100CL.html",
    "M100SL": "-100-silk-straight-15oz-zipper-fly/M100SL.html",
    "M100ST": "-100-standard-straight-14.7oz/M100ST.html",
    "M110ST": "-110-standard-straight-womens--14.7oz-zipper-fly/M110ST.html",
    "M200CA": "-200-cashmere-tapered-13.7oz-zipper-fly/M200CA.html",
    "M200CL": "-200-classic-tapered-15.7oz/M200CL.html",
    "M200SL": "-200-silk-tapered-15oz-zipper-fly/M200SL.html",
    "M200ST": "-200-standard-tapered-14.7oz/M200ST.html",
    "M300CL": "-300-classic-slim-15.7oz/M300CL.html",
    "M300SL": "-300-silk-slim-15oz-zipper-fly/M300SL.html",
    "M300ST": "-300-standard-slim-14.7oz/M300ST.html",
    "M400ST": "-400-standard-wide-14.7oz/M400ST.html",
    "M410ST": "-410-standard-wide-womens-14.7oz-zipper-fly/M410ST.html",
    "M500ST": "-500--standard-narrow-straight-14.7oz-button-fly/M500ST.html",
    "MMJB6401": "-100-straight---shikon/MMJB6401.html",
    "MMJB6402": "-100-straight---black-sesame/MMJB6402.html",
    "MMJB6404": "-100-straight---kyotomontsuki-dyed/MMJB6404.html",
    "MMJB6500": "-400-wide---high-fade-blue/MMJB6500.html",
    "MMJB6501": "-400-wide---shikon/MMJB6501.html",
    "MMJB6700": "-500-narrow-straight---black-sesame/MMJB6700.html",
    "MMJE6300": "-200-tapered---onigurumi-dyed/MMJE6300.html",
    "MMJE6401": "-100-straight---kyotomontsuki-dyed/MMJE6401.html",
    "MMJE6500": "-400-wide---kyotomontsuki-dyed/MMJE6500.html",
    "MMJZ0109": "-100-natural-indigo-straight-12.2oz/MMJZ0109.html",
    "MMJZ03035S": "-300-cashmere-slim-13.7oz-zipper-fly/MMJZ03035S.html",
    "MMJZ6402": "-100-straight---bc/MMJZ6402.html",
    "MMJZ6403": "-100-straight---moss/MMJZ6403.html",
    "MWJZ6500": "super-wide-womens-11oz/MWJZ6500.html",
    "MWJZ6600": "flare-womens-11oz/MWJZ6600.html",
    "MXJE1303": "-200-tapered---11oz/MXJE1303.html",
    "MXJE1403": "-100-straight---high-fade-blue/MXJE1403.html",
    "MXJE1404": "-100-straight---sakura-dyed/MXJE1404.html",
    "MXJE1405": "-100-straight---11oz/MXJE1405.html",
    "MXJE1506": "-400-wide---11oz/MXJE1506.html",
}

COTTON = [{"fibre": "cotton", "percent": 100}]
SILK = [{"fibre": "cotton", "percent": 58}, {"fibre": "silk", "percent": 42}]
CASHMERE = [{"fibre": "cotton", "percent": 65}, {"fibre": "cashmere", "percent": 35}]

# Fabric lines, as the product pages state them. finish: raw / rinsed / washed / None.
STANDARD = dict(weight_oz=14.7, composition=COTTON, sanforized=None, sold="raw", finish="raw")
CLASSIC = dict(weight_oz=15.7, composition=COTTON, sanforized=False, sold="one-washed", finish="rinsed")
SILKF = dict(weight_oz=15, composition=SILK, sanforized=None, sold="raw", finish="raw")
CASHF = dict(weight_oz=13.7, composition=CASHMERE, sanforized=None, sold="raw", finish="raw")
OZ11_RIGID = dict(weight_oz=11, composition=COTTON, sanforized=None, sold="raw", finish="raw")
OZ11 = dict(weight_oz=11, composition=COTTON, sanforized=None, sold=None, finish=None)
DYED = dict(weight_oz=None, composition=COTTON, sanforized=None, sold=None, finish=None)
# "a subtle, pale blue wash on dark denim through bleaching"
HIGH_FADE = dict(weight_oz=None, composition=COTTON, sanforized=None, sold="one-washed", finish="washed")

# (cut id, product id, name, pid, fabric)
PRODUCTS = [
    ("100", "100-standard-14-7oz-indigo", "Standard 14.7oz, Indigo", "M100ST", STANDARD),
    ("100", "100-standard-14-7oz-black", "Standard 14.7oz, Black", "M100ST", STANDARD),
    ("100", "100-classic-15-7oz-indigo", "Classic 15.7oz, Indigo", "M100CL", CLASSIC),
    ("100", "100-classic-15-7oz-black", "Classic 15.7oz, Black", "M100CL", CLASSIC),
    ("100", "100-silk-15oz", "Silk 15oz", "M100SL", SILKF),
    ("100", "100-cashmere-13-7oz", "Cashmere 13.7oz", "M100CA", CASHF),
    ("100", "100-natural-indigo-12-2oz", "Natural Indigo 12.2oz", "MMJZ0109",
     dict(weight_oz=12.2, composition=COTTON, sanforized=None, sold="raw", finish="raw")),
    ("100", "100-11oz", "11oz", "MXJE1405", OZ11),
    ("100", "100-high-fade-blue", "High Fade Blue", "MXJE1403", HIGH_FADE),
    ("100", "100-sakura-dyed", "Sakura Dyed", "MXJE1404",
     dict(weight_oz=None, composition=COTTON, sanforized=None, sold="raw", finish="raw")),
    ("100", "100-shikon", "Shikon", "MMJB6401", DYED),
    ("100", "100-black-sesame", "Black Sesame", "MMJB6402", DYED),
    ("100", "100-kyotomontsuki-dyed", "Kyotomontsuki Dyed", "MMJB6404", DYED),
    ("100", "100-bc", "BC", "MMJZ6402", DYED),
    ("100", "100-moss", "Moss", "MMJZ6403", DYED),
    ("200", "200-standard-14-7oz-indigo", "Standard 14.7oz, Indigo", "M200ST", STANDARD),
    ("200", "200-standard-14-7oz-black", "Standard 14.7oz, Black", "M200ST", STANDARD),
    ("200", "200-classic-15-7oz-indigo", "Classic 15.7oz, Indigo", "M200CL", CLASSIC),
    ("200", "200-classic-15-7oz-black", "Classic 15.7oz, Black", "M200CL", CLASSIC),
    ("200", "200-silk-15oz", "Silk 15oz", "M200SL", SILKF),
    ("200", "200-cashmere-13-7oz", "Cashmere 13.7oz", "M200CA", CASHF),
    ("200", "200-11oz", "11oz", "MXJE1303", OZ11),
    ("200", "200-onigurumi-dyed", "Onigurumi Dyed", "MMJE6300",
     dict(weight_oz=15, composition=COTTON, sanforized=None, sold=None, finish=None)),
    ("300", "300-standard-14-7oz-indigo", "Standard 14.7oz, Indigo", "M300ST", STANDARD),
    ("300", "300-standard-14-7oz-black", "Standard 14.7oz, Black", "M300ST", STANDARD),
    ("300", "300-classic-15-7oz-indigo", "Classic 15.7oz, Indigo", "M300CL", CLASSIC),
    ("300", "300-classic-15-7oz-black", "Classic 15.7oz, Black", "M300CL", CLASSIC),
    ("300", "300-silk-15oz", "Silk 15oz", "M300SL", SILKF),
    ("300", "300-cashmere-13-7oz", "Cashmere 13.7oz", "MMJZ03035S", CASHF),
    ("400", "400-standard-14-7oz-indigo", "Standard 14.7oz, Indigo", "M400ST", STANDARD),
    ("400", "400-standard-14-7oz-black", "Standard 14.7oz, Black", "M400ST", STANDARD),
    ("400", "400-11oz", "11oz", "MXJE1506", OZ11),
    ("400", "400-high-fade-blue", "High Fade Blue", "MMJB6500", HIGH_FADE),
    ("400", "400-shikon", "Shikon", "MMJB6501", DYED),
    ("400", "400-kyotomontsuki-dyed", "Kyotomontsuki Dyed", "MMJE6500", DYED),
    ("500", "500-standard-14-7oz", "Standard 14.7oz, Indigo", "M500ST", STANDARD),
    ("500", "500-black-sesame", "Black Sesame", "MMJB6700", DYED),
    ("110", "110-standard-14-7oz", "Standard 14.7oz, Indigo", "M110ST", STANDARD),
    ("410", "410-standard-14-7oz", "Standard 14.7oz, Indigo", "M410ST", STANDARD),
    ("super-wide-womens", "super-wide-womens-11oz", "11oz", "MWJZ6500", OZ11_RIGID),
    ("flare-womens", "flare-womens-11oz", "11oz", "MWJZ6600", OZ11_RIGID),
]

# (name, brand_label, line). Momotaro files only its women's cuts in a category of
# their own ("WOMENS JEANS"); the rest sit under plain "JEANS", so their line is null.
CUTS = OrderedDict([
    ("100", ("#100", "Straight", None)),
    ("200", ("#200", "Tapered", None)),
    ("300", ("#300", "Slim", None)),
    ("400", ("#400", "Wide", None)),
    ("500", ("#500", "Narrow Straight", None)),
    ("110", ("#110", "Straight", "women")),
    ("410", ("#410", "Wide", "women")),
    ("super-wide-womens", ("Super Wide Womens", "Super Wide", "women")),
    ("flare-womens", ("Flare Womens", "Flare", "women")),
])

# Numbers left out: (pid, size label) -> set of keys. Every one is explained in EXTRACTING.md.
DROP = {
    ("MMJZ6402", "34"): {"seat"},          # hip published as "15"
    ("MMJZ6403", "34"): {"seat"},          # hip published as "15"
    ("MXJE1506", "36"): {"leg_opening"},   # hem 27.7 between 25.1 and 26.3
    ("M300SL", "40"): {"front_rise"},      # rise 28.2, below every smaller size
    ("M300SL", "42"): {"front_rise"},      # rise 29, below every smaller size
}
for s in ("30", "32", "34", "36", "38"):  # hems 24.4..26.8 jump 3.6 cm from size 28's 20.8
    DROP[("MXJE1405", s)] = {"leg_opening"}

KEYMAP = {"hem": "leg_opening", "waist": "waist", "rise": "front_rise", "thigh": "thigh",
          "inseam": "inseam", "hip": "seat"}
ORDER = ["waist", "front_rise", "seat", "thigh", "leg_opening", "inseam"]


def num(s):
    f = float(s)
    return int(f) if f == int(f) else f


def sizes_for(pid):
    d = raw[pid]
    header = [h.strip().lower() for h in d["r"][0].split(",")][1:]
    keys = [KEYMAP[h] for h in header]
    sold = set(d["s"])
    out = []
    for row in d["r"][1:]:
        cells = row.split(",")
        label = cells[0]
        if label not in sold:  # in the chart but not offered on the page
            continue
        m = {}
        for k, v in zip(keys, cells[1:]):
            if v.strip() == "":
                continue
            if k in DROP.get((pid, label), set()):
                continue
            m[k] = num(v)
        m = OrderedDict((k, m[k]) for k in ORDER if k in m)
        out.append(OrderedDict([("label", label), ("waist_tag", int(label)), ("measurements", m)]))
    return out


def price(pid):
    return num(raw[pid]["p"].split("|")[0])


table_pids = {row[3] for row in PRODUCTS}
for pid in sorted(table_pids - set(raw)):
    if not ONLY or pid in ONLY:
        print(f"WARNING: {pid} is in PRODUCTS but wasn't scraped (gone from the site?)", file=sys.stderr)
for pid in sorted(set(raw) - table_pids - set(SKIP)):
    print(f"WARNING: {pid} {raw[pid]['t']!r} has an inseam chart but isn't in PRODUCTS or SKIP (new?)", file=sys.stderr)

cuts = OrderedDict((c, []) for c in CUTS)
for cut, prod_id, name, pid, fab in PRODUCTS:
    if pid not in raw or (ONLY and pid not in ONLY):
        continue
    url = BASE + URLS[pid] if pid in URLS else raw[pid]["url"]
    p = OrderedDict()
    p["id"] = prod_id
    p["name"] = name
    p["url"] = url
    p["status"] = "current"
    p["status_seen"] = SEEN
    p["fabric"] = OrderedDict([("weight_oz", fab["weight_oz"]), ("composition", fab["composition"]),
                               ("sanforized", fab["sanforized"])])
    p["sold"] = fab["sold"]
    p["finish"] = fab["finish"]
    p["list_price"] = OrderedDict([("amount", price(pid)), ("currency", "USD"), ("url", url), ("seen", SEEN)])
    p["shrinkage"] = None
    p["charts"] = [OrderedDict([
        ("state", "as-sold"),
        ("source", OrderedDict([("url", url), ("seen", SEEN)])),
        ("unit", "cm"),
        ("from_image", False),
        ("sizes", sizes_for(pid)),
    ])]
    cuts[cut].append(p)


INLINE_LISTS = {"composition", "sizes"}  # each element of these lists goes on one line


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
for cut, prods in cuts.items():
    if not prods:
        continue
    name, label, line = CUTS[cut]
    dump(OrderedDict([("id", cut), ("brand", "momotaro"), ("name", name), ("brand_label", label),
                      ("line", line), ("products", prods)]), os.path.join(OUT, cut + ".json"))

dump(OrderedDict([
    ("id", "momotaro"),
    ("name", "Momotaro Jeans"),
    ("site", "https://www.momotaro-jeans.com"),
    ("currency", "USD"),
    ("sizing", "waist"),
    ("tolerance", OrderedDict([("value", 1), ("unit", "in")])),
    ("conventions", OrderedDict([("waist", "around"), ("seat", "around"), ("thigh", "flat"),
                                 ("leg_opening", "flat")])),
    ("measuring_points", OrderedDict([("thigh", "at-crotch"), ("seat", "unknown")])),
    ("method_source", OrderedDict([("url", "https://www.momotaro-jeans.com/en_US/sizing-guide.html"),
                                   ("seen", SEEN)])),
]), os.path.join(OUT, "brand.json"))

n_sizes = sum(len(p["charts"][0]["sizes"]) for ps in cuts.values() for p in ps)
print(f"{sum(len(ps) for ps in cuts.values())} products, {n_sizes} sizes")
