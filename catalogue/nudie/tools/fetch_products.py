"""Find Nudie's dry jeans and extract their product data from nudiejeans.com (en-US).

Usage (Python 3, standard library only; plain HTTP works, no browser needed):
    python3 catalogue/nudie/tools/fetch_products.py WORKDIR [--only URI,URI,...]

    WORKDIR   a scratch folder for the cache and outputs (pages are cached in WORKDIR/pages;
              delete a page to fetch it again)
    --only    look at just these product URIs (for checks), instead of the whole sitemap

What it does, one request at a time, 2 s apart:
  1. reads Nudie's fit records (gender, chips) from /en-US/selection/mens-jeans -> WORKDIR/fits.json
  2. reads /en-US/sitemap/products.xml and keeps URIs that start with a fit's name
     (lean-dean-..., wide-heidi-...), minus the "-pants-" ones (duck canvas etc.)
  3. fetches each of those product pages and pulls the product object out of the page's
     Next.js stream (self.__next_f.push), plus the JSON-LD description
  4. classifies each one (see EXTRACTING.md, "What's in"):
       in  = flagged disp_garmentWash "Dry", AND called dry or raw in Nudie's own text
             (name, description, bullets, meta, listing blurb), AND not "One Wash" in the name,
             AND category Jeans (or Pants that the text calls jeans)
       finish = "rinsed" when the text says the pair was rinsed, else "raw"
Outputs:
  WORKDIR/products.json        the dry products, as fetch_virtusize.py and build.py expect
  WORKDIR/classification.tsv   every candidate with its decision and reason, for review
"""
import argparse, json, os, re, sys, time, urllib.error, urllib.request

UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15"
SITE = "https://www.nudiejeans.com"
PUSH = re.compile(r'self\.__next_f\.push\(\[1,"((?:[^"\\]|\\.)*)"\]\)')
CALLS_DRY = re.compile(r"\b(dry|raw)\b(?![- ]?(clean|bleed|or wet|touch))", re.I)
RINSED = re.compile(r"\b(have|has) been rinsed\b|treated these jeans to a rinse", re.I)


def get(url, path=None, delay=2.0):
    """Fetch url politely (one at a time, then a pause), cached at path if given."""
    if path and os.path.exists(path) and os.path.getsize(path) > 1000:
        return open(path, encoding="utf-8").read()
    try:
        with urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": UA}), timeout=60) as r:
            body = r.read().decode("utf-8")
    except urllib.error.URLError as e:
        print(f"  {url}: {e}", file=sys.stderr)
        body = None
    time.sleep(delay)
    if body is not None and path:
        with open(path, "w", encoding="utf-8") as f:
            f.write(body)
    return body


def stream(html):
    return "".join(json.loads('"' + m.group(1) + '"') for m in PUSH.finditer(html))


def obj_at(s, start):
    depth, in_str, esc = 0, False, False
    for j in range(start, len(s)):
        c = s[j]
        if in_str:
            if esc:
                esc = False
            elif c == "\\":
                esc = True
            elif c == '"':
                in_str = False
            continue
        if c == '"':
            in_str = True
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return s[start:j + 1]
    return None


def fits_from(html):
    fits = {}
    for m in re.finditer(r'"title":"([^"]+)","video":\{.*?"gender":"(\w+)","component":"fit","fit_style":"(\w*)",'
                         r'"attributes":\[(.*?)\]', stream(html)):
        fits[m.group(1)] = [m.group(2), m.group(3), re.findall(r'"text":"([^"]+)"', m.group(4))]
    return fits


def extract(html, uri):
    chunks = stream(html)
    i = chunks.find('"uri":"' + uri + '","productNumber"')
    if i < 0:
        return None
    o = json.loads(obj_at(chunks, chunks.rfind('{"id":', 0, i)))
    attrs = {}
    for a in o.get("attributes") or []:
        n = a["type"]["name"]
        if n.startswith("tr_"):
            continue
        attrs.setdefault(n, []).extend(e.get("value") if isinstance(e.get("value"), str) else json.dumps(e.get("value", e))
                                       for e in a["elements"])
    ld = None
    for m in re.finditer(r'<script type="application/ld\+json"[^>]*>(.*?)</script>', html, re.S):
        try:
            j = json.loads(m.group(1))
        except ValueError:
            continue
        if isinstance(j, dict) and j.get("@type") == "Product":
            offers = j.get("offers")
            offers = offers if isinstance(offers, list) else [offers]
            ld = {"description": j.get("description"),
                  "offers": [{"price": x.get("price"), "currency": x.get("priceCurrency")} for x in offers if x]}
    desc = (o.get("description") or {}).get("raw")
    if isinstance(desc, str) and re.fullmatch(r"\$[0-9a-f]+", desc):
        desc = None  # a reference to another stream chunk; the JSON-LD has the text
    return {
        "id": o["id"], "uri": o["uri"], "pn": o["productNumber"], "name": o["name"],
        "cat": (o.get("category") or {}).get("name"), "available": o.get("available"),
        "price": o.get("price"), "orig": o.get("originalPrice"),
        "short": (o.get("shortDescription") or {}).get("raw"), "desc": desc, "ld": ld,
        "mt": o.get("measurementTable"),
        "items": [f'{x["name"]}|{x["sku"]}' for x in o.get("items") or []],
        "attrs": attrs,
    }


