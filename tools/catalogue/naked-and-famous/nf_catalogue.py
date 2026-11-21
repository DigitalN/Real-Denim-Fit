#!/usr/bin/env python3
"""Re-read the Naked & Famous catalogue from nakedandfamousdenim.com.

Python 3 standard library only. Run from the repository root:

    python3 tools/catalogue/naked-and-famous/nf_catalogue.py fetch --cache ~/nf-cache
    python3 tools/catalogue/naked-and-famous/nf_catalogue.py build --cache ~/nf-cache
    python3 tools/catalogue/naked-and-famous/nf_catalogue.py check

fetch   Downloads products.json (all pages) into CACHE/products/<today>/. Then it
        downloads every in-scope jeans product page into CACHE/pages/<handle>.html,
        one at a time, 3 s apart. Pages already in the cache are kept unless you
        pass --refresh; --handles a,b,c limits the run to those pages.
        CACHE/index.json records the date each page was read.
build   Rewrites the cut files in catalogue/naked-and-famous/ (--out to change)
        from the cache, using the manual calls in decisions.json next to this file.
        Products that have gone from N&F's site are kept as "discontinued".
        brand.json and EXTRACTING.md are left alone. Writes CACHE/report.json with
        every product included or skipped, every number left out and why, and
        every flag. Exits 2 when there are flags: decide each one in
        decisions.json, then run build again.
check   Checks the catalogue files against docs/catalogue-format.md and the
        CI rules in docs/PLAN.md. Exits 1 on any error.

The cache, however it's filled:
    CACHE/products/<YYYY-MM-DD>/products-<n>.json   products.json?limit=250&page=<n>
    CACHE/pages/<handle>.html                       the product page as served
    CACHE/index.json   {"pages": {"<handle>": {"seen": "<date read>", "snapshot": "<products date>"}}}
The newest products/ folder is the listing build works from. Its date becomes
status_seen.

How the charts are read, and why each manual call was made, is in
catalogue/naked-and-famous/EXTRACTING.md.
"""
import argparse
import datetime
import html
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, '..', '..', '..'))
DEFAULT_OUT = os.path.join(REPO, 'catalogue', 'naked-and-famous')
DECISIONS = os.path.join(HERE, 'decisions.json')
SITE = 'https://nakedandfamousdenim.com'
USER_AGENT = ('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 '
              '(KHTML, like Gecko) Chrome/130.0 Safari/537.36')

BRAND_CONV = {'waist': 'around', 'seat': 'flat', 'thigh': 'flat', 'knee': 'flat', 'leg_opening': 'flat'}
KEY_ORDER = ['waist', 'front_rise', 'back_rise', 'seat', 'thigh', 'knee', 'leg_opening', 'inseam']
RANGES = {  # inches; anything outside is flagged (build) or an error (check)
    'waist': (20, 56), 'front_rise': (7, 16), 'back_rise': (11, 21), 'seat': (14, 30),
    'thigh': (7.5, 19), 'knee': (5, 15), 'leg_opening': (4.5, 14), 'inseam': (24, 38),
}


# ---------------------------------------------------------------- fetching

def http_get(url, tries=3):
    """GET with N&F's 429 rate limit in mind: back off and retry."""
    delay = 30
    for attempt in range(tries):
        req = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
        try:
            with urllib.request.urlopen(req, timeout=60) as r:
                return r.status, r.read()
        except urllib.error.HTTPError as e:
            if e.code in (429, 502, 503) and attempt < tries - 1:
                wait = int(e.headers.get('Retry-After') or delay)
                print(f'  {e.code} on {url}; waiting {wait} s', file=sys.stderr)
                time.sleep(wait)
                delay *= 2
                continue
            return e.code, b''
        except urllib.error.URLError as e:
            if 'CERTIFICATE_VERIFY_FAILED' in str(e):
                sys.exit('SSL certificates are missing for this Python. On macOS, run '
                         '"Install Certificates.command" in its Applications folder, or use /usr/bin/python3.')
            if attempt < tries - 1:
                time.sleep(delay)
                continue
            raise
    return 0, b''


def cmd_fetch(args):
    cache = os.path.expanduser(args.cache)
    today = datetime.date.today().isoformat()
    snap_dir = os.path.join(cache, 'products', today)
    pages_dir = os.path.join(cache, 'pages')
    os.makedirs(snap_dir, exist_ok=True)
    os.makedirs(pages_dir, exist_ok=True)
    n = 1
    while True:
        status, body = http_get(f'{SITE}/products.json?limit=250&page={n}')
        if status == 429:
            sys.exit(f'products.json page {n}: N&F refused the request (HTTP 429, "Too Many Requests"). '
                     'On 2026-11-21 it answered this script that way every time. See "How to rerun" in '
                     'catalogue/naked-and-famous/EXTRACTING.md: build works from any cache laid out as described there.')
        if status != 200:
            sys.exit(f'products.json page {n}: HTTP {status}')
        products = json.loads(body)['products']
        if not products:
            break
        with open(os.path.join(snap_dir, f'products-{n}.json'), 'wb') as f:
            f.write(body)
        print(f'products.json page {n}: {len(products)} products')
        n += 1
        time.sleep(args.delay)
    dec = load_decisions()
    snapshot = load_snapshot(cache, today)
    index = load_index(cache)
    wanted = args.handles.split(',') if args.handles else None
    todo = []
    for h, p in sorted(snapshot.items()):
        if p['product_type'] != 'Jeans' or excluded(p, dec):
            continue
        if wanted is not None and h not in wanted:
            continue
        if not args.refresh and h in index['pages'] and os.path.exists(os.path.join(pages_dir, h + '.html')):
            continue
        todo.append(h)
    print(f'{len(todo)} product pages to fetch, {args.delay} s apart')
    for i, h in enumerate(todo, 1):
        status, body = http_get(f'{SITE}/products/{h}')
        if status == 200:
            with open(os.path.join(pages_dir, h + '.html'), 'wb') as f:
                f.write(body)
            index['pages'][h] = {'seen': today, 'snapshot': today}
            print(f'  [{i}/{len(todo)}] {h}')
        else:
            print(f'  [{i}/{len(todo)}] {h}: HTTP {status}, skipped', file=sys.stderr)
        if i % 20 == 0:
            save_index(cache, index)
        time.sleep(args.delay)
    save_index(cache, index)
    print('done')


