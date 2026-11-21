# Iron Heart: extraction notes

Last full run: 2026-11-20. 110 products in 6 cuts (555, 634, 666, 777, 888, 1955), 1,408 sizes. 21 of the products were sold out on that day.

## Where things are

- **Product list:** `https://www.ironheart.co.uk/products.json?limit=250&page=N` (6 pages, about 1,400 items). Prices there are GBP.
- **Charts:** in each product's `body_html`, as a static `<table>` after the `[SIZE-GUIDE-INCH]` marker, followed by a cm table after `[SIZE-GUIDE-CM]`. Record the inch table, except for the products listed under "Inch and cm tables" below. Rows are Waist, Front Rise, Rear Rise, Thigh, Knee, Hem, Inseam, which map to `waist`, `front_rise`, `back_rise`, `thigh`, `knee`, `leg_opening`, `inseam`.
- **Rendered page** (`/products/<handle>`): needed for three things that aren't in `products.json`:
  - the **cut**: a heading such as `634 Straight Cut` in the description, followed by bullet points
  - the **fabric**: a heading such as `21oz Japanese Selvedge Denim`, then a bullet such as `- 100% cotton`
  - the **Fabric Focus** block (see below)

  The description starts at a line reading "Product Description" or "Product Details" (both occur). On 634S-SB the fabric heading comes before the cut heading.
- **Spot-checking:** the rendered page carries the same chart as plain text (`[SIZE-GUIDE-INCH]\nTag Size\n28\n29…`), which JavaScript turns into the table you see.

## Which products are jeans

- Product type `Bottoms` (a few are also tagged OUTLET), with SKUs starting `IH-555`, `IH-634`, `IH-666`, `IH-777`, `IH-888`, `IH-1955` or `IH-684`, plus two collaborations sold on the site: `bs69xsop420` (777 cut) and `ihxsjf-634` (634 cut).
- The cut comes from the rendered page's cut heading. The brand calls its cuts "The 555 Cut", "The 634 Cut" and so on ([our core cuts](https://www.ironheart.co.uk/pages/our-core-cuts)), so cut ids are the bare numbers. SKUs with and without the `S` (for example `IH-634S` and `IH-634-XHS`) carry the same cut heading and bullets, so they're in the same cut file.
- **Sold-out products are included** with `"status": "current"`. `products.json` still lists them, with every variant `"available": false`; the catalogue never records stock.
- **`line` is `null` on every cut:** the site doesn't file its jeans as men's, women's or unisex. The menu goes straight from BOTTOMS to the cuts, and `/collections.json` has no gendered collections.
- **Left out because they aren't denim:** the 17oz Duck Canvas jeans (`ih-555d`, `ih-634d`, `ih-666d`, `ih-777d`, `ih-888d`). Their fabric heading reads "17oz Japanese Duck Canvas".
- **Left out because they aren't jeans:**
  - chinos (717, 721, 727, 730, 731), work pants, painter's pants (including the 12oz Wabash 814) and cargo pants (including the denim cargo `ihdr-502bb`)
  - overalls (805, 812, 819), double-knee engineer pants (822), shorts and sweatpants
  - "Hall of Fades" and "RELOVED Clothing", which are worn, pre-owned pairs rather than products

## Fabric Focus → `sanforized`, `sold`, `shrinkage`

| Fabric Focus heading | `sanforized` | `sold` | `finish` | `shrinkage` |
|---|---|---|---|---|
| Sanforized & One-Washed | true | one-washed | rinsed | null ("no further shrinkage") |
| Sanforized, One-Washed & Overdyed | true | one-washed | rinsed | null |
| Unsanforized but One-Washed (19oz LHT) | false | one-washed | rinsed | null (the brand expects little to no further shrinkage) |
| Sanforized but Unwashed (25oz XHS) | true | raw | raw | 1″ waist, 1″ inseam |
| Unsanforized & Unwashed (21/23oz UHR) | false | raw | raw | 1″ waist, up to 2″ inseam (recorded as 2) |

None of the charts says which state it shows, so all are `"as-sold"`. The UHR fabric is 21oz raw and about 23oz after washing; `weight_oz` is 21.

