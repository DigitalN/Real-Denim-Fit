# Naked & Famous: extracting the catalogue

Last full run: **2026-11-21**. 408 products in 21 cuts, 4,612 sizes (as-sold). 13 products also have an after-wash chart.

## What's in scope

The rules are in "What goes in" in [catalogue-format.md](../../docs/catalogue-format.md).

- **Products:** every product in `https://nakedandfamousdenim.com/products.json?limit=250&page=N` with `product_type` "Jeans", **sold out or not**. On this run that was 594 products over pages 1–5; stop at the first empty page.
  - All are `"status": "current"`, because the catalogue never records stock.
  - Every page fetched (575) returned 200. The 19 sold-out products that were going to be left out as non-denim weren't fetched.
- **Line:** each cut's `line` is how N&F files it.
  - Men's cuts are in the fit guide's men's section or the Men's Jeans collection.
  - Women's cuts are in the women's section or the Women's Jeans collection.
  - No cut is filed under both, so there's no `"unisex"`.
  - The Skinny Guy is in neither collection, has no men's or women's tag and isn't in the fit guide, so its line is `null`.
- **Denim only:** a product counts as denim when N&F calls its cloth denim. Leave out:
  - duck canvas, herringbone (including "Sugiaya"), chino, "Selvedge Slacks" cloth, Maekake dobby, Indigo Sashiko, corduroy, linen and tweed
  - shorts and overalls
- **Kept as denim:**
  - Linen Blend Denim (62% cotton) and Hickory, which N&F calls "Hickory Denim".
  - Trousers cut from denim (the Wide Leg Trouser in 10oz Rinsed Denim and Natural Indigo Denim).
- **Dates:** the 290 products in stock on 2026-11-20 were read that day. The sold-out ones were read on 2026-11-21. Each chart and price carries its own date. `status_seen` is 2026-11-21 for all, when `products.json` still listed every one.

## Where the charts are

- **The panel:** each product page has `<div id="nfd-size-chart-panel">`. Its attributes hold HTML-escaped JSON:
  - `data-columns`: the header row, e.g. `["Tag Size","28","29",…]`. The first cell varies: `Tag Size`, `TAG SIZE`, `tagsize`, `TAGSIZE`, `Tagsize`, `Tag size`, `SIZE / PRE WASH` or `Size / Pre Wash`.
  - `data-rows`: `[{"label","unit","values"}]`, plus `"convention"` and `"sizeScale"` on newer charts.
  - `data-post-columns` / `data-post-rows`: the after-wash chart, on the Cat's Pyjamas and Prince of Slub template.
- **Not in products.json.** Some pages repeat the chart in other elements (`data-cats-pyjamas-chart`). Check that the copies agree with the panel; they all did.
- **How the page shows it:** the page's script prints the cells verbatim as inches and works out cm as inches × 2.54. So record the inch values.
- **No panel means no chart.** Browser checks on an in-stock page and a sold-out one confirmed nothing loads later.

## Reading a chart

- **Labels:** case and spacing vary ("UPPER THIGH", "upper thigh", "backrise", "back  rise", "THIGH", "knee"). Compare the letters only:
  - waist
  - front rise
  - back rise, also "Black Rise", a typo on two Wide Wild West charts
  - upper thigh or thigh
  - knee
  - leg opening
  - inseam
  - hips → `seat`
- **Row order varies:** Hips sometimes sits between the two rises.
- **Unit** is `"in"` or `null`. Null charts render as inches too.
- **Cells** are strings. `null`, empty or `—` means not published. `"12."` is 12 and `"29.00"` is 29.
- **Empty size columns** (often 27, 42 or 44) are left out.
- **`convention`** is `circumference` for the waist, and `flat` or `length` for the rest. They all agree with the measuring guide, so no chart overrides `brand.json`. A few rows say `unknown`; ignore them, and ignore `sizeScale`.
- **After-wash charts, Cat's Pyjamas and Prince of Slub template:** the chart is in `data-post-rows`. The page labels it "Post-wash · Estimated" or "Estimated after the first wash", so it gets `estimated_by_brand: true`.
- **After-wash charts, Hedorah template:** the header is `SIZE / PRE WASH`. A separator row `POST WASH*` has `*` in every cell, and the rows after it end in `*` (`Waist*`, `Knee*` and so on); those are the after-wash rows. The page's "Unsanforized Denim Notice" calls them "expected measurements after the first wash", so they're estimated. The notice also gives shrinkage (3% width, 6% length), which is recorded on all three Hedorah products.
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

