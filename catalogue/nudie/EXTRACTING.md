# Extracting Nudie Jeans

Last full run: 2026-11-21, en-US store, USD. Dry jeans only (see "What's in"), men's and women's: 81 products and 5,278 sizes, in 17 cuts (12 men's, 5 women's).

## How to rerun

The tools are in `tools/catalogue/nudie/`. The top of each file explains it. `WORK` is any scratch folder; it caches pages, so a rerun only fetches what's missing.

1. **Cookies (the user does this).** Open a Nudie product page in the built-in browser. Ask the user to click **Size Guide → Update Cookie Settings** and accept. Never click the consent dialog yourself.
2. **Widget key.** Run `widget.js` in that tab. It should return `consent.ok: true` and an `apiKey`. With no key, the consent isn't in place, so go back to step 1.
3. **Products.** Run `python3 tools/catalogue/nudie/fetch_products.py WORK`. It makes about 300 page requests, 2 s apart, so the first run takes about 15 minutes.
   - Then read `WORK/classification.tsv`, especially the rows marked "flagged Dry but never called dry or raw" and any products that are new since the last run.
4. **Measurements.** Run `python3 tools/catalogue/nudie/fetch_virtusize.py WORK --api-key KEY`. Don't commit the key anywhere.
5. **Build.** Run `python3 tools/catalogue/nudie/build.py WORK --seen YYYY-MM-DD`. It writes `catalogue/nudie/` and prints a report: products with no Virtusize data, sizes missing from Virtusize, and name differences.
6. **Spot-check.** Call `window.nudieOpenWidget("<uri>")` (defined by `widget.js`), pick a size and compare the labels with the files.

On 2026-11-21 the chain was rerun on the 2026-11-21 data and reproduced the committed files byte for byte.
- `fetch_products.py` classified 98 known pages exactly as before: 85 dry, 3 of them rinsed.
- Three Virtusize records were fetched fresh, one through the store-product fallback.

## Where the numbers are

- **Only in Virtusize.** Nudie publishes jeans measurements only through the Virtusize widget behind **Size Guide → Open in Virtusize** on each product page. The product data has a `measurementTable` field, but it's `null` for every jeans product. The help centre (`/en-US/help-center/topic/fits-products`) also points to the "Virtual Size Guide".
- **Cookies first (the user does this).** The widget only loads once Nudie's consent manager (consentmanager.net) allows the **Function** category and the **Virtusize** vendor.
  - Open a product page in the built-in browser and ask the user to click **Size Guide → Update Cookie Settings** and accept. Never click the consent dialog yourself.
  - If an earlier run declined, the banner won't reappear. The Size Guide's button is the way back in.
  - Check with `__cmp('getCMPData')`: `purposeConsents.c51` and `vendorConsents.c74506` (Virtusize) must both be `true`.
- **Nudie's consent bug.** The Size Guide component checks vendor **`c74504` (Front Chat)**, not `c74506` (Virtusize), and reads the consent only when it first mounts. After the user accepts, it keeps saying "Please accept cookies to use VirtuSize" until it mounts again.
  - Moving to another product without a full reload fixes it, for example with `window.next.router.push('/en-US/product/<uri>')`. "Open in Virtusize" then appears.
  - It may need two clicks: the first binds the widget, the second opens it.
  - The widget opens *behind* the Size Guide panel, so close the panel.
- **Reading the numbers.** The widget runs in a cross-origin frame (`static.api.virtusize.com`), so its text can't be read from the page, only seen in screenshots, one size at a time.
  - With the user's go-ahead on 2026-11-21, this run read the data the widget itself loads instead: `GET https://services.virtusize.com/product/check?apiKey=<key>&externalId=<id>&version=1`.
  - It was one request per product, 2 s apart, with `<key>` = `window.vs.apiKey` on nudiejeans.com.
  - `externalId` is worked out the way Nudie's component does it: the first part of the first item's SKU (`111946-L28-W24` → `111946`), plus the second part if that isn't a length, cut to 6 characters. It has always equalled `productNumber`.
