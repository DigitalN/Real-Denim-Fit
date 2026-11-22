# 3sixteen extraction script

`extract.py` rebuilds `catalogue/3sixteen/` from 3sixteen.com. It needs only Python 3.9 or later with its standard library (macOS's built-in `/usr/bin/python3` works), and no browser.

```sh
python3 catalogue/3sixteen/tools/extract.py
```

**What it does:**
- downloads `products.json`, then each jean's Measurements modal from Shopify's section endpoint (`/products/<handle>?sections=sizing-modal`), pausing 2 seconds between requests
- reads both modal formats: the `<ul>` columns with fraction glyphs, and the newer `<table>`
- writes `brand.json` and one file per cut (`cs.json`, `ct.json`, `rs.json` …) to `catalogue/3sixteen/`
- doesn't touch `EXTRACTING.md`
- prints a summary to stderr:
  - the fabric facts it inferred for every product, for you to check against new pages
  - which charts publish a flat waist, and which products rely on the FAQ for "sanforized"
  - cells it left out (blank or misaligned)
- exits with code 1 for anything it can't place, such as a new jean whose cut isn't in `CUTS`/`COLLABS`, or an unreadable cell

**How it decides the fabric facts,** from each page's own description and details:
- **weight:** from the fabric bullet, or the description if that's missing or garbled
- **`100% cotton`:** only when the fabric bullet says so
- **`sanforized`:** "unsanforized" → false, "sanforized" → true, otherwise true from the FAQ
- **`finish`:** stonewash, vintage wash or bleach → washed; rinse or garment wash → rinsed; otherwise raw
- **flat waist:** a chart whose median waist is under 0.75 × the tag size gets a flat-waist `conventions` override
- **`OVERRIDES`:** for any product where a rule gets it wrong. Keep the reason next to each entry.

**Options:**
- `--date YYYY-MM-DD`: the `seen` date (default: today)
- `--offline`: rebuild from the cache without the network
- `--cache DIR`: where downloads are kept (default: `<tmp>/real-denim-fit/3sixteen/<date>`)
- `--out DIR`: write somewhere other than `catalogue/3sixteen/`
- `--only HANDLE`: build one product, print its JSON and write nothing

See [`catalogue/3sixteen/EXTRACTING.md`](../EXTRACTING.md) for the site's quirks.
