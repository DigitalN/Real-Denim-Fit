---
description: Refresh one brand in the catalogue (or add a new brand), verify every number, and open a PR for it
argument-hint: "<brand id or name, e.g. iron-heart>"
---

Update the catalogue for one brand: **$ARGUMENTS**. If no brand was given, ask which one and stop.

Running `/update-catalogue` is the user's go-ahead for these: make a branch, commit, push it and open a pull request for this one brand. Nothing else gets pushed or merged.

**Nobody checks the numbers by hand.** The user merges the PR after reading its verification summary, so this command has to do the checking itself:
- every number read from an image is read twice, independently (step 3)
- an independent sample check compares numbers against the live pages (step 6)

Every number here helps someone choose jeans they can't try on, so **accuracy matters more than finishing**. A number that can't be confirmed is left out, and the PR says so.

**Sub-agents:** the checks below use sub-agents that never saw your extraction. Start each one fresh, at effort `xhigh`, since it's checking work, with only what the step says to give it. Never pass on your own readings, the catalogue files or this conversation. Compare readings with a script (`python3 -I`), never by eye.

**Working files:** keep everything for this run in `<scratchpad>/catalogue-<brand-id>/`: saved images, each read, the sample list, the checker's readings and the comparison results. None of it is committed.

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

Use the built-in browser pane to read brand pages, because many of their charts only appear once the page's scripts run. Use `get_page_text` or `read_page` to read text.

**For each product on the brand's site:**

- Read its size chart, fabric (weight, composition, sanforized or not), whether it's sold raw or washed, shrinkage, and list price.
- Record each number exactly as the brand publishes it, in its unit, with no converting or rounding.
- Set every `source` and `seen` to the page you read it on and today's date.
- Set `status: "current"` and `status_seen` to today.
- If a chart is a picture, go through step 3 before you write any of its numbers into the catalogue, and set `from_image: true` on it.
- Add new products and cuts the brand now sells.
- If a product has no chart at all, leave it out and add it to the skipped list in `EXTRACTING.md`.

**Products that are gone:** keep anything that's in the catalogue but no longer on the brand's site. Set its `status` to `"discontinued"` and `status_seen` to today. Leave its numbers as they are. Check the brand's own search, and the cut's other colourways, before you call a product gone.

**Exchange rates:** refresh `catalogue/exchange-rates.json` from the European Central Bank's daily file:

1. Download it into your scratchpad: `curl -fsSL https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml -o <scratchpad>/ecb.xml`.
2. Read it with `python3 -I`. `date` is the file's own `time` attribute, not today. `rates` has every currency in the file, each rate exactly as written there, as units per 1 EUR.
3. Keep `source` and `base` as they are.

A script reads these rates straight from the ECB's own file, so step 6 doesn't check them again.

## 3. Read every image chart twice, independently

1. **Save the image** to the working folder. Find its address with `read_page`, then download it with `curl -fsSL <image URL> -o <working folder>/images/<product-id>-<state>.<ext>`. If the chart can't be saved as a file, write down the image's URL instead.
2. **Read A, yours.** Transcribe the whole table to `reads/<chart>-a.json`, as soon as you've read it:
   - every row label and column heading exactly as printed
   - every cell as the text printed, such as `"12 1/2"` or `"31.7"`
   - an empty string for an empty cell
3. **Read B, independent.** Start a sub-agent and give it only:
   - the saved image (or its URL)
   - the path to write its JSON to
   - the instruction to transcribe the whole table in the same shape, writing `"?"` for any cell it can't read for certain

   Don't tell it what you read, what the brand is or what values to expect.
4. **Compare A and B cell by cell** with a script. Keep a cell where both reads give the same text.
5. **Read C, for cells that disagree.** Start another fresh sub-agent. Give it the image, plus only the row label and column heading of each disputed cell, never the values. Keep a cell where C gives the same text as A or as B.
6. **Leave out every other cell:** no two reads agree, or a read said `"?"`. Don't write it into the catalogue. Add it to `EXTRACTING.md` under "Left out: image reads disagreed", with the product, size, measurement and image URL.
7. Write only the kept cells into the cut file, then convert the printed text to the number the format wants, such as `12 1/2` → `12.5`. That's a change of notation, not of value.

Keep a tally for the PR: cells read, agreed at once, settled by read C, and left out.

## 4. Never guess a number

- If a measurement isn't published, leave its key out. Never write `null` or `0`, and never work it out from other sizes, other products, other cuts or a retailer.
- If the fabric facts aren't stated (sanforized, composition, weight), use `null`.
- If a chart doesn't say whether it's before or after washing, it's `"as-sold"`.
- If something doesn't fit the format, stop and ask the user rather than bending the data to fit.

## 5. Cookie banners and sign-ins: the user does these

Some sites only show their measurements after you accept cookies. Nudie is one of them.

