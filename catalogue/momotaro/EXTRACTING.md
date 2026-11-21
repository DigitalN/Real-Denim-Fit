# Extracting Momotaro

Last full run: 2026-11-20. 34 product pages became 41 products (7 pages sell the same chart in two fabrics, Indigo and Black) and 421 sizes, in 9 cuts.

## Where things are

- **Site:** `momotarojeans.com` redirects to `www.momotaro-jeans.com`, a Salesforce Commerce Cloud store. Plain `curl` gets **403**, so use the built-in browser. Once a page has loaded, same-origin `fetch()` from that tab works. Fetch one page at a time, about 2 s apart.
- **Locale:** the store picks a country from your IP (it sent us to `en_CA`, in CAD). Typing `/en_US/` into the URL doesn't stick. Use the flag menu at the top left, set **Ship to: United States**, then Save. After that, `/en_US/...` pages show USD. Prices include duties ("Duties & VAT included"). Set the menu back to your own country afterwards.
- **Finding the jeans:** `/en_US/jeans/` is a curated landing page, not the full list. Search catches everything: `/en_US/search/?q=straight&sz=200`, then `tapered`, `wide`, `slim` and `womens`. Each tile is `.product[data-pid]`, and its `data-gtm` JSON has the name. The fit categories (`/jeans/jeans-fit/...`) miss some products, for example the #300 Cashmere.
- **The chart:** static HTML in the product page, `table.sizechart` inside `div.sizechart-container.unit-cm.method-flat`, so a fetch of the page has it. Each page has one table, shared by every colour. The default is CM. The INCH button converts in the browser, so record cm.
  - The columns are `hem, waist, rise, thigh, inseam, hip`. **Their order and case differ between products**, so map them by header text: hem → `leg_opening`, rise → `front_rise`, hip → `seat`.
  - There's no knee and no back rise.
- **Fabric:** two blocks on the page.
  - `#product-description` is the spec list: fit, "Fabric: Cotton 100%", "Rigid denim (unwashed)" or "One wash (washed)", "No anti-shrink processing".
  - `#product-accordion-detail` is the prose, with the weight and "FABRIC — INDIGO / FABRIC — BLACK" sections.
  - The colours are the `[data-attr="color"] [aria-label]` buttons ("Select Color Black").

## How Momotaro measures

From the [sizing guide](https://www.momotaro-jeans.com/en_US/sizing-guide.html):
- **Waist:** "measure around the full upper edge of the waist belt", so `around`.
- **Front rise:** from the crotch seam along the fly to the top of the waistband. The chart's single "rise" is this.
- **Thigh:** "the fullest part of your thigh from the inside edge to the outside edge", so `flat`. The photo puts the tape straight across from the crotch seam, so `at-crotch`.
- **Hem:** "side to side", so `flat`.
- **Hip:** not in the guide at all. The values are circumferences (size 32 = 110 cm), so `around`, and the point is `unknown`.
- **"method-flat":** the container's class says this, but it is wrong for waist and hip. Size 32 has waist 87 and hip 110, which only make sense all the way around. Thigh 34.5 and hem 22 only make sense flat.
- **Tolerance:** the guide says measurements may vary by "up to 2~3cm (1 inch)". We record the brand's single figure, 1 in.
- **State:** no chart says which state it shows, so every chart is `as-sold`.

## Fabric decisions

- **Standard label:** "Rigid (unwashed)", so `sold: "raw"`. It says "a special finish keeps shrinkage after washing to a minimum", but never calls the denim sanforized, so `sanforized: null`.
- **Classic label:** "One wash (washed)" and "No anti-shrink processing", so `sold: "one-washed"` and `sanforized: false`. Its inseams are 2.5 cm longer than Standard's.
- **Silk and cashmere (Exclusive label):** "Rigid denim (unwashed)", with the composition from the spec: cotton 58 / silk 42 and cotton 65 / cashmere 35.
  - Cashmere is "dry clean only", and "washing in water may cause significant shrinkage".
- **Dyed specials** (Shikon, Black Sesame, Kyotomontsuki, BC, Moss, Onigurumi): they're product-dyed or garment-dyed. No page says raw or washed, so `sold: null`.
  - Most don't state a weight either. Onigurumi says "15oz".
- **High Fade Blue:** bleached, with no weight and no wash state stated, so `null`.
- **#100/#200/#400 11oz (men's):** they don't say rigid or washed, so `sold: null`. The women's 11oz pages say "Rigid denim (unwashed)".
- **Shrinkage:** no product gives a figure, so it's `null` everywhere.
- **Price:** each page has one price for every colour and size.

## Left out, and why

- **#100 Straight - Kyotomontsuki Dyed, `MMJE6401` ($369.10):** the whole product is skipped. Its chart is the #400 Wide chart: identical to `MMJE6500` and `M400ST`, with hem 23.3–26.9 and hip 102.7–132.7. Yet the page describes a #100 straight. The other #100 Kyotomontsuki page, `MMJB6404`, has a normal #100 chart and is included.
- **Numbers left out of otherwise good charts:**
  - #100 BC (`MMJZ6402`) and #100 Moss (`MMJZ6403`), size 34: the hip is published as "15", so `seat` is left out.
  - #400 11oz (`MXJE1506`), size 36: hem 27.7 sits between 25.1 and 26.3, so `leg_opening` is left out.
  - #300 Silk (`M300SL`), sizes 40 and 42: rise 28.2 and 29 are smaller than every smaller size's (30.3 at size 38), so `front_rise` is left out.
    - The rest of those two rows is in order but irregular: waist 106/111 where the other #300s have 104/109, hem 19.4/20, hip 121.3/126.3. It's kept as published. Check it next run.
  - #100 11oz (`MXJE1405`), sizes 30–38: hems 24.4–26.8 jump 3.6 cm from size 28's 20.8. They equal the #100's grading for sizes 40 and up, so `leg_opening` is left out. Size 28's 20.8 matches the other #100s and is kept.
  - #400 Standard (`M400ST`), size 42: the hem cell is empty.
- **Sizes for sale with no chart row** (no numbers, so left out):
  - #400 Standard: 24 and 25
  - #100 11oz: 26
  - #400 11oz: 26
- **Chart rows for sizes not for sale:** #200 Onigurumi's chart has size 40, but the page doesn't sell it, so it's left out.
- **Kept as published, but odd:**
  - #400 Standard size 28: hip 102.2 (the other #400s say 102.7).
  - #400 Kyotomontsuki (`MMJE6500`) size 33: thigh 36.6 (#400 Standard says 36.3).
  - #100 Shikon: every waist is 1 cm under the other #100s, and the inseams up to size 36 are 1 cm shorter. It's consistent, so it may be real garment-dye shrinkage.
- **Not jeans, so not in the catalogue:**
  - #100 Straight in corduroy, smooth leather and suede leather
  - Denim Trousers, denim shorts, sashiko pants and other trousers

## Sold-out products and lines

- **Sold out:** search results include sold-out products, which carry an "Out of Stock" badge (`.pd-badge--no-stock` without `d-none`). So the search sweep above already finds them. About half of the jeans were sold out on 2026-11-21, and all of them are in the catalogue as `current`.
  - `sitemap_0.xml` lists no product pages, so it's no help in finding products.
  - A page lists every size the product comes in, whether or not it's in stock.
- **Line:** Momotaro has one gender category, `/jeans/jeans-womens-jeans/` ("WOMENS JEANS"). #110, #410, Super Wide Womens and Flare Womens are filed there, so their `line` is `"women"`. Every other cut sits under plain "JEANS" with no men's or unisex label, so its `line` is `null`.