def load_index(cache):
    path = os.path.join(cache, 'index.json')
    if os.path.exists(path):
        return json.load(open(path))
    return {'pages': {}}


def save_index(cache, index):
    with open(os.path.join(cache, 'index.json'), 'w') as f:
        json.dump(index, f, indent=1, sort_keys=True)


def snapshot_dates(cache):
    d = os.path.join(cache, 'products')
    return sorted(x for x in os.listdir(d) if re.fullmatch(r'\d{4}-\d{2}-\d{2}', x)) if os.path.isdir(d) else []


_snapshots = {}


def load_snapshot(cache, date):
    if date not in _snapshots:
        d = os.path.join(cache, 'products', date)
        allp = {}
        for f in sorted(os.listdir(d)):
            if re.fullmatch(r'products-\d+\.json', f):
                for p in json.load(open(os.path.join(d, f)))['products']:
                    allp[p['handle']] = p
        _snapshots[date] = allp
    return _snapshots[date]


# ---------------------------------------------------------------- decisions

def load_decisions():
    return json.load(open(DECISIONS, encoding='utf-8'))


def split_title(title):
    """'Weird Guy - Blue Haze Selvedge - Indigo' -> ('Weird Guy', 'Blue Haze Selvedge, Indigo')."""
    parts = [re.sub(r'\s+', ' ', p).strip() for p in re.split(r'\s+-\s+', title.strip())]
    parts = [p for p in parts if p]
    return parts[0], ', '.join(parts[1:])


def excluded(p, dec):
    """Reason the product is out of scope (not denim, not jeans), or None."""
    pd = dec['products'].get(p['handle'], {})
    if 'exclude' in pd:
        return pd['exclude']
    cut, fabric = split_title(p['title'])
    for rule in dec['exclude_rules']:
        field = cut if rule['field'] == 'cut' else fabric
        if re.search(rule['pattern'], field, re.I):
            return rule['reason']
    return None


# ---------------------------------------------------------------- charts

LABELS = {
    'waist': 'waist',
    'frontrise': 'front_rise',
    'backrise': 'back_rise',
    'rearrise': 'back_rise',
    'blackrise': 'back_rise',  # typo on two Wide Wild West charts; it sits where Back Rise goes
    'upperthigh': 'thigh',
    'thigh': 'thigh',
    'knee': 'knee',
    'legopening': 'leg_opening',
    'opening': 'leg_opening',
    'hem': 'leg_opening',
    'inseam': 'inseam',
    'hips': 'seat',
    'hip': 'seat',
    'seat': 'seat',
}
NUM = re.compile(r'^\d+(\.\d*)?$')
TAG_RE = re.compile(r'<[a-zA-Z][a-zA-Z0-9-]*\s[^>]*?data-(?:post-)?columns=[^>]*>', re.S)
ATTR_RE = re.compile(r'([a-zA-Z0-9_:-]+)(?:="([^"]*)")?')


class ChartError(Exception):
    pass


def chart_elements(page):
    """Every element on the page that carries chart JSON, as dicts of unescaped attributes."""
    out = []
    for m in TAG_RE.finditer(page):
        tag = m.group(0)
        attrs = {}
        for a, v in ATTR_RE.findall(tag[tag.index(' '):]):
            attrs[a] = html.unescape(v) if v else ''
        out.append(attrs)
    return out


def number(v):
    """Cell text -> JSON number, exactly as published. None when the cell is empty."""
    if v is None:
        return None
    s = str(v).strip()
    if s in ('', '-', '—', '–'):
        return None
    if not NUM.match(s):
        raise ChartError(f'cell {v!r} is not a number')
    f = float(s)  # "12." -> 12, "29.00" -> 29
    return int(f) if f.is_integer() else f


def label_key(label):
    raw = str(label or '').strip()
    post = raw.startswith('*') or raw.endswith('*')  # Hedorah template: after-wash rows end in "*"
    norm = re.sub(r'[^a-z]', '', raw.lower())
    if norm not in LABELS:
        raise ChartError(f'unknown row label {label!r}')
    return LABELS[norm], post


def parse_table(columns, rows, where):
    """-> (header, size labels, {state: {'rows': {key: [values]}, 'units': set, 'conventions': {}}})."""
    if not isinstance(columns, list) or len(columns) < 2:
        raise ChartError(f'{where}: no size columns')
    header = str(columns[0] or '').strip()
    sizes = [str(c).strip() if c is not None else '' for c in columns[1:]]
    tables = {}
    for row in rows:
        norm = re.sub(r'[^a-z]', '', str(row.get('label') or '').lower())
        if norm in ('postwash', 'prewash', 'afterwash') and all(v in (None, '', '*') for v in row.get('values') or []):
            continue  # separator row: "POST WASH*" with every cell "*"
        key, post = label_key(row.get('label'))
        state = 'after-wash' if post else 'as-sold'
        t = tables.setdefault(state, {'rows': {}, 'units': set(), 'conventions': {}})
        if key in t['rows']:
            raise ChartError(f'{where}: duplicate row {row.get("label")!r}')
        vals = row.get('values') or []
        if len(vals) > len(sizes):
            raise ChartError(f'{where}: row {row.get("label")!r} has {len(vals)} values for {len(sizes)} sizes')
        t['rows'][key] = [number(v) for v in vals] + [None] * (len(sizes) - len(vals))
        t['units'].add(row.get('unit'))
        if row.get('convention') is not None:
            t['conventions'][key] = row.get('convention')
    return header, sizes, tables