- **When the check has no sizes.** About a third of the checks return only `data.productDataId`, with no `productData`. The widget then loads `GET https://api.virtusize.com/a/api/v3/store-products/<productDataId>?format=json` (no key needed), so do the same.
  - Check that its `externalId` matches. Its `name` is often abbreviated ("Space Blk Selv"), which is harmless.
  - `validProduct: false` means Virtusize has nothing for the product. The widget can't show sizes either, so the product is left out.
- **What the data looks like.** `productData.sizes[]` is `{name: "30/32", measurements: {waist, frontRise, backRise, hip, thigh, knee, legOpening, inseam}}`, in **millimetres**. We divide by 10 and record cm, which is exactly what the widget displays (its default unit).
  - Spot checks against the widget on 2026-11-21 all matched to the decimal:
    - Lean Dean Dry Apex Selvage W30/L32: 42.8 / 26.6 / 39.6 / 49.5 / 30.3 / 19.7 / 81.2 / 17
    - Gritty Jackson Dry Selvage W30/L32: 42.3 / 26.3 / 38.6 / 50.8 / 30.8 / 21.1 / 81.2 / 18.6
    - Gritty Jackson Dry Selvage W34/L34: everything but the waist, which a tooltip hid
    - Wide Heidi Dry Selvage W30/L32: 43 / 25.5 / 38.2 / 51.5 / 33.4 / 28.3 / 81.2
- **Key mapping:** `hip` → `seat`, `legOpening` → `leg_opening` (the widget says "ankle opening").
- **Sizes:** a size is recorded when it's both in the product's own items (`W30/L32`) and in Virtusize (`30/32`; Regular Alf Dry Selvage writes it `W30/32`). Every size matched.
  - Every product has 65 sizes, W24–38 × L28–36, except Sonic Sue Dry Ecru Waves, which also sells L26 (78 sizes).
  - Wide Heidi Dry Sunburns has L26 in Virtusize but doesn't sell it.
- **Missing numbers:** the four Dry Sunburns products (Gritty Jackson, Lean Dean, Solid Ollie, Wide Heidi) have no `frontRise`/`backRise` in Virtusize, so those keys are left out.

## How Nudie measures

From the guide image linked from the help centre ("How to measure your jeans at home"): the jeans are measured lying flat.
- waist side to side
- front rise and back rise including the waistband
- hip about 16 cm below the waistband
- thigh at the crotch
- inseam
- knee at ½ inseam
- ankle opening

So everything is `flat`, with `thigh: at-crotch`, `knee: half-inseam` and `seat: 16cm-below-waistband`. The values bear it out: W30 has a 42.8 cm waist.

The knee doesn't change between lengths of the same waist, so Nudie probably measured it on one length. No tolerance is stated.

## Product data

Product pages are server-rendered Next.js. Plain `curl` with a browser user agent works. The product object sits in the `self.__next_f.push` stream: find `"uri":"<uri>","productNumber"` and parse the object around it.
- **Price:** `originalPrice.value` (USD). The JSON-LD `offers.price` agrees.
- **Composition:** attribute `pr_composition`, for example "99% Cotton 1% Elastane".
- **Weight:** "13.5 oz." in `prd_bullet_list` / `shortDescription` / `description`. When these disagree, `weight_oz` is `null`. That happened on:
  - Dry Ace Selvage ×4: 14 vs 14.2
  - Dry Heavy ×2: 14.9 vs 15
  - Lean Dean Dry Japan Selvage: 13 vs 13.5
- **Sanforized:** `true` for all. The help centre's denim-knowledge page says: "Yes, all Nudie Jeans are sanforized (pre-shrunk)… no more than 5% after it is washed for the first time."
- **Shrinkage:** the product's own words.
  - Lean Dean Dry 16 Dips: "Account for inseam shrinkage of 2% and 4.5-6% in width." That becomes length 2 and width 5.25 (the midpoint).
  - Otherwise the care line `ci_washText` "May shrink up to 3%" becomes length 3 and width 3. The help centre adds that shrinkage is "primarily in length but also slightly in width", so 3% width is the brand's upper figure, not a typical one.
  - Nine products have no shrink line, so their shrinkage is `null`: Regular Alf Dry Selvage, Loud Larry Dry Streaky, Solid Ollie Dry Truest Selvage, Solid Ollie Dry Black Selvage, Clean Eileen Dry Streaky, and the four Deadstock pairs.
