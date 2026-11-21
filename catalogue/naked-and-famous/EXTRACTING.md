# Naked & Famous: extracting the catalogue

Last full run: **2026-11-20**. 262 products in 22 cuts, 2,971 sizes (as-sold), 7 products with an after-wash chart too.

## What's in scope

- **Products:** every product in `https://nakedandfamousdenim.com/products.json?limit=250&page=N` (pages 1–5 on this run) with `product_type` "Jeans" and at least one size in stock (`variants[].available`). That was 290 of 594 on this run.
- **Why in stock:** "currently sells". N&F hides sold-out products from its Men's Jeans collection. The 304 sold-out jeans, mostly past seasons and including every Prince of Slub, were not added. A later run could add them as `"current"` if the catalogue wants them for picking a reference pair.
- **Women's cuts are included:** True Girl, Super Girl, Strong Girl, Max, High Skinny, Skinny, Bestie, Arrow, Maudie, Maudie Flared, Wide Wild West, Wide Leg Trouser and Classic.
- **Fabrics:** jeans in canvas, hickory, dobby and herringbone are included, because N&F sells them as jeans. Shorts, corduroy, linen and tweed are not (see Skipped).

## Where the charts are

- **The panel:** each product page has `<div id="nfd-size-chart-panel">`. Its attributes hold HTML-escaped JSON:
  - `data-columns`: the header row, e.g. `["Tag Size","28","29",…]`. The first cell varies: `Tag Size`, `TAG SIZE`, `tagsize`, `TAGSIZE`, `Tagsize`, `Tag size` or `SIZE / PRE WASH`.
  - `data-rows`: `[{"label","unit","values"}]`, plus `"convention"` and `"sizeScale"` on newer charts.
  - `data-post-columns` / `data-post-rows`: the after-wash chart, on the Cat's Pyjamas (and Prince of Slub) template.
- **Not in products.json.** Some pages repeat the chart in other elements (`data-cats-pyjamas-chart`). Check that the copies agree with the panel; they all did.
- **How the page shows it:** the page's script prints the cells verbatim as inches and works out cm as inches × 2.54. So record the inch values.
- **No panel means no chart.** A browser check confirmed nothing loads later.

## Reading a chart

- **Labels:** case and spacing vary ("UPPER THIGH", "upper thigh", "backrise", "back  rise", "THIGH"). Compare the letters only:
  - waist
  - front rise
  - back rise, also "Black Rise", a typo on two Wide Wild West charts
  - upper thigh or thigh
  - knee
  - leg opening
  - inseam
  - hips → `seat`
- **Unit** is `"in"` or `null`. Null charts render as inches too.
- **Cells** are strings. `null` or empty means not published. `"12."` is 12 and `"29.00"` is 29.
- **Empty size columns** (often 27, 42 or 44) are left out.
- **`convention`** is `circumference` for the waist, and `flat` or `length` for the rest. They all agree with the measuring guide, so no chart overrides `brand.json`. A few rows say `unknown`; ignore them, and ignore `sizeScale`.
- **After-wash charts, Cat's Pyjamas template:** the chart is in `data-post-rows`. The page labels it "Post-wash · Estimated" and "Estimated after the first wash", so it gets `estimated_by_brand: true`.
- **After-wash charts, Hedorah template:** the header is `SIZE / PRE WASH`. A separator row `POST WASH*` has `*` in every cell, and the rows after it end in `*` (`Waist*`, `Knee*` and so on); those are the after-wash rows. The page's "Unsanforized Denim Notice" calls them "expected measurements after the first wash", so they're estimated. The notice also gives shrinkage (3% width, 6% length), which is recorded.
- **The page `<title>` can be wrong:** the Coffee Dye page's title says "Workforce". Use the title from `products.json`, which matches the page's h1.

## How N&F measures (brand.json)