def read_charts(page):
    """The charts on one product page, or None when the page has no chart panel.

    The chart is HTML-escaped JSON in attributes on <div id="nfd-size-chart-panel">:
      data-columns / data-rows              the as-sold chart
      data-post-columns / data-post-rows    the after-wash chart (Cat's Pyjamas, Prince of Slub)
    The Hedorah template instead puts the after-wash rows in data-rows, after a
    "POST WASH*" separator, with labels ending in "*". Copies of the chart in
    other elements (data-cats-pyjamas-chart) must agree with the panel.
    """
    els = chart_elements(page)
    main = [e for e in els if e.get('id') == 'nfd-size-chart-panel']
    if not main:
        return None
    if len(main) > 1:
        raise ChartError('more than one #nfd-size-chart-panel')
    p = main[0]
    header, sizes, tables = parse_table(json.loads(p.get('data-columns') or '[]'),
                                        json.loads(p.get('data-rows') or '[]'), 'data-rows')
    pre = tables.get('as-sold')
    post = tables.get('after-wash')
    post_source = 'starred rows' if post else None
    post_sizes = sizes
    if p.get('data-post-rows'):
        if post:
            raise ChartError('both starred rows and data-post-rows')
        _, post_sizes, ptables = parse_table(json.loads(p.get('data-post-columns') or p.get('data-columns')),
                                             json.loads(p['data-post-rows']), 'data-post-rows')
        if set(ptables) != {'as-sold'}:
            raise ChartError('starred rows inside data-post-rows')
        post = ptables['as-sold']
        post_source = 'data-post-rows'
    agree = True
    for e in els:
        if e is p:
            continue
        try:
            _, s2, t2 = parse_table(json.loads(e.get('data-columns') or '[]'),
                                    json.loads(e.get('data-rows') or '[]'), 'copy')
        except (ChartError, ValueError):
            agree = False
            continue
        t2 = t2.get('as-sold')
        if t2 and s2 == sizes and t2['rows'] == pre['rows']:
            continue
        if t2 and post and s2 == post_sizes and t2['rows'] == post['rows']:
            continue
        agree = False
    # Cat's Pyjamas / Prince of Slub: "Estimated after the first wash."
    # Hedorah: "Post-Wash Measurements – expected measurements after the first wash"
    estimated = bool(post) and bool(re.search(
        r'Estimated after the first wash|expected measurements after the first wash', page))
    return {'header': header, 'sizes': sizes, 'pre': pre, 'post': post, 'post_sizes': post_sizes,
            'post_source': post_source, 'estimated': estimated, 'copies_agree': agree}


# ---------------------------------------------------------------- fabric

FIBRES = {
    'cotton': 'cotton',
    'organic cotton': 'cotton',
    'organic foxfibre cotton': 'cotton',
    'foxfibre cotton': 'cotton',
    'recycled cotton': 'cotton',
    'mexican upland cotton': 'cotton',
    'elastane': 'elastane',
    'polyurethane': 'elastane',  # Japanese mills label elastane "polyurethane"; N&F copies that
    'pu': 'elastane',
    'spandex': 'elastane',
    'stretch': 'elastane',       # "2% Stretch": no fibre named, but N&F calls the fabric a stretch denim
    'lycra': 'elastane',
    'linen': 'linen',
    'hemp': 'hemp',
    'rayon': 'rayon',
    'polyester': 'polyester',
    'nylon': 'nylon',
    'cashmere': 'cashmere',
    'kevlar': 'kevlar',
    'milk protein fiber': 'milk protein fibre',
    'lyocell': 'lyocell',
    'tencel': 'lyocell',
    'wool': 'wool',
    'silk': 'silk',
}
PAIR = re.compile(r'(\d+(?:\.\d+)?)\s*%\s*([A-Za-z][A-Za-z®™ \-]*?)\s*(?=[,/&+]|\band\b|$|\d)', re.I)
COMP_LINE = re.compile(r'^(?:\d+(?:\.\d+)?\s*%\s*[A-Za-z][A-Za-z®™ \-]*?\s*(?:[,/&+]|\band\b)?\s*)+$', re.I)
WEIGHT_LINE = re.compile(r'^(?:weight:\s*)?(\d+(?:\.\d+)?)\s*-?\s*oz\b', re.I)
WEIGHT_ANY = re.compile(r'(\d+(?:\.\d+)?)\s*-?\s*oz\b', re.I)


def description_lines(body_html):
    """The product description (products.json body_html) as clean lines of text."""
    t = re.sub(r'<br\s*/?>|</p>|</li>|</div>|</h\d>|</span>\s*</p>', '\n', body_html or '')
    t = html.unescape(re.sub(r'<[^>]+>', ' ', t))
    out = []
    for line in t.split('\n'):
        line = re.sub(r'[ \t ​]+', ' ', line).strip()
        if line:
            out.append(line)
    return out


