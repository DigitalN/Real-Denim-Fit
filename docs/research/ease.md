# Ease research behind the ease table

Researched on 20 November 2026. This is the background for the starting ease table in [PLAN.md](../PLAN.md). In short, the minimums and the position of the knee are well sourced, and the ranges for each room are a best estimate pieced together from the sources below.

## What we found

- **No standard sets ease.** ASTM D6240, ISO 8559-1 and EN 13402 define how to measure the body and how sizes are labelled, but none of them sets ease. The ISO and ASTM texts are behind paywalls and weren't read.
- **No brand publishes a body size chart**, so we can't work out the ease brands designed into their cuts. See [brand-size-charts.md](brand-size-charts.md).
- **The closest reputable body chart is Levi's.** Its size 32 is a body waist of 81.3 cm, seat 97.8 cm and thigh 57.2 cm ([Levi's size chart](https://www.levi.com/NL/en/info/sizechart)). The ease-table calibration test uses it.

## Ease sources

- **Pattern cutting**
  - **Tanya Dove, *Menswear Pattern Cutting*:** 4 cm of seat ease "so you can sit down", which she calls the industry standard, and +2 cm at the trouser waist. This was seen only in an excerpt.
  - **[Fashion Capital trouser block](https://www.fashioncapital.co.uk/tools/how-to-construct-a-basic-trouser-block/):** works out to about +4 cm total at the seat.
  - **[FreeSewing Titan](https://freesewing.eu/docs/designs/titan/options/):** defaults of seat 2%, waist 2% and knee 6%.
  - **Aldrich, *Metric Pattern Cutting for Menswear*:** not verified, because it couldn't be reached.
- **Home-sewing guides from New Mexico State University**
  - **[C-227](https://pubs.nmsu.edu/_c/C227/):** waist ½–1″, hip 2″, thigh 1″.
  - **[C-209](https://pubs.nmsu.edu/_c/C209/):** waist up to 1″, hips at least 2–4″, thigh 1–2″, knee at least 1″.
  - Neither is specific to jeans.
- **Sitting studies.** These are the best sources for the minimums.
  - **Rudolf, Stjepanovič & Cupar, *Materials* 14(18):5124 (2021), 22 women.** Sitting adds 3.23 cm at the hip, 0.82 cm at the thigh, 1.48 cm at the knee and 0.73 cm at the waist. They recommend a minimum ease for woven trousers of 3.5 cm at the hip, 1 cm at the thigh, 1.5 cm at the knee and 1 cm at the waist.
  - **Kim & Nam, *Fashion & Textile Research Journal* 18(1) (2016), women in slacks.** Comfortable hip ease is 0 cm standing and 2 cm sitting in a chair. Comfortable knee ease is 6 cm standing and 10 cm sitting.
  - No study on men turned up.
- **Stretch denim.** Negative ease is common in stretch jeans. Style Arc's Blakley (about 3% elastane) is −4 cm at the hip. Every source found is a women's pattern, and nothing gives figures for men's jeans with 1–2% elastane.

## Leg shape and rise in numbers

- **Rise has no standard.** Front rises at size 32:

  | Cut | Front rise |
  |---|---|
  | J.Crew 484 Slim | 10¼″ |
  | J.Crew 770 Straight | 10¾″ |
  | J.Crew Classic | 11⅝″ |
  | Levi's 501 | 11¼″ |
  | Naked & Famous Weird Guy ("medium rise") | 11.5″, about ⅛″ more per waist inch |

  Raw denim runs higher than this, which is why rise is judged against the catalogue itself ([ADR 0008](../adr/0008-leg-shape-and-rise-from-the-numbers.md)).
- **Taper**, from the [Citizens of Humanity men's specs](https://citizensofhumanity.com/pages/mens-fit-guide), all the way around:
  - Straights narrow by 1–2″ from knee to opening.
  - Slim and tapered cuts narrow by 2.5–3.25″.
- **Bootcut:** the opening is at least as wide as the knee.

## Where the knee falls

- **The measurements.** ANSUR II (Gordon et al., NATICK/TR-15/007) measured 4,082 US Army men:
  - crotch height averages 84.6 cm (SD 4.65)
  - mid-kneecap height averages 48.8 cm
- **What that means.** Crotch to kneecap is about 35.8 cm (14.1″), roughly 0.42 × crotch height. That's about 12.8″ at the 5th percentile and 15.4″ at the 95th, assuming everyone scales in proportion.
- **The brands' knee lines on an average man:**

  | Brand | Knee line | Where it falls |
  |---|---|---|
  | 3sixteen | 14″ | at the kneecap |
  | Iron Heart, Rogue Territory | 13″ | about 1″ above it |
  | Naked & Famous | 12″ | about 2″ above it |

  This is why the body is measured at each brand's knee point ([ADR 0005](../adr/0005-measure-at-every-brands-points.md)).

## How to take body measurements

- **[EN 13402-1](https://en.wikipedia.org/wiki/EN_13402)** (seen via Wikipedia):
  - **Waist:** the natural waist, between the hip bones and the lower ribs.
  - **Hip:** the largest girth around the buttocks.
  - **What to wear:** nothing, or the underwear you'll wear.
- **ANSUR II:** thigh at the fold under the buttock, with the tape square to the thigh.
- **NMSU:**
  - **What to wear:** the underwear you'll wear.
  - **Hip:** fullest point, 7–9″ below the waist.
  - **Thigh:** fullest point.
  - **Knee:** slightly bent.
- **Levi's:** keep the tape "flat against your skin but not tight".
- **Jeans waist:** measure where the waistband will actually sit. Valentina's "trouser waist" is about 5 cm below the natural waist.
