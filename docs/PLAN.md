# Real Denim Fit: the plan

What we agreed in the design session of 20 November 2026. Words in bold are defined in [GLOSSARY.md](GLOSSARY.md). The reasons behind the bigger choices are in [docs/adr](adr/), and the facts they rest on are in [docs/research](research/): [what brands publish](research/brand-size-charts.md) and [the ease research](research/ease.md).

## What it is

A free desktop app for macOS and Windows. You measure your body, choose your **desired fit**, and optionally add a **reference pair**. It shows which raw denim **products**, in which **size**, are worth looking into, and how close each one comes. It never says where to buy them.

- **Licence:** GPL-3.0, the same as the user's other projects.
- **Platforms:** one universal macOS app (Apple silicon and Intel), and 64-bit Windows, which also runs on ARM Windows laptops.
- **Built with:** Tauri 2. The matching, catalogue and updater are Rust, and the screens are HTML and CSS ([ADR 0001](adr/0001-tauri-for-the-app.md)).
- **Users:** one person per app. English only. Follows the system's light or dark mode.

## What the user enters

All of this stays on the computer. Nothing is ever sent anywhere.

### Body measurements (always)

A guide takes you through these one per screen. Each screen has a drawing, two or three lines on where the tape goes, and the most common mistake. Use a soft tape, snug without pressing in, over thin underwear.

- **Waist:** where you wear your jeans, which isn't always your natural waist.
- **Seat:** around the fullest part.
- **Thigh:** around the fullest part at the top, and again 2.5 cm (1″) lower.
- **Leg:** around the leg at 30.5, 33 and 35.5 cm (12″, 13″, 14″) below the crotch, and halfway down the inseam. Strips of tape mark the spots. These are the brands' **knee** points ([ADR 0005](adr/0005-measure-at-every-brands-points.md)).
- **Inseam you want:** from the crotch to where you want the hem. The guide explains adding length for cuffs or stacking.

Once everything is entered, a single form lets you change any number.

### Desired fit (always)

- **Room:** snug, regular or roomy, through the seat and thigh.
- **Leg shape:** tapered, straight, wide or any.
- **Rise:** low, mid, high or any.

The app shows the usual name for the combination, such as "slim tapered". The waist isn't a choice: it should fit your waist.

- **Importance:** each measurement counts from 1 (least) to 5 (most), starting at 3, set with five dots per measurement.

### Reference pairs (optional, as many as you like, each with a name)

- **Measured by you.** Button it, lay it flat and smooth it without stretching. Then measure flat:
  - waist (the app doubles it), front rise, back rise and seat
  - thigh at the crotch and 2.5 cm (1″) below
  - knee at 30.5, 33 and 35.5 cm (12″, 13″, 14″) below the crotch, and halfway down the inseam; the tape stays on the leg and you read it at each mark
  - leg opening and inseam

  You can skip any measurement.
- **Or picked from the catalogue** by brand, cut, fabric and size, including a pair you've only tried on in a shop. It uses that size's **after-wash measurements** and is marked "from the brand's chart".
- **Feel** for each measurement: too tight, just right or too loose (too short or too long for inseam and rise). Each "too" becomes a **nudge** of ¼″ (0.5 cm) in that direction.
- **Free-text notes**, for you only. They're never used in matching.
- **Your pair's own ease:** with a body and a pair both entered, the app shows the pair's **ease** at each point, such as "+6.5 cm at the thigh, which is regular". It suggests the **room** that matches.

### Settings

- **Units:** centimetres by default, or inches in ⅛″ steps. You can switch any time.
- **Currency:** taken from the system's region by default.
- **Try-on brands:** brands you've marked as ones you can try on locally.

## How matching works

1. **Every product is judged after its first wash** ([ADR 0004](adr/0004-matches-judged-after-first-wash.md)):
   - the brand's after-wash chart if it has one
   - the chart as it is if the product is sold already washed
   - otherwise the chart adjusted by the brand's stated **shrinkage**
   - unknown shrinkage on a product that will shrink is marked "shrinkage unknown" and ranked after the rest
   - products with no measurements are left out

   Waist stretch from wear isn't added to the numbers. Rigid products carry the note "the waist usually stretches about ½–1″ with wear".