def parse_composition_line(line):
    pairs = []
    for pct, name in PAIR.findall(line):
        f = FIBRES.get(re.sub(r'\s+', ' ', re.sub(r'[®™]', '', name).strip().lower()))
        if f is None:
            return None, f'unknown fibre {name!r} in {line!r}'
        v = float(pct)
        pairs.append({'fibre': f, 'percent': int(v) if v.is_integer() else v})
    total = sum(x['percent'] for x in pairs)
    if abs(total - 100) > 0.01:
        return None, f'composition {line!r} adds up to {total}'
    merged = {}
    for x in pairs:
        merged[x['fibre']] = merged.get(x['fibre'], 0) + x['percent']
    out = [{'fibre': k, 'percent': v} for k, v in merged.items()]
    out.sort(key=lambda x: -x['percent'])
    return out, None


def read_fabric(body_html):
    """-> (fabric dict, flags, evidence). The description ends with short spec lines,
    e.g. "13.6oz Japanese Selvedge Denim" and "98% Cotton / 2% Elastane"."""
    lines = description_lines(body_html)
    flags, ev = [], {}

    def no_patch(l, m):  # "5oz vegetable-tanned leather patch" is not the fabric
        return not re.search(r'patch|leather', l[max(0, m.start() - 40):m.end() + 40], re.I)

    weights = [m.group(1) for l in lines if len(l) <= 90 and not re.search(r'patch|leather', l, re.I)
               for m in [WEIGHT_LINE.match(l)] if m]
    if not weights:
        prose = [m.group(1) for l in lines for m in WEIGHT_ANY.finditer(l) if no_patch(l, m)]
        if prose and len(set(float(w) for w in prose)) == 1:
            weights = prose[:1]
        elif prose:
            flags.append(f'several weights in prose: {sorted(set(prose))}')
    distinct = sorted(set(float(w) for w in weights))
    weight = None
    if len(distinct) == 1:
        w = distinct[0]
        weight = int(w) if w.is_integer() else w
        prose = {float(m.group(1)) for l in lines for m in WEIGHT_ANY.finditer(l) if no_patch(l, m)}
        if prose - {w}:
            flags.append(f'weight {w} in spec block but prose also says {sorted(prose - {w})}')
    elif len(distinct) > 1:
        flags.append(f'several weights in spec block: {distinct}')

    comp = None
    comp_lines = [l for l in lines if len(l) < 90 and '%' in l and COMP_LINE.match(l)]
    partial = [l for l in lines if len(l) < 90 and '%' in l and re.match(r'^(warp|weft)\s*:', l, re.I)]
    parsed = []
    for l in comp_lines:
        c, error = parse_composition_line(l)
        if error:
            flags.append(error)
        else:
            parsed.append(c)
    if len({repr(c) for c in parsed}) == 1:
        comp = parsed[0]
        ev['composition_line'] = ' '.join(comp_lines)
    elif parsed:
        flags.append(f'several compositions: {comp_lines}')
    elif not partial and len({m.group(0).lower() for l in lines if len(l) < 90
                              for m in [re.search(r'\b100\s*%\s*(organic\s+)?cotton\b', l, re.I)] if m}) == 1:
        comp = [{'fibre': 'cotton', 'percent': 100}]  # "20oz 100% cotton Japanese selvedge denim"
    else:
        flags.append('no composition line in spec block' + (f' (partial: {partial})' if partial else ''))

    low = '\n'.join(lines).lower()
    if re.search(r'un-?sanfori[sz]ed|non-?sanfori[sz]ed|not sanfori[sz]ed|loomstate|loom-state', low):
        sanforized = False
    elif re.search(r'sanfori[sz]ed', low):
        sanforized = True
    else:
        sanforized = None
    return {'weight_oz': weight, 'composition': comp, 'sanforized': sanforized}, flags, ev


# ---------------------------------------------------------------- finish

# docs/catalogue-format.md, decided from the description:
#   washed: faded on purpose (bleached, enzyme washed to a faded tone, stonewashed ...)
#   rinsed: washed or soaked once, not faded (rinse-washed, pre-washed, fabric rinsed, tempi)
#   raw:    nothing of the kind is said; the site footer says "no washes, no distressing"
WASHED = [r'\benzyme[- ]washed\b', r'\bfabric bleached\b', r'\bbleach(?:ed)? the denim\b',
          r'\bstone[- ]?wash', r'\bvintage wash', r'\bacid wash', r'\bdistressed\b']
RINSED = [
    r'^rinse[- ]?washed\b',  # spec line "Rinse-washed" / "Rinse washed"
    r'\bjeans? (?:have|has) been rinse[- ]washed',
    r'\bjeans come rinse[- ]washed',
    r'\beach pair has undergone a rinse wash',
    r'\b(?:denim|fabric) has been rinsed\b',
    r'\bone[- ]wash treatment\b',
    r'^pre-?washed\b',
    r'\bpre-?rinsed\b',
    r'\bfabric[- ]rinsed\b',
    r'\brinsed (?:in japan|at the fabric stage)\b',
    r'\bte[mn]pi\b',  # tempi: a soak and hang-dry that shrinks the fabric
]


def finish_state(body_html):
    lines = description_lines(body_html)
    for kind, pats in (('washed', WASHED), ('rinsed', RINSED)):
        for line in lines:
            for pat in pats:
                m = re.search(pat, line, re.I)
                if m:
                    return kind, line[max(0, m.start() - 50):m.end() + 50]
    return 'raw', None


# ---------------------------------------------------------------- building

def slug(s):
    s = s.lower().replace('&', ' and ').replace('’', '').replace("'", '')
    s = re.sub(r'[®™©]', '', s)
    return re.sub(r'[^a-z0-9]+', '-', s).strip('-')