- **Cuts:** attribute `fit` (Lean Dean, Gritty Jackson…).
  - `brand_label` and `line` come from Nudie's own fit records, the Storyblok `global/fits/<fit>` objects in any `/selection/<fit>` page.
  - `gender` (`men`/`women`) becomes `line`.
  - `brand_label` is the fit's chips without the waist and fly ones, for example "Slim Fit, Tapered Leg, Slim Leg Opening".

## What's in

- **Scope:** the user chose dry (raw) jeans only for Nudie on 2026-11-21. Most of Nudie's jeans are pre-washed or distressed, and they would swamp a raw-denim app.
- **Finding the products:** `/en-US/selection/mens-jeans` (192) and `/en-US/selection/womens-jeans` (59) load more as you scroll; `?limit=72` works on the women's page.
  - Then compare with `/en-US/sitemap/products.xml` (614 product pages). It has 29 jeans pages the listings don't show, and 11 of them were dry.
  - The jeans-fit pages with `-pants-` in the URI are duck canvas and other trousers, not jeans.
- **The "Dry" flag isn't enough.** Nudie's `disp_garmentWash` attribute says "Dry" on some washed products, Gritty Jackson Gravel Stone ("The mid-blue wash…") for one. The rule used: **flagged Dry *and* called dry or raw in Nudie's own text** (name, description, bullet list, meta or listing blurb).
  - In: Tight Terry Rinse Twill ("made in dry, power stretch fabric"), Tuff Tony Mystery Selvage ("14 oz. dry rigid selvedge"), the three Static Noise pairs ("a pair of dry jeans"), Tuff Tony / Gritty Jackson Everblue and Rad Rufus / Solid Ollie Everivory (listing blurb: "raw jeans").
  - Rad Rufus Deep Black also passes the rule but has no Virtusize data (see below).
  - Gritty Jackson Everblue is filed under the category "Pants" but sold as "raw jeans", so it's kept.
  - Out (flagged Dry, but never called dry or raw):
    - Gritty Jackson Gravel Stone
    - Lean Dean Echo Tracks
    - Lean Dean Ivory ("treated … with a soak in water")
    - Clean Eileen Burgundy
    - Rad Rufus Everbrown, Sonic Sue Everbrown
    - Lean Dean Everblue, Rad Rufus Everblue, Rad Rufus Everlion
  - The same Everblue fabric appears on both sides of that line. Check it next run.
- **Dry, but no measurements (left out):** Gritty Jackson Dry Deadstock 22, Tuff Tony Dry Deadstock 23, Rad Rufus Deep Black and Regular Alf Dry Black Selvage. Virtusize returns `validProduct: false` for them, so no chart is published.
- **Finish:** `raw` for 78 products. `rinsed` (with `sold: "one-washed"`) for 3 that Nudie names Dry but whose pages say they were rinsed:
  - Gritty Jackson Dry Everblack and Steady Eddie II Dry Everblack ("these jeans have been rinsed")
  - Gritty Jackson Dry Onyx Selvage ("We have treated these jeans to a rinse")
- **Doubtful finish calls:**
  - **Tight Terry Rinse Twill stays `raw`.** "Rinse" is only in the name, and the page says "Jeans made in dry, power stretch fabric" and "not to wash them for 6 months".
  - **Lean Dean Dry 16 Dips** says "Unlike other stretch denim, these are not rinsed". That hints Nudie's other stretch dry denims may be rinsed, but none of their pages says so, so they're `raw`.
- **Not dry, so not in:** "One Wash" pairs (they've been rinsed, so they're like other brands' one-washed raw denim), including Rad Rufus Dusk Selvage One Wash, which is flagged Dry. Also every washed product.
- **Not jeans:** jackets, shirts (Jonas Dry Black Selvage is a shirt), shorts, skirts, dungarees, kids' Tiny Tony / Tiny Turner, and the Tuff Tony pants.
- **Sold out:** the listings can hide pages, and the sitemap is the fullest list of product pages found, so always compare against it. Product pages list every size, sold-out ones included (`stock.available: false`). Everything is recorded as `current`.