**`finish`:** no Iron Heart jean is faded at the factory, so none is `"washed"`. The one-wash is an industrial wash "to further reduce the risk of shrinkage", so it's `"rinsed"`. On this run that gave 95 rinsed and 15 raw. Calls that a name might suggest otherwise:
- **Overdyed products** (22) are `"rinsed"`. They're one-washed, then dyed black after construction. That adds colour rather than fading it. The sulphur overdye does age the metal hardware, but the denim isn't faded. *This is the most doubtful call:* switch them to `"washed"` if overdyeing should count as a factory treatment.
- **"18oz Vintage Selvedge"**, **"Superblack Non-Fade"**, **"Superblack (Fades To Grey)"** and **"13.5oz White"** are fabric names. Their pages say "Sanforized & One-Washed", with no fading treatment. The White's "pure white bleached warp and weft yarns" describes the yarn, not a faded garment. All are `"rinsed"`.

## Measuring method

- From [how we measure bottoms](https://www.ironheart.co.uk/pages/how-we-measure-bottoms), which is mostly photos.
- Knee: 13″ down from the crotch seam (stated in text).
- Thigh: straight across at the crotch (photo only).
- Waist: the photo shows it measured flat across the waistband, but the published numbers are about the tag size (tag 32 → 31.7), so they're the full circumference: `"around"`.
- No tolerance is stated anywhere, so `tolerance` is `null`.

## Quirks

- **Prices:** outlet items (13.5oz White, 14oz Broken Twill, SJF collab, 634S-14ii) have a `compare_at_price`. That's the list price and is what's recorded; `price` is the sale price and is ignored ([ADR 0003](../../docs/adr/0003-no-shopping-information.md)).
- **Chart sizes vs sold sizes:** these sometimes differ (for example, the chart lists 28–40 but the shop sells up to 44). Record the chart's sizes only.
- **Duplicate titles:** two pairs of listings share a title but have different charts: `ih-634s-142ib` / `ih-634s-142ib-c` and `ih-666s-142ib` / `ih-666s-142ib-c`. Both of each pair are kept; the id tells them apart.
- **Stretch:** the 21oz Shinayaka (`ih-555-sst`, `ih-555-sst-blk`) is "98% cotton / 2% polyurethane". The fibre is recorded as `polyurethane`, as published; it's an elastane-type stretch fibre.
- **Typo kept as published:** one product title says "Overyded".
- One chart's corner cell is blank instead of "Tag Size". It's harmless.

## Inch and cm tables

The script compares every inch cell with its cm cell (1 in = 2.54 cm) and prints any pair more than 0.2″ apart. On 2026-11-21 two products disagreed:
- **`ih-666s-uhr`, size 35 waist:** the inch table says 37.8, which is larger than size 36's 37.1. The cm table says 92.5 (36.4″), which fits between its neighbours, and every other cell in the two tables agrees. **This product's chart is recorded from the cm table** (`"unit": "cm"`), via `USE_CM` in the script.
- **`ih-777s-142ib`, size 34 thigh:** this time it's the cm table that's off. 30.9 cm (12.2″) dips below size 33, while the inch table's 12.7 fits. The inch table is kept.

Recheck these on the next run. If a pair disagrees and neither table fits, leave that one value out and note it here.

## How to rerun

```sh
python3 tools/catalogue/iron-heart/extract.py
```

- It fetches everything again and rewrites `brand.json` and the cut files. Then it lists what it skipped, any blank cells and any inch/cm disagreements. See [`tools/catalogue/iron-heart/README.md`](../../tools/catalogue/iron-heart/README.md) for the options.
- About 130 requests, taking roughly 5 minutes. It uses only Python's standard library and doesn't need a browser.
- Afterwards, review `git diff catalogue/iron-heart`, and update this file and the "Last full run" date.
- If it exits with a PROBLEM (for example, a new Fabric Focus heading), fix the script, not the output.

## Skipped (no chart)

All 10 are tagged `future-release` and have no size chart yet (pages without a chart stay out):
- `ih-634s-19lod`, `ih-666s-19lod`, `ih-777s-19lod`, `ih-888s-19lod` (19oz LHT overdyed)
- `ih-684s-slb`, `ih-684s-18`, `ih-684s-142bb` (the whole 684 Relaxed Straight cut)
- `ih-1955s-sb`, `ih-1955s-142bb`, `ih-1955s-18`

Check them again on the next run.