def list_price(p):
    """The regular price in USD: compare_at_price when the product is on sale."""
    prices = {float(v['price']) for v in p['variants']}
    cmps = {float(v['compare_at_price']) for v in p['variants'] if v.get('compare_at_price')}
    if len(prices) != 1 or len(cmps) > 1:
        raise ValueError(f'prices differ by size: {prices} {cmps}')
    price = prices.pop()
    amount = max(price, max(cmps) if cmps else price)  # compare_at 0.00 means no regular price
    return (int(amount) if amount.is_integer() else amount), amount != price


def outliers(sizes, table, state):
    """Cells that break the run of their row or fall outside sensible ranges, and rows
    that can't be what their label says. Flagged for a decision, never dropped silently."""
    out = []
    rows = table['rows']
    for k, row in rows.items():
        vals = [(i, v) for i, v in enumerate(row) if v is not None]
        for j, (i, v) in enumerate(vals):
            nb = [vals[x][1] for x in (j - 1, j + 1) if 0 <= x < len(vals)]
            limit = 4 if k == 'waist' else 2
            if nb and all(abs(v - n) > limit for n in nb):
                out.append(f'{k} at size {sizes[i]} is {v}; neighbours {nb}')
        lo, hi = RANGES[k]
        for i, v in enumerate(row):
            if v is not None and not lo <= v <= hi:
                out.append(f'{k} at size {sizes[i]} is {v}, outside {lo}-{hi}')
        if k not in ('inseam', 'waist'):
            seq = [(sizes[i], v) for i, v in enumerate(row) if v is not None]
            for (l1, a), (l2, b) in zip(seq, seq[1:]):
                if b < a - 0.5:
                    out.append(f'{k} at size {l2} is {b}, below {a} at {l1}')
    if 'waist' in rows and state == 'as-sold':
        for i, v in enumerate(rows['waist']):
            if v is not None and re.fullmatch(r'\d{2}', sizes[i]) and not -3 <= v - int(sizes[i]) <= 9:
                out.append(f'waist at size {sizes[i]} is {v}, far from the tag')
        seq = [(sizes[i], v) for i, v in enumerate(rows['waist']) if v is not None]
        for (l1, a), (l2, b) in zip(seq, seq[1:]):
            if b < a - 0.75:
                out.append(f'waist at size {l2} is {b}, below {a} at {l1}')
    if 'seat' in rows and 'thigh' in rows:
        bad = [sizes[i] for i, (s, t) in enumerate(zip(rows['seat'], rows['thigh']))
               if s is not None and t is not None and s <= t]
        if len(bad) > 2:
            out.append(f'seat row is not larger than thigh at sizes {bad}')
    return out


def build_sizes(sizes, table, pd, log, where, post=False):
    drop = pd.get('drop_sizes', {})
    drop_rows = pd.get('post_drop_rows' if post else 'drop_rows') or {}
    drop_cells = pd.get('post_drop_cells' if post else 'drop_cells') or {}
    for k, why in drop_rows.items():
        if k in table['rows']:
            log.append(f'{where}: row {k} left out: {why}')
    out = []
    for i, label in enumerate(sizes):
        if label in drop:
            log.append(f'{where}: size {label!r} left out: {drop[label]}')
            continue
        if not re.fullmatch(r'\d{2}', label):
            log.append(f'FLAG {where}: size label {label!r} is not a two-digit waist tag; left out until decided')
            continue
        m = {}
        for k in KEY_ORDER:
            row = table['rows'].get(k)
            if k in drop_rows:
                continue
            if k in drop_cells.get(label, {}):
                log.append(f'{where}: size {label} {k} ({row[i]}) left out: {drop_cells[label][k]}')
                continue
            if row is not None and row[i] is not None:
                m[k] = row[i]
        if not m:
            continue  # a size column with no numbers in it
        out.append({'label': label, 'waist_tag': int(label), 'measurements': m})
    tags = [s['waist_tag'] for s in out]
    if tags != sorted(tags) or len(set(tags)) != len(tags):
        log.append(f'FLAG {where}: sizes out of order {tags}')
    return out


def chart_conventions(table, log, where):
    """A chart-level override only where a row's own convention contradicts brand.json."""
    over = {}
    for k, c in table['conventions'].items():
        if k not in BRAND_CONV:
            continue
        mapped = {'circumference': 'around', 'flat': 'flat'}.get(c)
        if mapped is None:
            log.append(f'{where}: row {k} convention {c!r} ignored (the measuring guide applies)')
        elif mapped != BRAND_CONV[k]:
            over[k] = mapped
    return over


