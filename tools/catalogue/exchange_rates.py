"""Fixes the exchange rates from the European Central Bank's daily
reference rates. Run at catalogue update time:

    python3 tools/catalogue/exchange_rates.py
"""

import json
import os
import re

from common import ROOT, fetch

URL = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml"


def main():
    xml = fetch(URL, cache=False)
    date = re.search(r"time=['\"](\d{4}-\d{2}-\d{2})['\"]", xml).group(1)
    rates = {c: float(r) for c, r in re.findall(r"currency=['\"]([A-Z]{3})['\"]\s+rate=['\"]([\d.]+)['\"]", xml)}
    if "USD" not in rates or "GBP" not in rates:
        raise SystemExit("The ECB file didn't have the expected rates")
    out = {"source": URL, "date": date, "base": "EUR", "rates": dict(sorted(rates.items()))}
    path = os.path.join(ROOT, "catalogue", "exchange-rates.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(out, f, indent=1)
        f.write("\n")
    print(f"wrote {path}: {len(rates)} rates from {date}")


if __name__ == "__main__":
    main()
