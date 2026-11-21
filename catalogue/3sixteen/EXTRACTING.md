# 3sixteen: extraction notes

Last full run: 2026-11-20. 48 products in 7 cuts, 529 sizes. 6 of the products were sold out on that day.

## Where things are

- **Product list:** `https://www.3sixteen.com/products.json?limit=250&page=N` (2 pages). Prices are USD.
- **Which products are jeans:** titles shaped like `CS-100x ~ Classic Straight - Indigo Selvedge`, where the two letters are the cut (CS, CT, RS, NT, ST) and the code is the fabric, plus two collaborations (below). All of them are denim.
- **Left out:** `bs-103x` (a jean short), and trousers and work pants even where the fabric is denim (Harvest Pant ~ Grey Denim, Officer Chino ~ Nep Denim).
- **Sold-out products are included** with `"status": "current"`. `products.json` still lists them, with every variant `"available": false`; the catalogue never records stock.
- **`line` is `null` on every cut:** the site doesn't file its jeans as men's, women's or unisex. The menu goes Shop → Denim, and `/collections.json` has no gendered collections.
- **Charts:** the product page's "Measurements" button fetches a Shopify section. Get it directly with `https://www.3sixteen.com/products/<handle>?sections=sizing-modal`, which returns JSON whose `sizing-modal` key holds the modal's HTML. It comes in two formats:
  - **Lists** (most products): one `<ul>` per column. The first is the labels (Sizing, Waist, Front Rise, Back Rise, Inseam, Thigh, Knee, Opening), then one per size. Numbers use fraction glyphs (`11½`, `15⅜`), recorded as decimals (11.5, 15.375). The same lists are in `products.json` `body_html` after `!measurements!`; on this run all 367 sizes agreed.
  - **A `<table>`** (newer products such as the 105x/225x "5 Series", RS-120x, RS-203x, RS-220x, RS-222xs, RS-223xv, CS-220x, CS-222x, CS-223xv, RS-33x and Big Jean): decimals, and **not** in `body_html`, so you must use the section.
- **Fabric facts:** `!details!` bullets in `body_html` (weight, "100% cotton", "Sanforized…", "rinsed", "Stonewash") and the description above `!split!`.

## Measuring method

- From the [measuring guide](https://www.3sixteen.com/pages/measuring-guide-jeans):
  - The waist is measured end to end and multiplied by two, so `"around"`.
  - The thigh is 1″ below the crotch.
  - The knee is exactly 14″ below the crotch seam.
- No tolerance is stated, so `tolerance` is `null`.

## Fabric decisions

- **`sold`:** raw, unwashed denim is `"raw"`. Anything rinsed, garment-washed, stonewashed or vintage-washed by the brand is `"one-washed"`. That includes the Kibata (`…xk`), 101x, 222x, BF1x, 33x, `…xs`, `…xv`, Big Jean and the Self Edge collab.
- **`sanforized`:**
  - The Kibata products and the Self Edge collab say "unsanforized", so `false`.
  - Products that say "Sanforized" on their page are `true`.
  - **Where a page doesn't say, it's `true` from the [FAQ](https://www.3sixteen.com/pages/faq):** "Our denim is sanforized". That applies to cs-100xv, cs-101x, cs-222x, cs-223xv, ct-100x, ct-101x, ct-220x, ct-bf1x, nt-100x, nt-120x, nt-220x, nt-222x, rs-100xv, rs-101x, rs-223xv, rs-33x and st-120x.
- **`composition`:** `100% cotton` only where the page says so; otherwise `null`.
- **`shrinkage`:** always `null`. Raw products say "minimal shrinkage" and washed ones "no shrinkage", but none gives an amount.
- **CT-BF1x weight:** the details bullet renders as "1" and "3.4oz" because of broken markup; the description says 13.4oz, which is what's recorded.

## Quirks on this run (check again next time)

- **Flat waist on some charts:** CS-220x, CS-222x, RS-120x, RS-203x, RS-220x and RS-222xs publish the waist as a flat half-measurement (size 32 → 16 to 16.75), unlike every other 3sixteen chart. Those charts carry a `conventions` override with `"waist": "flat"`.
- **CS-222x knee left out:** its table has 9 knee values for 10 sizes, so there's no telling which size is missing. The `knee` key is omitted for every size of that product.
- **RS-101xs, size 31:** the Opening cell is blank, so `leg_opening` is omitted for that size only.
- **CT-120x, sizes 34–36:** Waist and Front Rise sit in one `<li>` separated by `<br>` (`34<br>11½`). They render as two stacked cells, so they're read as waist 34, front rise 11½. The parser splits on `<br>`.
- **NT-100x, size 35:** inseam 35¾ while every other size is 35. Recorded as published; it may be a typo on the brand's side.
- **RS-225x:** inseam jumps from 35 to 38 at size 36, and RS-105x does the same at 36–38. Recorded as published.
- Some charts list sizes the shop doesn't sell, or the other way round (for example, CS-100x is sold in 42 but charted only to 40). Record the chart's sizes only.

## Collaborations (each is its own cut)

- **Throwing Fits "Big Jean"** (`big-jean.json`): the brand calls it "a brand new cut specifically for this project". Sanforized and stonewashed, "no further shrinkage".
- **Self Edge x Samurai x 3sixteen SEXSX3s27-02, Straight Tapered** (`sexsx3s27-02.json`): made in Japan, 15oz, unsanforized and lightly rinsed. Its chart labels sizes 30–44 but lists waists of 28–40, recorded as published.

## Skipped

Nothing with a chart was skipped.