def build_product(h, p, page, seen, status_seen, dec, report):
    """One product's catalogue entry, or None when it's skipped."""
    pd = dec['products'].get(h, {})
    url = f'{SITE}/products/{h}'
    cut_name, fabric_name = split_title(p['title'])
    cut_name = dec['cut_names'].get(cut_name, cut_name)
    if cut_name not in dec['cuts']:
        report['flags'].append({'handle': h, 'flags': [f'new cut {cut_name!r}: add it to "cuts" in decisions.json']})
        return None, cut_name
    r = read_charts(page)
    if r is None:
        report['skipped'].append({'handle': h, 'title': p['title'], 'reason': 'no size chart on the product page'})
        return None, cut_name
    if not r['copies_agree']:
        raise ChartError(f'{h}: copies of the chart on the page disagree')
    log, flags = [], []
    units = r['pre']['units'] | (r['post']['units'] if r['post'] else set())
    if units - {'in', None}:
        raise ChartError(f'{h}: units {units}')
    text = ' '.join(description_lines(p['body_html']))
    if not re.search(r'\bdenims?\b', re.sub(r'naked\s*(&|and)\s*famous\s*denim', '', text, flags=re.I), re.I) \
            and not pd.get('denim_reason'):
        flags.append('the description never calls the fabric denim: add an exclude rule, or a "denim_reason"')
    reviewed = set(pd.get('reviewed_outliers', []))
    for state, sz, tb in (('as-sold', r['sizes'], r['pre']), ('after-wash', r['post_sizes'], r['post'])):
        if not tb:
            continue
        rows_dropped = pd.get('drop_rows' if state == 'as-sold' else 'post_drop_rows') or {}
        cells_dropped = pd.get('drop_cells' if state == 'as-sold' else 'post_drop_cells') or {}
        for o in outliers(sz, tb, state):
            m = re.match(r'(\w+) at size (\d+) ', o)
            if m and (m.group(1) in rows_dropped or m.group(1) in cells_dropped.get(m.group(2), {})
                      or m.group(2) in pd.get('drop_sizes', {})):
                continue  # already decided
            if o.startswith('seat row') and 'seat' in rows_dropped:
                continue
            if m and 'below' in o:  # a drop is decided if the size or cell before it was left out
                prev = re.search(r'at (\d+)$', o)
                if prev and (m.group(1) in cells_dropped.get(prev.group(1), {})
                             or prev.group(1) in pd.get('drop_sizes', {})):
                    continue
            if o not in reviewed:
                flags.append(f'{state}: {o}')

    charts = []
    ch = {'state': 'as-sold', 'source': {'url': url, 'seen': seen}, 'unit': 'in', 'from_image': False,
          'sizes': build_sizes(r['sizes'], r['pre'], pd, log, f'{h} as-sold')}
    conv = chart_conventions(r['pre'], log, f'{h} as-sold')
    if conv:
        ch['conventions'] = conv
    charts.append(ch)
    if r['post']:
        ch = {'state': 'after-wash', 'source': {'url': url, 'seen': seen}, 'unit': 'in', 'from_image': False}
        if r['estimated']:
            ch['estimated_by_brand'] = True
        ch['sizes'] = build_sizes(r['post_sizes'], r['post'], pd, log, f'{h} after-wash', post=True)
        conv = chart_conventions(r['post'], log, f'{h} after-wash')
        if conv:
            ch['conventions'] = conv
        charts.append(ch)
        log.append(f'{h}: after-wash chart from {r["post_source"]}')

    fab, fflags, ev = read_fabric(p['body_html'])
    cl = ev.get('composition_line', '')
    if re.search(r'\d\s*%\s*stretch\b', cl, re.I):
        log.append(f'{h}: "% Stretch" recorded as elastane: {cl!r}')
    if re.search(r'polyurethane|\bpu\b', cl, re.I):
        log.append(f'{h}: polyurethane recorded as elastane: {cl!r}')
    for k in ('weight_oz', 'composition', 'sanforized'):
        if k in pd.get('fabric', {}):
            fab[k] = pd['fabric'][k]
            log.append(f'{h}: fabric {k} set by decision: {pd["fabric_reason"]}')
    if fflags and not pd.get('fabric_reason'):
        flags.extend(fflags)
    if fab['sanforized'] is None and 'sanforized' not in pd.get('fabric', {}):
        fab['sanforized'] = dec['default_sanforized']['value']

    finish, why = finish_state(p['body_html'])
    if 'finish' in pd:
        finish = pd['finish']
        log.append(f'{h}: finish {finish} by decision: {pd["finish_reason"]}')
    elif why:
        log.append(f'{h}: finish {finish} because the description says: {why!r}')
    sold = {'raw': 'raw', 'rinsed': 'one-washed', 'washed': 'one-washed'}.get(finish)

    if fab['sanforized'] is False and not r['post'] and not pd.get('single_chart_reason'):
        because = [x.strip() for x in re.split(r'(?<=[.!])\s+', text) if re.search(r'te[mn]pi|rinse[ -]wash', x, re.I)]
        if because:
            log.append(f'{h}: unsanforized, one chart only; the description says: {because[0][:220]!r}')
        else:
            flags.append('unsanforized but no after-wash chart')
    if pd.get('single_chart_reason'):
        log.append(f'{h}: one chart only: {pd["single_chart_reason"]}')
    if r['post'] and fab['sanforized'] is not False:
        flags.append('after-wash chart but not marked unsanforized')

    amount, on_sale = list_price(p)
    if on_sale:
        log.append(f'{h}: on sale; list price is the regular (compare-at) price {amount}')
    shrink = pd.get('shrinkage', 'default')
    if shrink == 'default':
        shrink = dec['default_shrinkage'] if fab['sanforized'] and sold == 'raw' else None

    for n in log:
        if n.startswith('FLAG '):
            flags.append(n[5:])
    report['notes'].extend(n for n in log if not n.startswith('FLAG '))
    if flags:
        report['flags'].append({'handle': h, 'flags': flags})
    report['included'].append({'handle': h, 'sizes': [len(c['sizes']) for c in charts]})
    return {
        'id': f'{dec["cuts"][cut_name]["id"]}-{slug(fabric_name)}',
        'name': fabric_name,
        'url': url,
        'status': 'current',
        'status_seen': status_seen,
        'fabric': fab,
        'sold': sold,
        'finish': finish,
        'list_price': {'amount': amount, 'currency': 'USD', 'url': url, 'seen': seen},
        'shrinkage': shrink,
        'charts': charts,
    }, cut_name


