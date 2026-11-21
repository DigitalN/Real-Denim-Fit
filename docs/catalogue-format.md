# Catalogue format

The catalogue is plain JSON files under `catalogue/`, one folder per **brand**. People and Claude write it (via `/update-catalogue`), every change is reviewed in a PR, and the app builds it in. Terms follow [GLOSSARY.md](../GLOSSARY.md).

```
catalogue/
├── exchange-rates.json
└── iron-heart/
    ├── brand.json
    ├── EXTRACTING.md
    ├── 634s.json          one file per cut, holding all its products
    └── 555s.json
```

## What goes in

- **Every denim jeans product whose page is on the brand's site**, whether or not it's sold out. The app never shows stock, and a sold-out pair can still be someone's reference pair or a secondhand find.
- **Men's, women's and unisex cuts alike**, each with its `line`.
- **Denim only.** Leave out duck canvas, herringbone, corduroy, linen, twill trousers, shorts, and worn or pre-owned pairs.

## Rules

- **Record numbers exactly as the brand publishes them**, in its unit and convention, with no converting or rounding. The app converts. That way a reviewer can check every number against the source page.
- **Never guess.** If a measurement isn't published, leave its key out: never `null` or `0`. If something about the fabric isn't stated (sanforized or not, composition), use `null`.
- **Leave out a product that has no chart at all**, and say in the brand's `EXTRACTING.md` that it was skipped.
- **If a brand publishes both inches and cm, record one of them**: the brand's primary unit, or inches when it has none.
- **A chart whose state the brand doesn't name is `"as-sold"`.**
- **Ids** are lowercase kebab-case. **Dates** are ISO (`2026-11-20`).
- **Charts inherit their measuring points, conventions and tolerance from `brand.json`.** Override them on a chart only when that chart differs.

## Measurement keys

`waist`, `front_rise`, `back_rise`, `seat`, `thigh`, `knee`, `leg_opening`, `inseam`.

**Conventions** say whether a width was measured flat or all the way around:
- They apply to `waist`, `seat`, `thigh`, `knee` and `leg_opening`, and are either `"flat"` or `"around"`.
- They don't apply to `front_rise`, `back_rise` and `inseam`, which are lengths.

**Measuring points**:
- `thigh`: `"at-crotch"` or `"1in-below-crotch"`
- `knee`: `"12in-below-crotch"`, `"13in-below-crotch"`, `"14in-below-crotch"` or `"half-inseam"`
- `seat`: `"fullest"` or `"16cm-below-waistband"`

Any of them can also be `"unknown"`, which means the brand doesn't say where it measures, and the app marks its matches "approximate".

## `brand.json`

```json
{
  "id": "iron-heart",
  "name": "Iron Heart",
  "site": "https://www.ironheart.co.uk",
  "currency": "GBP",
  "sizing": "waist",
  "tolerance": { "value": 0.5, "unit": "in" },
  "conventions": { "waist": "around", "seat": "flat", "thigh": "flat", "knee": "flat", "leg_opening": "flat" },
  "measuring_points": { "thigh": "at-crotch", "knee": "13in-below-crotch", "seat": "unknown" },
  "method_source": { "url": "https://www.ironheart.co.uk/pages/how-we-measure-bottoms", "seen": "2026-11-20" }
}
```

- `sizing` is `"waist"`, or `"waist-and-length"` for brands like Nudie that sell W×L.
- `tolerance` is `null` when the brand doesn't state one.

## A cut file, such as `634s.json`

```json
{
  "id": "634s",
  "brand": "iron-heart",
  "name": "634S",
  "brand_label": "Slim Straight",
  "line": "men",
  "products": [
    {
      "id": "634s-21oz-indigo",
      "name": "21oz Selvedge Denim, Indigo",
      "url": "https://ironheart.co.uk/products/ih-634s",
      "status": "current",
      "status_seen": "2026-11-20",
      "fabric": {
        "weight_oz": 21,
        "composition": [{ "fibre": "cotton", "percent": 100 }],
        "sanforized": true
      },
      "sold": "one-washed",
      "list_price": { "amount": 293, "currency": "GBP", "url": "https://ironheart.co.uk/products/ih-634s", "seen": "2026-11-20" },
      "shrinkage": null,
      "charts": [
        {
          "state": "as-sold",
          "source": { "url": "https://ironheart.co.uk/products/ih-634s", "seen": "2026-11-20" },
          "unit": "in",
          "from_image": false,
          "sizes": [
            { "label": "32", "waist_tag": 32,
              "measurements": { "waist": 31.7, "front_rise": 11.0, "back_rise": 14.8, "thigh": 12.5, "knee": 9.3, "leg_opening": 8.7, "inseam": 35.5 } }
          ]
        }
      ]
    }
  ]
}
```

**On the cut:**
- `brand_label` is the brand's own name for the fit. It's shown in the app but never used for matching ([ADR 0008](adr/0008-leg-shape-and-rise-from-the-numbers.md)). Use `null` if there isn't one.
- `line` is `"men"`, `"women"` or `"unisex"`, as the brand files the cut. Use `null` if the brand doesn't say. It's shown as a label and drives a filter, but matching ignores it.

**On a product:**
- `status` is `"current"`, or `"discontinued"` once the product is gone from the brand's site. `status_seen` is the date that was last checked.
- `sold` is `"raw"` (unwashed) or `"one-washed"` (already washed by the brand). Use `null` if the brand doesn't say.
- `fabric.weight_oz` and `fabric.composition` can each be `null` if they aren't stated.
- **`shrinkage`** is `null` when the brand gives none, or when the product won't shrink. Otherwise it's one of these, each with a `source` like the one on a chart:
  - `{ "kind": "percent", "length": 2, "width": 5, ... }`: Nudie's "inseam 2%, width 4.5–6%" becomes the midpoints, with the brand's exact wording in `"note"`.
  - `{ "kind": "amount", "unit": "in", "waist": 1, "inseam": 2, ... }`: Iron Heart's "about 1″ in the waist, up to 2″ in the inseam".

**On a chart:**
- `state` is `"as-sold"` or `"after-wash"`. A product can have both, like Naked & Famous's pre-wash and post-wash tables. Add `"estimated_by_brand": true` when the brand calls its after-wash chart an estimate.
- `from_image` is `true` when the numbers were read from a picture. Every such number gets listed in the PR so someone can check it by hand.
- Optional overrides: `conventions`, `measuring_points`, `tolerance`.

**On a size:**
- `label` is the brand's own text ("32", "W32 L32").
- `waist_tag` is the number on the tag. `length_tag` is used only for W×L brands.

## `exchange-rates.json`

These are the European Central Bank's daily reference rates on the day of the catalogue update, given as units per 1 EUR.

```json
{
  "source": "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml",
  "date": "2026-11-20",
  "base": "EUR",
  "rates": { "USD": 1.05, "GBP": 0.83, "CAD": 1.47, "SEK": 11.4, "JPY": 160.2 }
}
```

The figures in that example are placeholders. Always take the real ones from the ECB file.

## `EXTRACTING.md` in each brand folder

Notes for the next `/update-catalogue` run on that brand:
- where the charts are and how to read them reliably
- any quirks (format drift, units, image charts, cookie walls)
- what was skipped and why
- the date of the last full run

Keep it short and practical. `/update-catalogue` reads it before starting.