- **Fabric** comes from the description's spec lines: `products.json` `body_html`, the same text as the page's "Fabric & details". For example, "13.6oz Japanese Selvedge Denim" and "98% Cotton / 2% Elastane", or "20oz 100% cotton Japanese selvedge denim".
  - Ignore leather-patch weights ("5oz vegetable-tanned leather patch").
  - When there's no spec line, take the weight from the prose, but only if the prose gives a single weight.
  - When the spec list and the prose disagree, the weight is left out.
- **Fibre names:**
  - "Polyurethane" or "PU" is recorded as elastane (20 products): it's the Japanese labelling term.
  - "2% Stretch" is recorded as elastane (14 products, Cashmere Stretch Blend Denim and Fingerprint Denim), because N&F calls both stretch denims.
  - Organic, FoxFibre® and recycled cotton are recorded as cotton.
- **Sanforized:**
  - `false` where the description says unsanforized or loomstate (38 products).
  - Otherwise `true`, from the [FAQ](https://nakedandfamousdenim.com/pages/faq): "All of our jeans are sanforized unless otherwise specified in the item description."
- **Sold:**
  - `one-washed` (20 products) where the description says the jeans themselves were washed: "Rinse-washed", "This jean has been rinse-washed", "each pair has undergone a rinse wash", "This denim has been rinsed".
  - Otherwise `raw`. The site footer says "no washes, no distressing".
  - Treatments to the fabric before sewing don't count as a wash: "Fabric rinsed", "enzyme washed in Japan", "fabric bleached", "tempi treated" on its own.
- **Shrinkage:**
  - Sanforized raw products get `percent`, length 2 and width 2, from the [Raw Denim Guide](https://nakedandfamousdenim.com/pages/raw-denim): "will shrink only about 1–3% after the first wash".
  - Washed products get `null`.
  - Unsanforized products without a figure get `null`: Cat's Pyjamas only says "approximately two sizes".
  - The Hedorah products have their own figure.
- **Unsanforized with one chart:** the Made-in-Japan series (MIJ7, 9, 10, 14, 16, 17, 19) and High Noon. The pages say the fabric is pre-shrunk ("tempi"/"Tenpi treated", "removes the fabric's residual shrinkage", "eliminates shrinkage") or the pair was rinse-washed, so the one chart is the size to choose. Shrinkage is `null`, as for any product that won't shrink, or whose shrinkage the brand doesn't give.
- **Price:**
  - USD, the store's base currency. `products.json` and the page's JSON-LD agree on all 575 pages. A browser in Canada shows CAD.
  - When `compare_at_price` > `price` the product is on sale, and the list price is `compare_at_price`. 92 of the 408 were on sale for Black Friday.
  - A `compare_at_price` of `0.00` means no regular price; use `price`.
  - The MIJ17 Chairo no Kitsune really is $888.
- **`brand_label`:**
  - From the [fit guide](https://nakedandfamousdenim.com/pages/fit-guide): "Slim tapered", "Classic tapered" and so on.
  - For cuts not in the guide, from the product page's own "… fit" wording: Stacked Guy "Tight tapered", High Skinny and Skinny "Skinny", Super Girl "Slim tapered", Classic "Timeless straight".
  - `null` for Max, Arrow, Maudie, Skinny Guy and Wide Leg Trouser.
  - "The Classic" and "Classic" are one cut.

## Skipped (186 products)

**No chart on the page (127):**
- **Easy Guy (25):** 1954 Godzilla; All Natural Ecru Foxfibre; Blue Wave; Burning Godzilla; Charcoal; Deadstock Real Gold; Deep Sea; Dusty Rose Denim; Elephant 13; Elvira; Forever Black; Forever Blue; High Noon; King Ghidorah; Love & Peace; MIJ12 Aomidori; MIJ13 Okayama Spirit Kuro; Mechagodzilla; Night Sky; Prototype; Shin Godzilla; Shinigami; Smokey Cotton Slub; Texas Chainsaw Massacre; Workforce
- **Super Guy (28):** 10 Fold; 1954 Godzilla; All Black Comfort Stretch; Blue Wave; Burning Godzilla; Deadstock Real Gold; Deep Sea; Elvira; Forever Black; Forever Blue; Gateway; High Noon; Jonathan Joestar; King Ghidorah; Love & Peace; MIJ12 Aomidori; MIJ13 Okayama Spirit Kuro; MIJ15 Kakishibu; Mechagodzilla; Night Sky; Pagoda Dyed; Prototype; Sakura Slub; Shin Godzilla; Shinigami; Smokey Cotton Slub; Texas Chainsaw Massacre; Workforce
- **True Guy (24):** 1954 Godzilla; All Natural Ecru Foxfibre; Burning Godzilla; Deadstock Real Gold; Deep Sea; Elephant 13; Elvira; Forever Black; Forever Blue; High Noon; King Ghidorah; Love & Peace; MIJ10 Okayama Spirit; MIJ12 Aomidori (both); MIJ13 Okayama Spirit Kuro; MIJ15 Kakishibu; Mechagodzilla; Prototype; Sakura Slub; Shin Godzilla; Shinigami; Texas Chainsaw Massacre; Workforce
- **Weird Guy (23):** 1954 Godzilla; Cell Perfect; Deadstock Real Gold; Deep Sea; Elephant 12; Elvira; Forever Black; Forever Blue; High Noon; Jolyne Cujoh; Jotaro Kujo; King Ghidorah; Love & Peace; MIJ12 Aomidori; MIJ13 Okayama Spirit Kuro; MIJ15 Kakishibu; Mechagodzilla; Night Sky; Sakura Slub; Shin Godzilla; Shinigami; Texas Chainsaw Massacre; Trunks Future
- **Strong Guy (12):** 10 Fold; 1954 Godzilla; Blue Grass; Burning Godzilla; High Noon; King Ghidorah; Love & Peace; Mechagodzilla; Shin Godzilla; Shinigami; Texas Chainsaw Massacre; Workforce
- **Others (15):**
  - Groovy Guy: 10 Fold
  - Stacked Guy: Blue Comfort Stretch
  - Arrow: Blue Comfort Stretch; Blue Wave; Ultralight Tech Denim
  - Bestie: Shadow Black Denim
  - High Skinny: Black Cashmere; Forever Blue Stretch; Stretch Selvedge Raw
  - Maudie Flared: 2x1 Nep Dungaree Denim; Shadow Black Denim
  - Max: Black Cashmere; Ultralight Tech Denim
  - Strong Girl: Salvaged Selvedge 2
  - Wide Wild West: Shadow Black Denim

**Not denim, or not jeans (59):**
- duck canvas (16)
- herringbone "Sugiaya" (10), plus Hard Twist Organic Herringbone (1) and the Classic Fatigue herringbone pant (1)
- Selvedge Slacks cloth (9)
- Maekake dobby (5)
- Indigo Sashiko (5)
- corduroy (4)
- linen canvas (2)
- tweed (1)
- chino (1)
- shorts (3)
- Weird Guy Overalls (1)

## Numbers left out

These are errors in N&F's own charts. Check each one again on the next run: N&F may fix them.

- **Size headers that can't be trusted:**
  - `bestie-9-7oz-2-1-nep-denim-mid-indigo` "3", which is probably 31.
  - `true-guy-fingerprint-denim-indigo` "3", which is probably 30.
  - `weird-guy-yatagarasu-selvedge-black` "35", which is probably 36. Its 28 and 29 columns are also scrambled (waist 11 and 11.5, thigh 8 and 8.5).
  - `easy-guy-hemp-blend-selvedge-indigo-1`, the five columns after 34: the header reads 35, 36, 37, 39, 41, but the product is sold in 36–44. Only 28–34 are kept.
- **Single typos,** where the intended number isn't known:
  - seat 2.5 at 28 (`bestie-organic-hickory`)
  - back rise 152.25 at 31 (`true-girl-elephant-2-reborn-indigo`)
  - back rise 166 at 42 (`weird-guy-chocolat-milk-selvedge-1`)
  - waist 27 at 34 (`weird-guy-coffee-dye-selvedge-indigo`)
  - waist 34 at 29 (`weird-guy-hickory-selvedge-1`)
  - waist 53.5 at 42 (`easy-guy-10-fold-selvedge`)
  - waist 23 at 42 (Weird Guy Ghostbusters Slimer)
  - waist 18 at 34 (`max-blue-comfort`)
  - knee 1.5 at 36 (`easy-guy-blood-brothers-selvedge-indigo-x-black`)
  - knee 75 at 29 (`high-skinny-black-comfort-stretch`)
  - knee 85 at 28 (Easy Guy Ghostbusters Slimer)
  - knee 0 and 0.25 at 31 and 32 (`true-guy-brown-fox-selvedge-brown`)
  - knee 9.25 at 28, above 8.5 at 29 (`easy-guy-mij11-tokushima-hand-dyed-kakishibu-selvedge`)
  - leg opening 110.5 at 40 (`strong-guy-brawler-selvedge-1`)
  - leg opening 7.5 at 27 against 9.75 at 28 (`strong-guy-greencast-slub-stretch-selvedge-indigo-1`)
- `strong-girl-dia-de-los-muertos-bat-selvedge-indigo`: back rise at 34 and 36. Both are 13, equal to the front rise, after 15.25 at 33.
- **Pairs that contradict each other**, where it isn't clear which is wrong, so both are left out:
  - `true-guy-loosey-goosey-selvedge-indigo`: waist 45 at 42 and 44 at 44
  - `naked-famous-stacked-guy-black-cobra-stretch-selvedge`: knee 7.5 at 31 and 6.5 at 32
  - `naked-famous-the-skinny-black-power-stretch`: back rise 12.1 at 26 and 11.5 at 27
- **Hips rows smaller than the thigh at every size,** on `maudie-slub-nep-stretch`, `super-girl-workforce-selvedge` and `wide-leg-trouser-natural-indigo-indigo`, so they can't be a flat seat measurement.
- `wide-leg-trouser-natural-indigo-indigo`, both rise rows: the front rise is about 3″ longer than the back rise, the reverse of every other N&F chart.
- **Fabric weight,** where the page gives two different weights:
  - High Skinny and Skinny Black Power Stretch
  - True Girl Workforce
  - Strong Guy Stretch Selvedge
  - True Guy Solid Black Selvedge (both)
- **Weight not stated** (`null`): Black Lightweight Denim (Bestie, Wide Wild West), Natural Indigo (Wide Leg Trouser, Wide Wild West) and Maudie Slub Nep Stretch.
- **Composition not stated** (`null`): Hellraiser "Pinhead" (5), MIJ18 Mezuki (4), MIJ19 Amaterasu (4), Super Sexy Selvedge (3) and True Girl Elephant 14.

## Odd, but kept as published

- `bestie-archive-blue-indigo`: leg opening 9.255 at size 29.
- `super-guy-hellraiser-pinhead-black`: leg opening 6.54 at size 33.
- **Skinny:** Black Power Stretch and Stretch Selvedge Raw Denim both jump in thigh from 10.5 (30) to 12.7 (31).
- `easy-guy-greencast-slub-stretch-selvedge-indigo-1`: waist 40.5 (38) to 44.5 (40).
- **Tenths of an inch:** the Skinny and High Skinny charts are in tenths.
- **Inseams** wobble by up to 1″ between sizes on a few charts.

## How to rerun

The extraction script was a throwaway in the session's scratchpad; this section describes everything it did.

1. Fetch `products.json?limit=250&page=1…` until a page is empty, and keep every `product_type == "Jeans"` product.
2. Leave out the non-denim and non-jeans fabrics above. Fetch each remaining product page, about 3 s apart, and save it as `<handle>.html`. Note the date for each page.
3. For each page, read the panel as described above. Cut and product name come from the `products.json` title, split on " - ".
   - Product id: cut id plus the slug of the fabric part.
   - Product name: the fabric part, with " - " replaced by ", ".
   - Then apply the fabric, state, shrinkage and price rules above.
4. Flag anything that breaks the run of a row, falls outside sensible ranges, or contradicts the format, and decide each case by hand, as above:
   - a value more than 2″ (waist 4″) from both its neighbours
   - a waist more than 3″ below or 9″ above its tag
   - a seat smaller than the thigh
   - a size smaller than the size before it
   - a size header that isn't a two-digit tag
5. Validate:
   - every file parses
   - every chart has a waist and a thigh
   - sizes are in order
   - no after-wash number is above its as-sold number
   - values fall in sensible ranges
   - `line` is set on every cut
6. Spot-check sizes in a browser: open "Size chart" on the page and compare. A newsletter popup may cover the page, and unsanforized products open a sizing dialog with pre-wash and post-wash tabs.