- **Measuring guide:** [copy-of-measuring-guide](https://nakedandfamousdenim.com/pages/copy-of-measuring-guide), checked 2026-11-20:
  - the waist is measured flat and doubled
  - the thigh is straight across from the far edge of the crotch seam
  - the knee is 12″ down the inseam from the crotch
  - everything else is flat
- **Seat:** the guide doesn't cover hips, so the seat's measuring point is `unknown`. Hips appear only on women's charts.
- **Tolerance:** ±0.5″, stated in every chart's footer.
- The site footer's "Measuring Guide" link goes to a retailer (tateandyoko.com). Use N&F's own page.

## Fabric, state, shrinkage, price

- **Fabric** comes from the description's spec lines: `products.json` `body_html`, the same text as the page's "Fabric & details". For example, "13.6oz Japanese Selvedge Denim" and "98% Cotton / 2% Elastane".
  - Ignore leather-patch weights ("5oz vegetable-tanned leather patch").
  - When there's no spec line, take the weight from the prose, but only if the prose gives a single weight.
  - "Polyurethane" or "PU" is recorded as elastane: it's the Japanese labelling term.
  - "2% Stretch" on the Cashmere Stretch Blend Denim is recorded as elastane, because the brand calls it a stretch denim.
  - Organic, FoxFibre® and recycled cotton are recorded as cotton.
- **Sanforized:**
  - `false` where the description says unsanforized or loomstate.
  - Otherwise `true`, from the [FAQ](https://nakedandfamousdenim.com/pages/faq): "All of our jeans are sanforized unless otherwise specified in the item description."
- **Sold:**
  - `one-washed` where the description says the jeans themselves were washed: "Rinse-washed", "each pair has undergone a rinse wash", "This denim has been rinsed".
  - Otherwise `raw`. The site footer says "no washes, no distressing".
  - Treatments to the fabric before sewing don't count as a wash: "Fabric rinsed", "enzyme washed in Japan", "fabric bleached", "tempi treated".
- **Shrinkage:**
  - Sanforized raw products get `percent`, length 2 and width 2, from the [Raw Denim Guide](https://nakedandfamousdenim.com/pages/raw-denim): "will shrink only about 1–3% after the first wash".
  - Washed products get `null`.
  - Unsanforized products without a figure get `null`: Cat's Pyjamas only says "approximately two sizes", and MIJ9 Tennen Ai gives nothing.
  - Hedorah has its own figure.
- **Products where unsanforized and one chart is right:**
  - MIJ14 Haru Kaze is "Rinse-washed, Tempi-treated", and the tempi "eliminates shrinkage".
  - True Girl High Noon: "each pair has undergone a rinse wash".
  - MIJ9 Tennen Ai has one chart and no shrinkage figure, so its shrinkage is unknown.
- **Price:**
  - USD, the store's base currency. `products.json` and the page's JSON-LD agree. A browser in Canada shows CAD.
  - When `compare_at_price` > `price` the product is on sale, and the list price is `compare_at_price`. 57 of the 262 were on sale for Black Friday.
- **`brand_label`:**
  - From the [fit guide](https://nakedandfamousdenim.com/pages/fit-guide): "Slim tapered", "Classic tapered" and so on.
  - For cuts not in the guide, from the product page's own "… fit" wording: Stacked Guy "Tight tapered", High Skinny and Skinny "Skinny", Super Girl "Slim tapered", Classic "Timeless straight".
  - `null` for Max, Arrow, Maudie, Skinny Guy and Wide Leg Trouser.

## Skipped (28 products)

**No chart on the page (18):**
- Arrow: Blue Comfort Stretch Selvedge, Blue Wave Selvedge, Ultralight Tech Denim
- Easy Guy: Love & Peace Selvedge, Workforce Selvedge
- High Skinny: Black Cashmere, Forever Blue Stretch Selvedge
- Max: Black Cashmere, Ultralight Tech Denim
- Strong Guy: Workforce Selvedge
- Super Guy: Gateway Selvedge, High Noon Selvedge, Love & Peace Selvedge, Texas Chainsaw Massacre "Leatherface" Selvedge, Workforce Selvedge
- Weird Guy: Night Sky Selvedge, Sakura Slub Selvedge, Texas Chainsaw Massacre "Leatherface" Selvedge

**Not jeans, or not denim (10).** N&F types them "Jeans", but they are:
- Twin Pleat Short, three products: shorts, kept out of N&F's own jeans collections
- Wide Leg Trouser and Wide Wild West, four products in organic faded corduroy
- Relaxed Pleated Trouser, two products in French linen fine canvas (100% linen)
- Wide Leg Trouser, one product in triple yarn twist brushed tweed

## Numbers left out

These are errors in N&F's own charts, confirmed on the rendered page. Check each one again on the next run: N&F may fix them.

- `bestie-9-7oz-2-1-nep-denim-mid-indigo`, size column "3": it sits between 30 and 32 and is probably 31, but the page doesn't say so.
- `easy-guy-hemp-blend-selvedge-indigo-1`, the five columns after 34: the header reads 35, 36, 37, 39, 41, but the product is sold in 36, 38, 40, 42 and 44. The waists (39–47) only follow the rest of the chart if those columns are 36–44. Only 28–34 are kept.
- Single typos, where the intended number isn't known:
  - `bestie-organic-hickory`: seat at 28, published as 2.5
  - `bestie-maekake-dobby-indigo-1`: seat at 27, published as 29.25
  - `true-girl-elephant-2-reborn-indigo`: back rise at 31, published as 152.25
  - `weird-guy-coffee-dye-selvedge-indigo`: waist at 34, published as 27
  - `weird-guy-hickory-selvedge-1`: waist at 29, published as 34
  - `wide-wild-west-maekake-dobby-indigo-1`: waist at 34, published as 36
- `strong-girl-dia-de-los-muertos-bat-selvedge-indigo`: back rise at 34 and 36. Both are 13, after 15.25 at 33, and equal to the front rise, so they look copied from that row.
- Pairs that contradict each other, where it isn't clear which is wrong, so both are left out:
  - `true-guy-loosey-goosey-selvedge-indigo`: waist 45 at 42 and 44 at 44
  - `naked-famous-stacked-guy-black-cobra-stretch-selvedge`: knee 7.5 at 31 and 6.5 at 32
  - `naked-famous-the-skinny-black-power-stretch`: back rise 12.1 at 26 and 11.5 at 27
- The "Hips" row on `maudie-slub-nep-stretch`, `super-girl-workforce-selvedge` and `wide-leg-trouser-natural-indigo-indigo`: it's smaller than the thigh at every size, so it can't be a flat seat measurement.
- `wide-leg-trouser-natural-indigo-indigo`, both rise rows: the front rise (15.25–17) is about 3″ longer than the back rise (12.5–15.5), the reverse of every other N&F chart.
- **Fabric weight**, where the page gives two weights:
  - High Skinny and Skinny Black Power Stretch: 12oz in the spec list, 12.5oz in the prose.
  - True Girl Workforce: 14oz in the spec list, 10.9oz in the prose.
- **Composition not stated:** the three Hellraiser "Pinhead" products and True Girl Elephant 14 have `composition: null`.

## Odd, but kept as published

- `bestie-archive-blue-indigo`: leg opening 9.255 at size 29.
- `super-guy-hellraiser-pinhead-black`: leg opening 6.54 at size 33.
- **Skinny:** Black Power Stretch and Stretch Selvedge Raw Denim both jump in thigh from 10.5 (30) to 12.7 (31).
- `easy-guy-greencast-slub-stretch-selvedge-indigo-1`: waist 40.5 (38) to 44.5 (40).
- **Tenths of an inch:** the Skinny and High Skinny charts are in tenths.
- **Inseams** wobble by up to 1″ between sizes on a few charts.

## How to rerun

The extraction script was a throwaway in the session's scratchpad; this section describes everything it did.

1. Fetch `products.json?limit=250&page=1…` until a page is empty. Keep the `product_type == "Jeans"` products with any variant available.
2. Fetch each product page, about 3 s apart, and save it as `<handle>.html`.
3. For each page, read the panel as described above. Cut and product name come from the `products.json` title, split on " - ".
   - Product id: cut id plus the slug of the fabric part.
   - Product name: the fabric part, with " - " replaced by ", ".
   - Then apply the fabric, state, shrinkage and price rules above.
4. Flag anything that breaks the run of a row or contradicts the format, and decide each case by hand, as above:
   - a value more than 2″ (waist 4″) from both its neighbours
   - a seat smaller than the thigh
   - a size smaller than the size before it
5. Validate:
   - every file parses
   - every chart has a waist and a thigh
   - sizes are in order
   - every as-sold waist is within -3 to +9″ of its tag
   - no after-wash number is above its as-sold number
   - values fall in sensible ranges
6. Spot-check sizes in a browser: open "Size chart" on the page and compare. A newsletter popup may cover the page, and unsanforized products open a sizing dialog with pre-wash and post-wash tabs.
