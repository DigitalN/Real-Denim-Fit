# Rogue Territory extraction script

Rogue Territory publishes its size charts only as JPEG images, so `extract.py` does everything **except read the numbers**. Claude reads each image twice, independently, as step 3 of `/update-catalogue` describes. The script needs only Python 3.9 or later with its standard library. On macOS it also uses the built-in `sips` to save enlarged copies of the images.

Every step works in one folder: `--work DIR`, or by default `<tmp>/real-denim-fit/rogue-territory/<date>`.

## 1. Fetch

```sh
python3 tools/catalogue/rogue-territory/extract.py fetch --work <scratchpad>/catalogue-rogue-territory
```

- Downloads `products.json` and keeps the products tagged `DENIM` whose fabric line says "denim". Herringbone, canvas and the like are listed as excluded.
- Downloads each product's chart image to `images/`, retrying transient errors. For a `_480x480`-style rendition it also fetches the full-size original.
- Byte-identical uploads share one chart.
- Writes enlarged copies to `images/large/` and records everything in `manifest.json`.
- Prints each chart with the two read files it expects. It exits with code 1 if an image couldn't be downloaded; run it again, and finished downloads are kept.

## 2. Read every chart twice, independently

Write `reads/<chart>-a.json` (your read) and `reads/<chart>-b.json` (a fresh sub-agent's), in this shape:

```json
{"columns": ["TAG SIZE", "WAIST", "FRONT RISE", "BACK RISE", "UPPER THIGH", "KNEE", "LEG OPENING", "INSEAM"],
 "rows": [["29", "15.5", "10.25", "14.75", "11.25", "7.5", "7.25", "36"]]}
```

- Headings and cells go in exactly as printed, one row per size, with the tag size first.
- Use `""` for an empty cell and `"?"` for a cell you can't read for certain.

## 3. Compare

```sh
python3 tools/catalogue/rogue-territory/extract.py compare --work <same folder>
```

- Compares A and B cell by cell.
- A cell is kept when A and B give the same text, or when a third read (`reads/<chart>-c.json`) matches one of them.
- Everything else is left out.
- Writes `agreed/<chart>.json`.
- Prints the tally (cells read, agreed at once, settled by read C, left out) and the disputed cells by row and column only, never by value, ready to hand to the read-C sub-agent.

## 4. Build

```sh
python3 tools/catalogue/rogue-territory/extract.py build --work <same folder>
```

- Writes `brand.json` and one file per cut to `catalogue/rogue-territory/` (or `--out DIR`), from the agreed cells only.
- Leaves out any cell that isn't a plain number, and any flat waist more than 2″ from half the tag size (such as a printed "5"). It lists each left-out cell and the reason.
- Fabric facts come from each page's `DENIM //` line:
  - the weight
  - `100% cotton`
  - "sanforized", or the FAQ's default
  - "Washed" → washed, "Rinsed" → rinsed, otherwise raw
- Exits with code 1 for anything it can't place, such as a new cut. Add it to `CUT_LABELS`, after checking its label.

**Common options:**
- `--date YYYY-MM-DD`: the `seen` date (default: today)
- `--offline`: for `fetch`, use only what's already in the work folder
- `--no-enlarge`: for `fetch`, skip the `sips` copies

See [`catalogue/rogue-territory/EXTRACTING.md`](../../../catalogue/rogue-territory/EXTRACTING.md) for the site's quirks.