2. **A target at each measuring point** ([ADR 0006](adr/0006-body-first-reference-pair-optional.md)). Every **target** is a range with an ideal middle:
   - **Where a reference pair has a reading:** the reading ±¼″ (0.5 cm), shifted by its nudges.
   - **Otherwise:** your body plus the ease your room calls for there, from the **ease table** below. Each brand's knee is compared with your leg reading at that brand's knee point.
   - **Inseam:** at least the inseam you want, or, where a reference pair has an inseam reading, at least that reading shifted by its nudge. Longer is fine, because raw denim gets hemmed. Shorter scores like passing a target's edge.
   - **Leg opening, front rise and back rise:** only from a reference pair. Without one, leg shape and rise act as filters instead (step 4).
   - **Every measurement** keeps its target whatever its **importance**; importance only changes how much it counts in closeness (step 5).
3. **Hard limit.** Anything tighter than your body plus **minimum ease**, at any point that can be compared, is left out entirely, at every importance.
4. **Leg shape and rise** are worked out from each size's own numbers ([ADR 0008](adr/0008-leg-shape-and-rise-from-the-numbers.md)):
   - Leg shape from knee minus leg opening, flat: tapered is 3 cm or more, straight is 1–3 cm, wide is under 1 cm.
   - Rise is low, mid or high by which third its front rise falls in among every product in the catalogue at the same waist.
   - Only your chosen leg shape and rise are shown, with a switch to show the rest.
5. **Closeness** ([ADR 0009](adr/0009-closeness-judged-by-the-worst-measurement.md)):
   - Each measurement scores 100% at the middle of its target and 80% at its edges.
   - Beyond the edges it loses 10% for every ⅛″ (3 mm) flat, down to 0%.
   - Each measurement's shortfall is 100% minus its closeness, and counts in proportion to its **importance** divided by the highest importance currently set.
   - A match's closeness is 100% minus its largest weighted shortfall, and ties go to the smaller weighted average. With every importance equal, that's the worst measurement's closeness and the higher average.
   - The closeness shown for each measurement stays unweighted.
   - A measurement the brand doesn't publish shows "not published". When two matches are equally close, the one compared on more measurements comes first.
   - Matches from a brand that doesn't say where it measures are marked "approximate".
