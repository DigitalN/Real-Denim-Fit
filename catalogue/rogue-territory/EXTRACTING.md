# Rogue Territory: extraction notes

Last full run: 2026-11-20. 30 products in 7 cuts, 294 sizes. None was sold out on that day. **Every chart is read by eye from a JPEG** (`from_image: true`), so each number needs checking against its picture (list below).

## Where things are

- **Product list:** `https://www.rogueterritory.com/products.json?limit=250` (one page). Prices are USD.
- **Which products are jeans:** those tagged `DENIM` whose fabric is denim. The title is `<Cut> // <Fabric>`, for example `Stanton // 14.5oz Indigo`. Trousers, fatigue pants, officer trousers and shorts aren't tagged `DENIM`.
- **Left out because they aren't denim**, though tagged `DENIM` and cut like jeans:
  - `relaxed-strider-brown-neppy-herringbone` and `relaxed-strider-black-neppy-herringbone` (12oz herringbone twill)
  - `relaxed-strider-mock-twist-selvedge-canvas` (14oz canvas)
  - `double-knee-strider-black-canvas` and `double-knee-strider-camel-canvas` (12oz canvas)
- **Sold-out products are included** with `"status": "current"`. `products.json` still lists them, with every variant `"available": false`; the catalogue never records stock. None was sold out on this run.
- **`line` is `null` on every cut:** the site doesn't file its jeans as men's, women's or unisex. The menu goes DENIM → fit, and `/collections.json` has no gendered collections.
- **Fit and fabric:** in `body_html`:
  - `FIT //` gives the brand label, for example "Slim Straight, Mid-Rise"
  - `DENIM //` gives weight, "100% cotton" and "sanforized"
  - the line "Measurements (in inches) taken flat, +/- 0.25″ tolerance…" is on every product page, which is where `tolerance` comes from
- **Charts:** one `<img>` in `body_html` right after that line, served from `cdn.shopify.com/s/files/1/2633/5944/files/…`. Most are only 480 px wide. Enlarge them before reading (2.5× works), and zoom into rows you're unsure of.
  - The Stealth Strong Taper page links a `_480x480` rendition. Drop the suffix (`15ozStealthStrongTapersizing.jpg`) to get the 2626 px original, which reads much more easily; both agree.
- **Columns:** TAG SIZE, WAIST, FRONT RISE, BACK RISE, UPPER THIGH, KNEE, LEG OPENING, INSEAM, which map to `label`/`waist_tag`, `waist`, `front_rise`, `back_rise`, `thigh`, `knee`, `leg_opening`, `inseam`.

## Measuring method