def own_text(p):
    a = p["attrs"]
    parts = [p["name"], p.get("desc"), (p.get("ld") or {}).get("description"), p.get("short")]
    for k in ("prd_bullet_list", "prd_meta_description", "prd_plp_description", "prd_short_description"):
        parts += a.get(k, [])
    return " ".join(x for x in parts if x)


def classify(p):
    """Returns (included, reason, finish)."""
    text = own_text(p)
    flag = (p["attrs"].get("disp_garmentWash") or [None])[0]
    cat = p.get("cat") or []
    if flag != "Dry":
        return False, f"flagged {flag}", None
    if re.search(r"\bone wash\b", p["name"], re.I):
        return False, "flagged Dry but named One Wash (rinsed, not dry)", None
    if not (re.search(r"\bdry\b", p["name"], re.I) or CALLS_DRY.search(text)):
        return False, "flagged Dry but never called dry or raw", None
    if "Jeans" not in cat and not ("Pants" in cat and re.search(r"\bjeans\b", text, re.I)):
        return False, f"category {cat}, not jeans", None
    m = RINSED.search(text)
    return True, "dry", ("rinsed" if m else "raw")


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description="Find Nudie's dry jeans")
    ap.add_argument("workdir")
    ap.add_argument("--only", default=None)
    args = ap.parse_args()
    WORK = args.workdir
    os.makedirs(os.path.join(WORK, "pages"), exist_ok=True)

    fits = fits_from(get(f"{SITE}/en-US/selection/mens-jeans", os.path.join(WORK, "mens-jeans.html")))
    if not fits:
        sys.exit("no fit records found on /en-US/selection/mens-jeans: the page layout changed")
    json.dump(fits, open(os.path.join(WORK, "fits.json"), "w"), indent=1, ensure_ascii=False)
    print(f"{len(fits)} fits", file=sys.stderr)

    if args.only:
        uris = args.only.split(",")
    else:
        sm = get(f"{SITE}/en-US/sitemap/products.xml", os.path.join(WORK, "sitemap-products.xml"))
        all_uris = sorted(set(re.findall(r"/en-US/product/([a-z0-9-]+)</loc>", sm)))
        prefixes = tuple(re.sub(r"[^a-z0-9]+", "-", f.lower()) + "-" for f in fits)
        uris = [u for u in all_uris if u.startswith(prefixes) and "-pants-" not in u]
        print(f"{len(all_uris)} product pages in the sitemap, {len(uris)} jeans candidates", file=sys.stderr)

    products, rows = [], []
    for n, uri in enumerate(uris, 1):
        html = get(f"{SITE}/en-US/product/{uri}", os.path.join(WORK, "pages", uri + ".html"))
        p = extract(html, uri) if html else None
        if not p:
            rows.append((uri, "", "out", "page or product data not found", ""))
            continue
        ok, why, finish = classify(p)
        rows.append((uri, (p["attrs"].get("disp_garmentWash") or [""])[0], "in" if ok else "out", why, finish or ""))
        if ok:
            p["finish"] = finish
            products.append(p)
        if n % 25 == 0:
            print(f"  {n}/{len(uris)}", file=sys.stderr)

    json.dump(products, open(os.path.join(WORK, "products.json"), "w"), indent=1, ensure_ascii=False)
    with open(os.path.join(WORK, "classification.tsv"), "w") as f:
        f.write("uri\tflag\tdecision\treason\tfinish\n")
        for r in rows:
            f.write("\t".join(r) + "\n")
    print(f"{len(products)} dry products of {len(uris)} candidates -> products.json, classification.tsv")