- When a page shows a cookie or consent banner that hides what you need, **stop**. Ask the user to accept it themselves in the browser pane, and wait for them to say it's done. Sub-agents use the same browser pane, so they get past the banner too.
- **Never** click, dismiss or hide a consent banner yourself, and never set cookies or remove the banner with JavaScript. Tell every sub-agent the same.
- The same goes for sign-ins, newsletter pop-ups that ask for an email address, CAPTCHAs and age gates: stop and ask the user. Never fill them in.
- Note the cookie wall in the brand's `EXTRACTING.md`.

## 6. Independent sample check

Before opening the PR, a sub-agent that didn't do the extraction re-reads part of the brand from the live pages.

1. **Pick the sample with a script**, at random, and save it as `sample.json` in the working folder.
   - At least 10% of this brand's current products (rounded up), and enough of them to cover at least 20 sizes. If the brand has fewer than 20 sizes in all, take every size.
   - Include at least one product with an image chart, and at least one product added or changed in this run, where there are any.
   - For each product, the sample lists: its URL, which charts (`as-sold`, `after-wash`) and the size labels. It holds no numbers.
2. **Start the checker:** a fresh sub-agent. Give it `sample.json`, the rules in "Record each number exactly as the brand publishes it" and step 4, the cookie rules in step 5, and a path to write its readings to. Tell it not to open `catalogue/` or any file in the working folder except `sample.json`.
   - It reads every number for those products from the live pages: each listed size's measurements, plus fabric weight, composition percentages, shrinkage figures and list price.
   - For an image chart, it saves the image again and reads it itself.
   - It leaves out what isn't published, and writes `"?"` for anything it can't read for certain.
3. **Compare with a script** every number the checker read against the cut files, and every number the cut files hold for the sampled sizes against the checker's. A value that differs, or is present on one side only, is a mismatch. So is a `"?"`, unless that number was already left out of the catalogue.
4. **For each product with a mismatch:**
   - Re-extract it from scratch from its live page, including step 3 for any image chart. Don't start from your old reading.
   - Then check that product again with another fresh checker.
   - It passes when the new extraction and the new checker agree on every number.
5. **If any product still has a mismatch after one re-extraction, stop.**
   - Don't open the PR.
   - Commit your work on the branch, but don't push it.
   - Tell the user which numbers disagree, what each reading said, and the page links.

Keep the tally for the PR: products and sizes sampled, numbers compared, matches on the first pass, products re-extracted, and the final match rate.

## 7. Run the tests

```sh
cargo test --workspace
```

The tests include catalogue validation: every number has a link and a date, numbers fall in sensible ranges, no after-wash number is bigger than its as-sold number, and every product has at least a waist and a thigh.

- **A failure that points at a mistake you made** (a typo, a missing source, a number in the wrong key): fix the data and run the tests again. If you changed any numbers, run step 6 again for those products.
- **A failure where the brand really publishes something odd**, such as a number outside the sensible range: don't change the number and don't change the tests. Tell the user what the brand shows and ask what to do.

## 8. Commit, push and open the PR

**One brand per PR.** Only this brand's folder and `catalogue/exchange-rates.json` change.

1. Update `EXTRACTING.md`:
   - what you learned, and any quirks
   - what was skipped or left out, and why
   - "Last full run: <today>"
2. Run `git status` and check that nothing outside those files changed.
3. Commit: `Update <Brand name> catalogue (<YYYY-MM-DD>)`, or `Add <Brand name> to the catalogue` for a new brand.
4. Push: `git push -u origin HEAD`.
5. Open the PR: `gh pr create --base main --title "<same as the commit>" --body-file <working folder>/pr.md`, with a body like this:

```markdown
## Verification

- **Numbers in this update:** <N>, of which <N> were read from text and <N> from images.
- **Image numbers:** every one was read twice, independently. <N> agreed at once, <N> were settled by a third read, and <N> were left out because the reads disagreed.
- **Sample check:** an independent re-read of <N> of <N> products (<N>%), covering <N> sizes and <N> numbers, from the live pages on <date>.
  - First pass: <N> of <N> matched (<N>%).
  - Re-extracted after a mismatch: <products, or "none">.
  - Final: <N> of <N> matched (100%).
- **Left out, and why:**
  - not published by the brand: <N> (<short list>)
  - image reads disagreed: <N> (<product, size, measurement>)
  - products with no chart: <N> (<list>)
- **Exchange rates:** read by script from the ECB reference rates of <date>.

## What changed

- <N> products checked across <N> cuts, all seen <date>.
- **Added:** <products or cuts, or "none">
- **Discontinued:** <products no longer on the brand's site, or "none">
- **Changed numbers:** <product, size, measurement: old → new; or "none">
- **Prices:** <changes, or "unchanged">
```

- Every count comes from the scripts' results, not from memory.
- Give the user the PR's link and the one-line result of the verification, for example: "Verified: 412 numbers, sample check 100% after one re-extraction, 3 left out." If anything was left out, say so plainly.
