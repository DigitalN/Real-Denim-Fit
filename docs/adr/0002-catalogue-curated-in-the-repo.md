# The catalogue is curated in the repo, and the app never calls Claude

The brands, cuts, products and sizes are gathered by a Claude prompt that a maintainer runs in this repository. Every number carries the link it came from and the date it was seen, and changes land only through a pull request that's reviewed and checked by CI. The app itself never calls Claude or reads brand sites: every user would need their own API key, and numbers nobody had checked would go straight to people trusting them to pick a size.

## Verified automatically

Nobody checks catalogue numbers by hand, so every change is verified before it reaches a PR:
- every number read from an image is read twice, independently, and kept only when both reads agree
- an independent pass re-checks a sample of products against the brand's live pages
- CI runs consistency checks on every chart: sizes must grow the right way within the brand's tolerance, and no number may sit far off the line through its neighbours

A number that can't be confirmed is left out, never guessed. The PR carries a verification summary, and the user merges it on that summary and green CI.
