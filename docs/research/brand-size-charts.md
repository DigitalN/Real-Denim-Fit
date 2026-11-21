# What brands publish in their size charts

Everything here was checked on 20 November 2026 on each brand's own site, unless a note says otherwise. It's background for `/update-catalogue` and for the catalogue's data model. Check it again before you rely on it: sites change.

## Naked & Famous (nakedandfamousdenim.com, Shopify)

- **What they publish:** waist, front rise, back rise, upper thigh, knee, leg opening and inseam, for tag sizes 28–44.
  - Example: [Weird Guy All Natural Ecru Foxfiber](https://nakedandfamousdenim.com/products/weird-guy-all-natural-ecru-foxfiber), size 32: waist 34, front rise 12, back rise 14.75, thigh 12.5, knee 8.75, leg opening 7.5, inseam 34 (inches).
- **How they measure** ([measuring guide](https://nakedandfamousdenim.com/pages/copy-of-measuring-guide)): inches, with a cm toggle, and a tolerance of ±0.5″.
  - The waist is a full circumference (measured flat, then doubled).
  - Everything else is flat.
  - The thigh is measured straight across from the end of the crotch seam.
  - The knee is measured 12″ down the inseam from the crotch.
- **State:** sanforized unless noted, with 1–3% shrinkage. Loomstate shrinks up to 10%.
  - Unsanforized products (Prince of Slub, Hedorah, Cat's Pajamas) have a pre-wash chart and a post-wash chart. The site labels the post-wash one "Estimated after the first wash".
  - Hedorah size 32: waist 35 pre-wash, 32 post-wash.
- **Charts belong to the product, not the cut.** A size 32 Weird Guy has a waist of 34, 34.75, 35 or 35.5 depending on the fabric, and its inseam ranges from 33.25 to 36.5.
- **Price and fabric:** prices are in USD (a Canada locale shows CAD). Stretch fabrics, such as Blue Haze with 2% elastane, are listed under the same cuts as rigid ones.
- **Their advice:** "Do not size down automatically". Compare against a pair that fits.
- **How to extract it:** the chart is HTML-escaped JSON in `data-columns` / `data-rows` attributes on the product page; it isn't in `products.json`. The format drifts between products:
  - Some have unit `null` and a "SIZE / PRE WASH" header, with post-wash rows marked `*` in the same table.
  - Others use `data-post-rows`.
  - Newer ones add `"convention": "circumference" | "flat" | "length"`.
  - Labels differ in case, and some cells are null.

## Iron Heart (ironheart.co.uk; ironheartamerica.com redirects there; Shopify)

- **What they publish:** waist, front rise, rear rise, thigh, knee, hem and inseam, with separate inch and cm tables.
  - Example: [634S](https://ironheart.co.uk/products/ih-634s), tag 32: waist 31.7, front rise 11.0, rear rise 14.8, thigh 12.5, knee 9.3, hem 8.7, inseam 35.5.
- **How they measure** ([how we measure](https://www.ironheart.co.uk/pages/how-we-measure-bottoms)):
  - The knee is 13″ below the crotch seam.
  - The thigh is straight across at the crotch; this is only shown in a photo.
  - The waist is a full circumference. The page doesn't say so; it's inferred from the values.
- **State:** most products are "Sanforized & One-Washed", with no further shrinkage, so the charts describe them as sold.
  - The 634S-UHR is "Unsanforized & Unwashed". Expect about 1″ off the waist and up to 2″ off the inseam.
  - Its chart is bigger (tag 32 waist 33.4) and doesn't say which state it shows; presumably as sold.
- **Charts belong to the product.** The tag-32 waist across 634 fabrics is 31.7, 32.0, 33.0 or 33.4.
- **Price and fabric:** prices are in GBP (USD/EUR by location). No elastane in any of their jeans.
- **Their advice:** the waist stretches about 1″.
- **How to extract it:** the easiest of the six. It's a static `<table>` in `body_html`, so `products.json` has it, between `[SIZE-GUIDE-INCH]` and `[SIZE-GUIDE-CM]` markers.
  - 6 of 123 jeans have no chart.
  - The fabric's state is in a "Fabric Focus" block that only appears on the rendered page.

## Nudie Jeans (nudiejeans.com, Centra/Next.js)

- **What they publish:** only through **Virtusize**, a third-party widget that loads after you accept cookies. The research run declined cookies, so it **never saw the numbers**. Units, measuring points and whether charts are per product are all unverified.
- **How they measure:** the guide is an image. Everything is flat:
  - waist side to side
  - rises including the waistband
  - hip about 16 cm below the waistband
  - thigh at the crotch
  - knee at half the inseam
  - leg opening
  - inseam
- **State:** everything is sanforized, shrinking 5% at most (the care label says about 3%). Some products give their own figures, for example Lean Dean Dry 16 Dips: "inseam shrinkage of 2% and 4.5-6% in width".
- **Sizes:** a waist × length grid (W24–38, L28–36).
- **Price and fabric:**
  - Prices depend on the locale (en-US USD 260, en-CA CAD 250, en-SE SEK 1600).
  - All five Lean Dean dry fabrics checked have 1–2% elastane.
  - Gritty Jackson and Regular Alf are 100% cotton.
- **Their advice:** between sizes, choose the larger.

## 3sixteen

- **What they publish:** an HTML table in a "Measurements" modal, added by JavaScript.
  - Example: [CS-105x](https://www.3sixteen.com/products/cs-105x-classic-straight-indigo-selvedge), size 32: waist 33, front rise 12.25, back rise 16.25, inseam 36, thigh 12.25, knee 9.5, opening 8.5.
- **How they measure** ([measuring guide](https://www.3sixteen.com/pages/measuring-guide-jeans)):
  - The waist is doubled.
  - The thigh is 1″ below the crotch.
  - The knee is 14″ below the crotch.
- **State:** sanforized, with minimal shrinkage. The [FAQ](https://www.3sixteen.com/pages/faq) says the waist stretches up to 1″ in two weeks.
- **Price:** USD 270.

## Rogue Territory

- **What they publish:** **a JPEG image** per product. Measurements are flat, in inches, ±0.25″. Example: Stanton tag 32 has a 16.75″ flat waist.
- **How they measure** ([sizing guide](https://www.rogueterritory.com/pages/sizing-guide)):
  - The thigh is 1″ below the crotch.
  - The knee is 13″ below the crotch.
- **Their advice:** add 0.5″ to the inseam for shrinkage.

## Momotaro

- **What they publish:** static HTML in a modal, in cm by default. The columns are hem, waist, one rise, thigh, inseam and hip. There's **no knee and no back rise**.
- **How they measure:** the container is labelled "method-flat", but the waist and hip values are circumferences. Size 32: waist 87, hip 110, thigh 34.5, hem 22.
- **State:** "Rigid (unwashed)"; the chart doesn't say which state it describes.
- **Access:** the official site returned 403 to the second research run; the first one could read it.

## Where each brand measures

| | At the crotch | 1″ below | 12″ below | 13″ below | 14″ below | Half the inseam |
|---|---|---|---|---|---|---|
| Thigh | N&F, Iron Heart, Nudie | 3sixteen, Rogue | | | | |
| Knee | | | N&F | Iron Heart, Rogue | 3sixteen | Nudie |

## None of them publishes a body size chart

None of the six says what body each size is cut for. Every one says to compare the chart with a pair that fits. Tag size isn't the garment's waist either: the Weird Guy Stretch in tag 32 has a 34″ garment waist, according to the Tate + Yoko retailer chart.
