"""Iron Heart: charts are static tables in Shopify's body_html, between
[SIZE-GUIDE-INCH] and [SIZE-GUIDE-CM]. The fabric's state is in the
"Fabric Focus" block on the rendered product page.

Run: python3 tools/catalogue/iron_heart.py
"""

import json
import re

from common import TODAY, compact_rows, fetch, num, text_of, write_brand

SITE = "https://ironheart.co.uk"
GUIDE = "https://www.ironheart.co.uk/pages/how-we-measure-bottoms"
CUT_NUMBERS = ("555", "634", "666", "777", "888", "1955")
ROWS = {
    "waist": "waist",
    "front rise": "front_rise",
    "rear rise": "back_rise",
    "back rise": "back_rise",
    "thigh": "thigh",
    "knee": "knee",
    "hem": "leg_opening",
    "inseam": "inseam",
}
POINTS = {
    "waist": {"way": "around", "at": "top of the waistband, measured flat and doubled (inferred from the values)"},
    "front_rise": {"way": "length", "at": "crotch seam to the top of the waistband, front"},
    "back_rise": {"way": "length", "at": "crotch seam to the top of the waistband, back"},
    "thigh": {"way": "flat", "at": "crotch"},
    "knee": {"way": "flat", "at": "13in-below-crotch"},
    "leg_opening": {"way": "flat", "at": "hem opening"},
    "inseam": {"way": "length", "at": "crotch seam to hem"},
}


def parse_table(body):
    m = re.search(r"\[SIZE-GUIDE-INCH\](.*?)(\[SIZE-GUIDE-CM\]|$)", body, re.S)
    if not m:
        return []
    table = m.group(1)
    rows = re.findall(r"<tr[^>]*>(.*?)</tr>", table, re.S | re.I)
    cells = [[text_of(c) for c in re.findall(r"<t[hd][^>]*>(.*?)</t[hd]>", r, re.S)] for r in rows]
    head = cells[0][1:]
    sizes = [{"size": s, "tag_waist": num(s)} for s in head]
    for r in cells[1:]:
        key = ROWS.get(r[0].strip().lower())
        if not key:
            raise SystemExit(f"unknown Iron Heart row {r[0]!r}")
        for s, v in zip(sizes, r[1:]):
            if num(v) is not None:
                s[key] = num(v)
    return [s for s in sizes if s["tag_waist"] is not None and len(s) > 2]


def fabric_focus(handle):
    page = text_of(fetch(f"{SITE}/products/{handle}"))
    i = page.find("Fabric Focus\n")
    if i < 0:
        return ""
    # The state line and the bullet points under it.
    lines = page[i:].split("\n")[1:8]
    out = [lines[0]] + [ln for ln in lines[1:] if ln.startswith("-")]
    return "\n".join(out)


def stated_shrinkage(focus, sizes, url):
    """Iron Heart states unwashed shrinkage in inches at the waist and
    inseam ("other dimensions shrink proportionately to the waist"). It's
    turned into percentages of the tag-32 size."""
    m = re.search(
        r'approximately (\d+(?:\.\d+)?)" shrinkage in the waist( and inseam|'
        r' and up to (\d+(?:\.\d+)?)" shrinkage on the inseam)',
        focus,
    )
    ref = min(sizes, key=lambda s: abs(s["tag_waist"] - 32), default=None)
    if not m or not ref or "waist" not in ref or "inseam" not in ref:
        return None
    waist_in = float(m.group(1))
    inseam_in = float(m.group(3)) if m.group(3) else waist_in
    sentence = re.search(r"[^\n]*shrinkage in the waist[^\n]*", focus).group(0).lstrip("- ").strip()
    return {
        "width_percent": round(100 * waist_in / ref["waist"], 1),
        "length_percent": round(100 * inseam_in / ref["inseam"], 1),
        "stated": f'{sentence} (as a share of the tag {ref["size"]} waist and inseam)',
        "source": url,
        "seen": TODAY,
    }


def main():
    products = json.loads(fetch(f"{SITE}/collections/bottoms/products.json?limit=250"))["products"]
    out = []
    for p in products:
        handle, title = p["handle"], p["title"]
        cut = next((n for n in CUT_NUMBERS if re.search(rf"(^|-){n}[a-z]*(-|$)", handle.replace("ih-", "-"))), None)
        if handle.startswith("bs69"):
            cut = "777"
        if not cut or "Jeans" not in title and "Cut -" not in title or "Duck" in title:
            continue
        sizes = parse_table(p["body_html"] or "")
        focus = fabric_focus(handle)
        state = focus.split("\n")[0] if focus else ""
        low = (focus + " " + p["body_html"]).lower()
        unsanforized = "unsanforized" in low or "unsanforised" in low or "loomstate" in low
        one_washed = "one-washed" in state.lower()
        weight = re.search(r"(\d+(?:\.\d+)?)(?:/\d+)?oz", title)
        comp = re.search(r"(\d+% ?[A-Za-z]+(?:[ ,/]+\d+% ?[A-Za-z]+)*)", focus)
        label = next((t for t in p["tags"] if t.endswith("Cut")), None)
        fabric = re.sub(r"\s+", " ", title.split(" - ")[0].replace("Jeans", "")).strip()
        fabric = re.sub(r"(Slim |Straight |Relaxed |Vintage |Mid/High Rise )*(Tapered |Straight |Slim )?Cut\s*$", "", fabric).strip()
        colour = title.split(" - ", 1)[1].strip() if " - " in title else ""
        url = f"{SITE}/products/{handle}"
        chart = {
            "state": "as-sold",
            "source": url,
            "seen": TODAY,
            "unit": "in",
            "tolerance": "not stated",
            "from_image": False,
            "points": POINTS,
            "sizes": sizes,
        }
        out.append(
            {
                "id": handle,
                "cut": cut,
                "brand_fit_label": label,
                "name": f"IH-{cut} {fabric} {colour}".strip(),
                "fabric": f"{fabric} {colour}".strip(),
                "fabric_weight_oz": float(weight.group(1)) if weight else None,
                "composition": comp.group(1) if comp else "100% cotton (no elastane in any Iron Heart jeans)",
                "elastane_percent": 0,
                "sanforized": False if unsanforized else True,
                "sold_washed": one_washed,
                "shrinkage": None if one_washed else stated_shrinkage(focus, sizes, url),
                "fabric_state": state or None,
                "list_price": {"amount": float(p["variants"][0]["price"]), "currency": "GBP", "source": url, "seen": TODAY},
                "url": url,
                "on_brands_site": True,
                "charts": [chart] if sizes else [],
            }
        )
    brand = {
        "id": "iron-heart",
        "name": "Iron Heart",
        "site": SITE,
        "measuring_guide": GUIDE,
        "says_where_it_measures": True,
        "sold_in_lengths": False,
        "notes": [
            "Thigh straight across at the crotch (shown only in a photo); knee 13in below the crotch seam.",
            "Waist is a full circumference: the guide doesn't say so; inferred from the values.",
            "Unwashed products (UHR, XHS) state their shrinkage in inches on the product page's Fabric Focus block; it's stored as a share of the tag-32 waist and inseam. Their charts don't say which state they show, so they're taken as sold.",
            "Duck canvas jeans, chinos and work pants are left out: not denim.",
        ],
        "products": out,
    }
    compact_rows(write_brand(brand))


if __name__ == "__main__":
    main()