6. **Stretch products** have every ease range 2 cm lower, measured around the body (1 cm flat), never below their minimum ease, which is 0.
7. **Brands sold in lengths** (Nudie's W32 L30/L32/L34): the app picks the shortest length that's at least your inseam.

### The starting ease table

Ease is garment minus body, shown flat in inches here, for rigid denim after its first wash. It's stored in the repo (`crates/fit-core/data/ease-table.json`) with a source on every number, each marked "well sourced" or "best estimate". Only the minimums are well sourced: they come from a 2021 study of sitting in woven trousers (women only; no study on men turned up) and a pattern-cutting text. Changes go through a PR. Before the first release, tests compare every cut with Levi's published body chart for its size. If the cuts brands call slim mostly land in regular, the snug and regular ranges move up.

| Flat, inches | Snug | Regular | Roomy | Minimum |
|---|---|---|---|---|
| Waist | +0.2 to +0.6 | +0.2 to +0.6 | +0.2 to +0.6 | +0.2 |
| Seat | +0.8 to +1.2 | +1.2 to +1.8 | +1.8 to +2.8 | +0.7 |
| Thigh | +0.6 to +1.0 | +1.0 to +1.6 | +1.6 to +2.4 | +0.2 |
| Leg at the brand's knee point | +0.8 to +1.4 | +1.2 to +2.0 | +2.0 to +3.0 | +0.3 |

## The results screen

- **One row per cut,** showing its closest product and size. Each row shows the closeness and how far off each measurement is in your units, for example "86% · thigh +⅛″, knee −¼″", coloured from close to far. Opening a row lists the cut's other products, closest first, then its other sizes.
- **Without a reference pair,** a one-line note sits at the top: "Matched from your body and typical ease. Adding a pair you know makes this sharper."
- **Filters:**
  - brand and price, where the **list price** is converted to your currency and shown as "about"
  - men's and women's cuts, both ticked by default, following each cut's **line**
  - try-on brands only
  - **fabric weight** range
  - "Sanforized only", off by default
  - "Include washed denim", off by default, which adds products whose **finish** is washed (stonewashed, vintage wash, faded); raw and rinsed products always show
  - "No stretch", off by default
  - "Current products only", on by default
  - leg shape and rise
- **Each product shows:**
  - fabric, weight and composition, including elastane %
  - **sanforized** or not
  - **as-sold** and after-wash measurements side by side; after-wash figures worked out from shrinkage are labelled "estimated from the brand's shrinkage"
  - the brand's **tolerance**
  - "no longer on the brand's site" where that applies
  - for each number, a link to the brand page it came from and the date it was seen
- **What it never shows:** product photos ([ADR 0007](adr/0007-no-product-photos.md)), shops, stock or sale prices ([ADR 0003](adr/0003-no-shopping-information.md)).

## The catalogue

**Brands for the first build:** Iron Heart, Naked & Famous, 3sixteen, Nudie, Momotaro and Rogue Territory.

**What's included:** every denim jeans product whose page is on the brand's site, including sold-out ones, from men's, women's and unisex lines, and in every finish: raw, rinsed and washed. Nudie is limited to its dry jeans. Anything that isn't denim is left out: duck canvas, herringbone, corduroy, and so on.

**What's kept for every number:**
- the source link and the date it was seen
- the measuring point
- whether it was measured flat or all the way around
- the unit
- the brand's tolerance
- whether it was read from an image

**Updating it:** run `/update-catalogue <brand>`, a Claude Code command in this repo ([ADR 0002](adr/0002-catalogue-curated-in-the-repo.md)). The same command adds a new brand.
- **One brand per run, one PR per brand.**
- **Nudie:** the run stops in the browser pane and asks you to accept Nudie's cookies, because its measurements only appear after that.
- **Verified automatically, not checked by hand:** every number read from an image is read twice, independently, and kept only when both reads agree. A separate pass re-checks a sample of products against the brand's live pages. A number that can't be confirmed is left out, never guessed. The PR carries a verification summary, and you merge on that.
- **Discontinued products:** a product that's gone from the brand's site stays, marked "no longer on the brand's site".
- **Exchange rates:** fixed from the European Central Bank's daily reference rates at update time.
- **Data in the app:** the catalogue and the ease table are built into the app, so new data arrives as a new release.

**CI checks on every catalogue PR:**
- every number has a link and a date
- numbers fall in sensible ranges
- no after-wash number is bigger than its as-sold number
- every product has at least a waist and a thigh
- within each chart, the waist grows with the size, allowing for the brand's tolerance
- seat, thigh, knee, leg opening and rises don't shrink between neighbouring sizes by more than the tolerance plus ¼″
- no number but the inseam sits more than 3 × the tolerance off the line through its neighbouring sizes (inseams step up in bands for bigger sizes)

## Updates

- **When it checks:** every time the app opens, it asks GitHub for the latest published release and waits a few seconds at most.
- **When there's a newer one:** it shows "Updating to 1.3…", installs it and reopens, before you can enter anything.
- **When you're offline or GitHub is slow:** it opens normally.
- **Verification:** every download is checked against Tauri's own update signing key. The private key lives in the repository's GitHub secrets, with a backup outside GitHub ([ADR 0001](adr/0001-tauri-for-the-app.md)).
- **Network use:** checking for updates is the app's only network access. No analytics and no crash reports.

## Releases

- **The Release workflow** in GitHub Actions is run by hand with a version number. It builds:
  - a universal macOS `.dmg`
  - a Windows installer that doesn't ask for admin rights
  - the update file

  It puts them all in a **draft** release.
- **`/release`** is a Claude Code command:
  1. Claude picks the version number from what changed.
  2. It writes the release notes: what's new, plus install steps that explain the one-time warning.
  3. It runs the workflow.
  4. It checks that the draft has both downloads and the update file.
  5. It asks you once, then publishes.
- **No paid signing** on either system for now, so first launch shows a one-time warning. macOS says it can't verify the app (click **Open Anyway** in Privacy & Security). Windows says "Windows protected your PC" (click **More info**, then **Run anyway**). Revisit this if the warning puts people off.

## Not in the first version

- profiles for more than one person
- shops, stock or photos
- paid code signing
- scheduled catalogue updates

## What the build needs from you

- Approve storing the update signing key in GitHub secrets, and keep the backup of the private key.
- Accept Nudie's cookies when the first catalogue run asks.
- Merge each catalogue PR on its verification summary and green CI.
- Confirm the first release before it's published.
