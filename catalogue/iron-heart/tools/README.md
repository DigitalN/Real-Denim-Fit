# Iron Heart extraction script

`extract.py` rebuilds `catalogue/iron-heart/` from ironheart.co.uk. It needs only Python 3.9 or later with its standard library (macOS's built-in `/usr/bin/python3` works), and no browser.

```sh
python3 catalogue/iron-heart/tools/extract.py
```

**What it does:**
- downloads `products.json` and each jeans product page, pausing 2 seconds between requests
- writes `brand.json` and one file per cut (`555.json`, `634.json` …) to `catalogue/iron-heart/`
- doesn't touch `EXTRACTING.md`
- prints a summary to stderr:
  - product and size counts
  - products skipped, and why (no chart, not denim)
  - blank cells
  - cells where the brand's inch and cm tables disagree
- exits with code 1 if a page has something it doesn't recognise, such as a new Fabric Focus heading

**Options:**
- `--date YYYY-MM-DD`: the `seen` date written on every number (default: today)
- `--offline`: rebuild from the cache without touching the network
- `--cache DIR`: where downloads are kept (default: `<tmp>/real-denim-fit/iron-heart/<date>`, so a second run on the same day reuses them)
- `--out DIR`: write somewhere other than `catalogue/iron-heart/`, for example to diff against the committed files
- `--only HANDLE`: build one product (such as `ih-634s`), print its JSON and write nothing

**When the site changes,** fix the script rather than the output. The rules it relies on (where the chart, cut, fabric and Fabric Focus are, and which products count as jeans) are explained in [`catalogue/iron-heart/EXTRACTING.md`](../EXTRACTING.md).

**Two tables to update by hand:**
- `USE_CM`: products whose inch table has a typo the cm table doesn't share
- `FABRIC_FOCUS`: the Fabric Focus headings and what each means for `sanforized`, `sold` and `finish`