def cmd_build(args):
    cache = os.path.expanduser(args.cache)
    out_dir = os.path.abspath(args.out)
    dec = load_decisions()
    dates = snapshot_dates(cache)
    if not dates:
        sys.exit(f'no products.json snapshot in {cache}: run fetch first')
    status_seen = dates[-1]
    listing = load_snapshot(cache, status_seen)
    index = load_index(cache)
    report = {'included': [], 'skipped': [], 'discontinued': [], 'notes': [], 'flags': []}
    cuts = {}
    for h, latest in sorted(listing.items()):
        if latest['product_type'] != 'Jeans':
            continue
        reason = excluded(latest, dec)
        if reason:
            report['skipped'].append({'handle': h, 'title': latest['title'], 'reason': reason})
            continue
        entry = index['pages'].get(h)
        page_path = os.path.join(cache, 'pages', h + '.html')
        if not entry or not os.path.exists(page_path):
            report['flags'].append({'handle': h, 'flags': ['page not in the cache: run fetch']})
            continue
        # Read the product as it was listed when its page was fetched.
        p = load_snapshot(cache, entry['snapshot']).get(h, latest)
        page = open(page_path, encoding='utf-8').read()
        prod, cut_name = build_product(h, p, page, entry['seen'], status_seen, dec, report)
        if prod:
            cuts.setdefault(cut_name, []).append(prod)

    # Products in the current files that N&F no longer lists stay, as discontinued.
    by_id = {m['id']: name for name, m in dec['cuts'].items()}
    if os.path.isdir(out_dir):
        for f in sorted(os.listdir(out_dir)):
            if not f.endswith('.json') or f == 'brand.json':
                continue
            old = json.load(open(os.path.join(out_dir, f), encoding='utf-8'))
            for prod in old['products']:
                h = prod['url'].rsplit('/', 1)[1]
                if h not in listing:
                    if prod['status'] != 'discontinued':
                        prod['status'] = 'discontinued'
                        prod['status_seen'] = status_seen
                    cuts.setdefault(by_id.get(old['id'], old['name']), []).append(prod)
                    report['discontinued'].append(h)

    os.makedirs(out_dir, exist_ok=True)
    for f in os.listdir(out_dir):  # rewrite every cut file; brand.json and EXTRACTING.md stay
        if f.endswith('.json') and f != 'brand.json':
            os.remove(os.path.join(out_dir, f))
    n_sizes = 0
    for cut_name, prods in sorted(cuts.items()):
        meta = dec['cuts'][cut_name]
        prods.sort(key=lambda x: x['id'])
        ids = [x['id'] for x in prods]
        if len(set(ids)) != len(ids):
            sys.exit(f'duplicate product ids in {cut_name}: {sorted(i for i in ids if ids.count(i) > 1)}')
        doc = {'id': meta['id'], 'brand': 'naked-and-famous', 'name': cut_name,
               'brand_label': meta['brand_label'], 'line': meta['line'], 'products': prods}
        with open(os.path.join(out_dir, meta['id'] + '.json'), 'w', encoding='utf-8') as fh:
            json.dump(doc, fh, indent=2, ensure_ascii=False)
            fh.write('\n')
        n_sizes += sum(len(x['charts'][0]['sizes']) for x in prods)
    report_path = os.path.expanduser(args.report or os.path.join(cache, 'report.json'))
    with open(report_path, 'w', encoding='utf-8') as fh:
        json.dump(report, fh, indent=1, ensure_ascii=False)
    print(f'{sum(len(v) for v in cuts.values())} products in {len(cuts)} cuts, {n_sizes} sizes; '
          f'{len(report["skipped"])} skipped, {len(report["discontinued"])} discontinued; report in {report_path}')
    if report['flags']:
        print(f'\n{len(report["flags"])} products need a decision in decisions.json:', file=sys.stderr)
        for fl in report['flags']:
            for x in fl['flags']:
                print(f'  {fl["handle"]}: {x}', file=sys.stderr)
        sys.exit(2)


# ---------------------------------------------------------------- checking