- From the [sizing guide](https://www.rogueterritory.com/pages/sizing-guide):
  - The waist is measured straight across, so it's `"flat"` like everything else (tag 32 ≈ 16.75).
  - The upper thigh is 1″ down from the crotch.
  - The knee is 13″ down from the crotch.
- Tolerance is ±0.25″, as stated on each product page.

## Fabric decisions

- **`sanforized`:** taken from the `DENIM //` line where it says "sanforized". Otherwise it's `true` from the [FAQ](https://www.rogueterritory.com/pages/faq): "Our denim is sanforized (pre-shrunk) unless otherwise noted". That applies to stanton-11oz-stealth, strider-11oz-stealth, strider-13-5oz-indigo-x-black, strider-stealth-wash, and the four River and Vintage Wash products.
- **`sold`:** River Wash, Vintage Wash, Stealth Wash and Rinsed are `"one-washed"`; the rest are `"raw"`.
- **`shrinkage`:** always `null`. The FAQ's advice to add 0.5″ to the inseam when hemming raw denim is general advice, not a product figure.
- **Prices:** sale items have a `compare_at_price`, which is the list price and what's recorded. Variants with `compare_at_price` "0.00" are ignored.

## Numbers to look at twice

- **Stealth Standard Issue (`standard-issue-15oz-stealth`), size 28:** the waist is printed as **"5"**, an obvious typo. It's left out rather than guessed.
- **Stanton 11oz (Indigo and Stealth share one image), size 29:** knee 7.5, larger than size 30's 7.25, and unlike every other size, where knee equals leg opening. Recorded as printed; probably a typo on the brand's side.
- **Stealth Strong Taper, size 36:** front rise jumps from 11.5 to 12.25, then 12.5 at 38. Recorded as printed (checked on the full-size original).
- **Strider Vintage Wash, sizes 36/38/40:** waist 18 → 18.5 → 19.5 (the River Wash has 18.25 → 19 → 20). Recorded as printed.
- **Shared images:** the Stanton 11oz Indigo and Stealth share one image, as do the Strider 11oz Indigo and Stealth. The Stanton River and Vintage Wash use two uploads of a byte-identical image.
- Chart sizes don't always match the sizes sold (for example, the SK chart has 26 and no 35). Only the chart's sizes are recorded.

## Every chart image

Check each product's numbers against its image.

| Cut | Product | Chart image |
|---|---|---|
| Relaxed Strider | [Relaxed Strider // 15oz Indigo Redline](https://www.rogueterritory.com/products/relaxed-strider-15oz-indigo-redline) | [Relaxed_Strider_15oz_Indigo_Redline_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Relaxed_Strider_15oz_Indigo_Redline_sizing.jpg?v=1793221978) |
| Relaxed Strider | [Relaxed Strider // 17oz Cryptic Indigo x Black](https://www.rogueterritory.com/products/relaxed-strider-17oz-cryptic-indigo-x-black) | [Relaxed_Strider_17oz_Cryptic_Ind_Blk_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Relaxed_Strider_17oz_Cryptic_Ind_Blk_sizing.jpg?v=1776284972) |
| Relaxed Strider | [Relaxed Strider // 17oz Cryptic Stealth](https://www.rogueterritory.com/products/relaxed-strider-17oz-cryptic-stealth) | [Relaxed_Strider_Cryptic_Stealth_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Relaxed_Strider_Cryptic_Stealth_sizing.jpg?v=1773329812) |
| SK | [SK // 15oz Indigo](https://www.rogueterritory.com/products/sk-15-oz) | [SK_15oz_Sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/SK_15oz_Sizing.jpg?v=1513927829) |
| SK | [SK // 15oz Stealth](https://www.rogueterritory.com/products/stealth-sk) | [SK_Stealth_Sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/SK_Stealth_Sizing.jpg?v=1513927899) |
| Silveridge | [Silveridge // 15oz Indigo](https://www.rogueterritory.com/products/silveridge-15oz-indigo) | [Indigo_Silveridge_sizing_082825.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Indigo_Silveridge_sizing_082825.jpg?v=1756406295) |
| Silveridge | [Silveridge // 15oz Stealth](https://www.rogueterritory.com/products/stealth-silveridge-15oz) | [Stealth_Silveridge_sizing_092925.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Stealth_Silveridge_sizing_092925.jpg?v=1759180360) |
| Standard Issue | [Standard Issue // 15oz Indigo](https://www.rogueterritory.com/products/standard-issue-15oz) | [15oz_Prop_Std_Issue_sizing_012626.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/15oz_Prop_Std_Issue_sizing_012626.jpg?v=1769467898) |
| Standard Issue | [Standard Issue // 15oz Indigo Kurabo](https://www.rogueterritory.com/products/standard-issue-15oz-indigo-kurabo) | [StdIssue15ozIndKurabosizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/StdIssue15ozIndKurabosizing.jpg?v=1771886467) |
| Standard Issue | [Standard Issue // 15oz Stealth](https://www.rogueterritory.com/products/stealth-standard-issue) | [Stealth_Std_Issue_sizing_092925.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Stealth_Std_Issue_sizing_092925.jpg?v=1759176781) |
| Stanton | [Stanton // 11oz Indigo](https://www.rogueterritory.com/products/stanton-11oz) | [11oz_denim_Stanton_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/11oz_denim_Stanton_sizing.jpg?v=1781907301) |
| Stanton | [Stanton // 11oz Stealth](https://www.rogueterritory.com/products/stealth-stanton-11oz) | [11oz_denim_Stanton_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/11oz_denim_Stanton_sizing.jpg?v=1781907301) |
| Stanton | [Stanton // 14.5oz Indigo](https://www.rogueterritory.com/products/stanton) | [Stanton_14.5oz_sizing_042426.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Stanton_14.5oz_sizing_042426.jpg?v=1777063758) |
| Stanton | [Stanton // 15oz Indigo Redline](https://www.rogueterritory.com/products/stanton-15oz-indigo-redline) | [15oz_Stanton_Redline_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/15oz_Stanton_Redline_sizing.jpg?v=1795221048) |
| Stanton | [Stanton // 15oz Stealth](https://www.rogueterritory.com/products/stealth-stanton) | [Stealth_Stanton_sizing_022426.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Stealth_Stanton_sizing_022426.jpg?v=1771951849) |
| Stanton | [Stanton // 17oz Cryptic Indigo](https://www.rogueterritory.com/products/stanton-17oz-cryptic-indigo) | [Stanton_17oz_Cryptic_Indigo_sizing_041526.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Stanton_17oz_Cryptic_Indigo_sizing_041526.jpg?v=1776285376) |
| Stanton | [Stanton // 22oz Indigo](https://www.rogueterritory.com/products/stanton-22oz-indigo-slub) | [22oz_Indigo_Stanton_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/22oz_Indigo_Stanton_sizing.jpg?v=1760656963) |
| Stanton | [Stanton // Indigo River Wash](https://www.rogueterritory.com/products/stanton-indigo-river-wash) | [Washed_Stantons_sizing_012926_40fb4eff-b7ca-467c-8bf2-1a451f152e56.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Washed_Stantons_sizing_012926_40fb4eff-b7ca-467c-8bf2-1a451f152e56.jpg?v=1769719134) |
| Stanton | [Stanton // Indigo Vintage Wash](https://www.rogueterritory.com/products/stanton-indigo-vintage-wash) | [Washed_Stantons_sizing_012926.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Washed_Stantons_sizing_012926.jpg?v=1769718938) |
| Strider | [Strider // 11oz Indigo](https://www.rogueterritory.com/products/strider-11oz-indigo) | [11oz_denim_Striders_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/11oz_denim_Striders_sizing.jpg?v=1781906795) |
| Strider | [Strider // 11oz Stealth](https://www.rogueterritory.com/products/strider-11oz-stealth) | [11oz_denim_Striders_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/11oz_denim_Striders_sizing.jpg?v=1781906795) |
| Strider | [Strider // 13.5oz Indigo x Black](https://www.rogueterritory.com/products/strider-14-5oz-indigo-x-black) | [Strider_13.5oz_Ind_Blk_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Strider_13.5oz_Ind_Blk_sizing.jpg?v=1734632911) |
| Strider | [Strider // 15oz Rinsed Indigo](https://www.rogueterritory.com/products/strider-15oz-rinsed-indigo) | [Rinsed_Indigo_Strider_sizing_092525.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Rinsed_Indigo_Strider_sizing_092525.jpg?v=1758833594) |
| Strider | [Strider // 17oz Cryptic Indigo](https://www.rogueterritory.com/products/strider-17oz-cryptic-indigo) | [Strider_17oz_Cryptic_Indigo_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Strider_17oz_Cryptic_Indigo_sizing.jpg?v=1776284236) |
| Strider | [Strider // 22oz Indigo](https://www.rogueterritory.com/products/strider-22oz-indigo-slub) | [22oz_Indigo_Strider_sizing.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/22oz_Indigo_Strider_sizing.jpg?v=1760656590) |
| Strider | [Strider // Indigo River Wash](https://www.rogueterritory.com/products/strider-indigo-river-wash) | [Strider_River_Wash_sizing_060426.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Strider_River_Wash_sizing_060426.jpg?v=1780615914) |
| Strider | [Strider // Indigo Vintage Wash](https://www.rogueterritory.com/products/strider-indigo-vintage-wash) | [Strider_Vintage_Wash_sizing_060426.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Strider_Vintage_Wash_sizing_060426.jpg?v=1780616415) |
| Strider | [Strider // Stealth Wash](https://www.rogueterritory.com/products/strider-stealth-wash) | [Stealth_Wash_Strider_sizing_050726.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/Stealth_Wash_Strider_sizing_050726.jpg?v=1778169311) |
| Strong Taper | [Strong Taper // 15oz Indigo](https://www.rogueterritory.com/products/strong-taper-15oz) | [15oz_Strong_Taper_Indigo_sizing_022525.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/15oz_Strong_Taper_Indigo_sizing_022525.jpg?v=1740593501) |
| Strong Taper | [Strong Taper // 15oz Stealth](https://www.rogueterritory.com/products/stealth-strong-taper-15oz) | [15ozStealthStrongTapersizing_480x480.jpg](https://cdn.shopify.com/s/files/1/2633/5944/files/15ozStealthStrongTapersizing_480x480.jpg?v=1695080621) |
