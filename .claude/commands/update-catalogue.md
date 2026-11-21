---
description: Refresh one brand in the catalogue (or add a new brand) and open a PR for it
argument-hint: "<brand id or name, e.g. iron-heart>"
---

Update the catalogue for one brand: **$ARGUMENTS**. If no brand was given, ask which one and stop.

Running `/update-catalogue` is the user's go-ahead for these: make a branch, commit, push it and open a pull request for this one brand. Nothing else gets pushed or merged.

Every number here helps someone choose jeans they can't try on, so **accuracy matters more than finishing**. If you can't read a number for certain, leave it out and say so in the PR.

## 1. Read first

- `docs/catalogue-format.md`: the format and its rules. Follow it exactly.
- `GLOSSARY.md`: the words.
- If `catalogue/<brand>/` exists:
  - its `EXTRACTING.md`, which says where the charts are and what's tricky
  - `brand.json`
  - every cut file
- **For a brand that isn't in the catalogue yet:**
  - Research it on **the brand's own site only**: its size charts, its "how we measure" page, its tolerance, its shrinkage notes and its currency. Never use retailers, forums or reviews. Their numbers aren't the brand's.
  - Create `catalogue/<brand-id>/` with `brand.json` (including `measuring_points` and `conventions`), one file per cut, and an `EXTRACTING.md` for the next run.
  - If the brand doesn't say where it measures, that point is `"unknown"`.
  - If it measures somewhere the format has no name for, such as a knee point 11″ below the crotch, stop and tell the user. The format, the measuring guide and the app all need that point first.

Make a branch from the latest `main` before changing anything:

```sh
git fetch origin
git switch -c catalogue/<brand-id>-<YYYY-MM-DD> origin/main
```

## 2. Fetch the current charts and update the files

Use the built-in browser pane to read brand pages, because many of their charts only appear once the page's scripts run. Use `get_page_text` or `read_page` to read text. Use a screenshot or zoom only for charts that are images.

**For each product on the brand's site:**

- Read its size chart, fabric (weight, composition, sanforized or not), whether it's sold raw or washed, shrinkage, and list price.
- Record each number exactly as the brand publishes it, in its unit, with no converting or rounding.
- Set every `source` and `seen` to the page you read it on and today's date.
- Set `status: "current"` and `status_seen` to today.
- Set `from_image: true` on any chart read from a picture. Write down every number you read from it, for the PR (step 6).
- Add new products and cuts the brand now sells.
- If a product has no chart at all, leave it out and add it to the skipped list in `EXTRACTING.md`.

**Products that are gone:** keep anything that's in the catalogue but no longer on the brand's site. Set its `status` to `"discontinued"` and `status_seen` to today. Leave its numbers as they are. Check the brand's own search, and the cut's other colourways, before you call a product gone.

**Exchange rates:** refresh `catalogue/exchange-rates.json` from the European Central Bank's daily file:

1. Download it into your scratchpad: `curl -fsSL https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml -o <scratchpad>/ecb.xml`.
2. Read it with `python3 -I`. `date` is the file's own `time` attribute, not today. `rates` has every currency in the file, each rate exactly as written there, as units per 1 EUR.
3. Keep `source` and `base` as they are.

## 3. Never guess a number

- If a measurement isn't published, leave its key out. Never write `null` or `0`, and never work it out from other sizes, other products, other cuts or a retailer.
- If you can't read a digit in an image for certain, leave that measurement out. List it in the PR as unreadable.
- If the fabric facts aren't stated (sanforized, composition, weight), use `null`.
- If a chart doesn't say whether it's before or after washing, it's `"as-sold"`.
- If something doesn't fit the format, stop and ask the user rather than bending the data to fit.

## 4. Cookie banners and sign-ins: the user does these

Some sites only show their measurements after you accept cookies. Nudie is one of them.

- When a page shows a cookie or consent banner that hides what you need, **stop**. Ask the user to accept it themselves in the browser pane, and wait for them to say it's done.
- **Never** click, dismiss or hide a consent banner yourself, and never set cookies or remove the banner with JavaScript.
- The same goes for sign-ins, newsletter pop-ups that ask for an email address, CAPTCHAs and age gates: stop and ask the user. Never fill them in.
- Note the cookie wall in the brand's `EXTRACTING.md`.

## 5. Run the tests

```sh
cargo test --workspace
```

The tests include catalogue validation: every number has a link and a date, numbers fall in sensible ranges, no after-wash number is bigger than its as-sold number, and every product has at least a waist and a thigh.

- **A failure that points at a mistake you made** (a typo, a missing source, a number in the wrong key): fix the data and run the tests again.
- **A failure where the brand really publishes something odd**, such as a number outside the sensible range: don't change the number and don't change the tests. Tell the user what the brand shows and ask what to do.

## 6. Commit, push and open the PR

**One brand per PR.** Only this brand's folder and `catalogue/exchange-rates.json` change.

1. Update `EXTRACTING.md`: what you learned, any quirks, what was skipped and why, and "Last full run: <today>".
2. Run `git status` and check that nothing outside those files changed.
3. Commit: `Update <Brand name> catalogue (<YYYY-MM-DD>)`, or `Add <Brand name> to the catalogue` for a new brand.
4. Push: `git push -u origin HEAD`.
5. Open the PR: `gh pr create --base main --title "<same as the commit>" --body-file <scratchpad>/pr.md`, with a body like this:

```markdown
## What changed

- <N> products checked across <N> cuts, all seen <date>.
- **Added:** <products or cuts, or "none">
- **Discontinued:** <products no longer on the brand's site, or "none">
- **Changed numbers:** <product, size, measurement: old → new; or "none">
- **Prices:** <changes, or "unchanged">
- **Skipped:** <products with no chart, and why>
- **Exchange rates:** ECB reference rates of <date>.

## Numbers read from images

Check each one against its picture.

| Product | Chart | Size | Measurement | Value | Image |
|---|---|---|---|---|---|
| … | as-sold | 32 | thigh | 12.5 in | <image URL> |

Unreadable, so left out: <list, or "none">
```

- List **every** number read from an image, one per row. If there were none, say "None: every number came from text on the page."
- Then give the user the PR's link, and tell them which numbers most need checking by eye.