def cmd_check(args):
    d = os.path.abspath(args.out)
    errors = []

    def err(where, msg):
        errors.append(f'{where}: {msg}')

    def check_source(where, s):
        if not isinstance(s, dict) or not str(s.get('url', '')).startswith(SITE + '/') \
                or not re.fullmatch(r'\d{4}-\d{2}-\d{2}', str(s.get('seen', ''))):
            err(where, f'bad source {s!r}')

    brand = json.load(open(os.path.join(d, 'brand.json')))
    if brand.get('id') != 'naked-and-famous':
        err('brand.json', 'id')
    ident = re.compile(r'^[a-z0-9]+(-[a-z0-9]+)*$')
    n_prod = n_sizes = 0
    ids = set()
    for f in sorted(os.listdir(d)):
        if not f.endswith('.json') or f == 'brand.json':
            continue
        try:
            cut = json.load(open(os.path.join(d, f), encoding='utf-8'))
        except ValueError as e:
            err(f, f'does not parse: {e}')
            continue
        if cut['id'] + '.json' != f or not ident.match(cut['id']):
            err(f, 'cut id does not match the file name')
        if list(cut) != ['id', 'brand', 'name', 'brand_label', 'line', 'products']:
            err(f, f'cut keys {list(cut)}')
        if cut.get('brand') != 'naked-and-famous':
            err(f, 'brand')
        if cut.get('line') not in ('men', 'women', 'unisex', None):
            err(f, f'line {cut.get("line")!r}')
        for p in cut['products']:
            n_prod += 1
            wp = f'{f}:{p["id"]}'
            if not ident.match(p['id']) or not p['id'].startswith(cut['id'] + '-'):
                err(wp, 'product id')
            if p['id'] in ids:
                err(wp, 'duplicate id')
            ids.add(p['id'])
            if list(p) != ['id', 'name', 'url', 'status', 'status_seen', 'fabric', 'sold', 'finish',
                           'list_price', 'shrinkage', 'charts']:
                err(wp, f'product keys {list(p)}')
            if p['status'] not in ('current', 'discontinued') or not re.fullmatch(r'\d{4}-\d{2}-\d{2}', p['status_seen']):
                err(wp, 'status')
            if p['finish'] not in ('raw', 'rinsed', 'washed', None):
                err(wp, f'finish {p["finish"]!r}')
            if {'raw': 'raw', 'rinsed': 'one-washed', 'washed': 'one-washed'}.get(p['finish'], p['sold']) != p['sold']:
                err(wp, f'finish {p["finish"]} with sold {p["sold"]}')
            if p['sold'] == 'one-washed' and p['shrinkage'] and 'Raw Denim Guide' in str(p['shrinkage']):
                err(wp, 'raw-denim shrinkage on a rinsed or washed product')
            fab = p['fabric']
            if set(fab) != {'weight_oz', 'composition', 'sanforized'}:
                err(wp, 'fabric keys')
            if fab['weight_oz'] is not None and not 4 <= fab['weight_oz'] <= 30:
                err(wp, f'weight {fab["weight_oz"]}')
            if fab['composition'] is not None and abs(sum(c['percent'] for c in fab['composition']) - 100) > 0.01:
                err(wp, 'composition does not add up to 100')
            if fab['sanforized'] not in (True, False, None):
                err(wp, 'sanforized')
            lp = p['list_price']
            if lp['currency'] != 'USD' or not 30 <= lp['amount'] <= 1500:
                err(wp, f'list price {lp}')
            check_source(wp + ' list_price', lp)
            if p['shrinkage'] is not None:
                check_source(wp + ' shrinkage', p['shrinkage'].get('source'))
            states = [c['state'] for c in p['charts']]
            if states not in (['as-sold'], ['as-sold', 'after-wash']):
                err(wp, f'chart states {states}')
            by_state = {}
            for c in p['charts']:
                wc = f'{wp}:{c["state"]}'
                check_source(wc, c['source'])
                if c['unit'] != 'in' or c['from_image'] is not False:
                    err(wc, 'unit or from_image')
                if set(c) - {'state', 'source', 'unit', 'from_image', 'sizes', 'estimated_by_brand',
                             'conventions', 'measuring_points', 'tolerance'}:
                    err(wc, 'chart keys')
                if c['state'] == 'as-sold' and 'estimated_by_brand' in c:
                    err(wc, 'estimated_by_brand on an as-sold chart')
                if not c['sizes']:
                    err(wc, 'no sizes')
                keys_seen, tags = set(), []
                for s in c['sizes']:
                    n_sizes += c['state'] == 'as-sold'
                    ws = f'{wc}:{s["label"]}'
                    if set(s) != {'label', 'waist_tag', 'measurements'} or str(s['waist_tag']) != s['label']:
                        err(ws, 'size keys or label')
                    tags.append(s['waist_tag'])
                    m = s['measurements']
                    if not m or set(m) - set(RANGES):
                        err(ws, f'measurements {sorted(m)}')
                    keys_seen.update(m)
                    for k, v in m.items():
                        if not isinstance(v, (int, float)) or isinstance(v, bool) or not RANGES[k][0] <= v <= RANGES[k][1]:
                            err(ws, f'{k}={v!r}')
                    if c['state'] == 'as-sold' and 'waist' in m and not -3 <= m['waist'] - s['waist_tag'] <= 9:
                        err(ws, f'waist {m["waist"]} against tag {s["waist_tag"]}')
                if not {'waist', 'thigh'} <= keys_seen:
                    err(wc, 'no waist or no thigh')
                if tags != sorted(tags):
                    err(wc, 'sizes not in order')
                by_state[c['state']] = {s['label']: s['measurements'] for s in c['sizes']}
            for lab, post in by_state.get('after-wash', {}).items():
                pre = by_state['as-sold'].get(lab, {})
                for k, v in post.items():
                    if k in pre and v > pre[k]:
                        err(f'{wp}:{lab}', f'after-wash {k} {v} is above as-sold {pre[k]}')
    print(f'{n_prod} products, {n_sizes} as-sold sizes, {len(errors)} errors')
    for e in errors[:100]:
        print('  ' + e)
    sys.exit(1 if errors else 0)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    sub = ap.add_subparsers(dest='cmd', required=True)
    f = sub.add_parser('fetch', help='download products.json and the product pages into a cache folder')
    f.add_argument('--cache', required=True, help='folder for downloaded pages (outside the repo)')
    f.add_argument('--delay', type=float, default=3.0, help='seconds between requests (default 3)')
    f.add_argument('--refresh', action='store_true', help='fetch pages again even if they are cached')
    f.add_argument('--handles', help='comma-separated product handles to fetch, instead of all')
    b = sub.add_parser('build', help='write the catalogue files from the cache')
    b.add_argument('--cache', required=True)
    b.add_argument('--out', default=DEFAULT_OUT, help='brand folder to write (default catalogue/naked-and-famous)')
    b.add_argument('--report', help='where to write the report (default CACHE/report.json)')
    c = sub.add_parser('check', help='check the catalogue files against the format')
    c.add_argument('--out', default=DEFAULT_OUT)
    args = ap.parse_args()
    {'fetch': cmd_fetch, 'build': cmd_build, 'check': cmd_check}[args.cmd](args)


if __name__ == '__main__':
    main()
