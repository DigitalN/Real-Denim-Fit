"""Fetch the Virtusize measurement data behind each Nudie product's Size Guide widget.

Usage (Python 3, standard library only):
    python3 catalogue/nudie/tools/fetch_virtusize.py WORKDIR --api-key KEY

    WORKDIR    the folder fetch_products.py wrote products.json into
    --api-key  window.vs.apiKey from a nudiejeans.com product page, read with widget.js AFTER
               the user has accepted Nudie's cookies (the key only appears once they have).
               It isn't stored in the repo.

What it does, one request at a time, 2 s apart, skipping anything already in WORKDIR:
  1. for each product, works out the widget's externalId the way Nudie's Size Guide does
     (first part of the first item's SKU, plus a non-length second part, cut to 6 characters)
     and calls  https://services.virtusize.com/product/check?apiKey=KEY&externalId=ID&version=1
     -> WORKDIR/vs/check-<externalId>.json
  2. when that answer has no productData.sizes but has data.productDataId, loads the store
     product the way the widget does:
     https://api.virtusize.com/a/api/v3/store-products/<productDataId>?format=json
     -> WORKDIR/vs-sp/<productDataId>.json
validProduct: false means Virtusize has nothing for that product (no chart; build.py skips it).
Measurements are in millimetres; build.py turns them into the cm the widget shows.
"""
import argparse, json, os, re, sys, time, urllib.error, urllib.parse, urllib.request

UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15"


def external_id(p):
    sku = p["items"][0].split("|")[1]
    parts = sku.split("-")
    t = parts[0]
    if len(parts) > 1 and not re.match(r"L\d{2}", parts[1]):
        t = t + parts[1]
    return t[:6]


def fetch_json(url, path):
    if os.path.exists(path) and os.path.getsize(path) > 0:
        return json.load(open(path))
    try:
        with urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": UA}), timeout=60) as r:
            body, code = r.read().decode("utf-8"), r.status
    except urllib.error.HTTPError as e:
        body, code = e.read().decode("utf-8", "replace"), e.code  # a 404 still carries the check's JSON
    time.sleep(2)
    with open(path, "w", encoding="utf-8") as f:
        f.write(body)
    print(f"  {os.path.basename(path)}: HTTP {code}", file=sys.stderr)
    return json.loads(body) if body.strip().startswith("{") else {}


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description="Fetch Nudie's Virtusize data")
    ap.add_argument("workdir")
    ap.add_argument("--api-key", required=True)
    args = ap.parse_args()
    W = args.workdir
    os.makedirs(os.path.join(W, "vs"), exist_ok=True)
    os.makedirs(os.path.join(W, "vs-sp"), exist_ok=True)
    products = json.load(open(os.path.join(W, "products.json")))
    missing = []
    for p in products:
        ext = external_id(p)
        q = urllib.parse.urlencode({"apiKey": args.api_key, "externalId": ext, "version": 1})
        chk = fetch_json(f"https://services.virtusize.com/product/check?{q}", os.path.join(W, "vs", f"check-{ext}.json"))
        if (chk.get("productData") or {}).get("sizes"):
            continue
        pid = (chk.get("data") or {}).get("productDataId")
        if not pid:
            missing.append(p["uri"])
            continue
        fetch_json(f"https://api.virtusize.com/a/api/v3/store-products/{pid}?format=json",
                   os.path.join(W, "vs-sp", f"{pid}.json"))
    print(f"{len(products)} products; no Virtusize data for: {missing or 'none'}")
