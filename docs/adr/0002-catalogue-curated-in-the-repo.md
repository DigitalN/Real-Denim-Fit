# The catalogue is curated in the repo, and the app never calls Claude

The brands, cuts, products and sizes are gathered by a Claude prompt that a maintainer runs in this repository. Every number carries the link it came from and the date it was seen, and changes land only through a pull request that's reviewed and checked by CI. The app itself never calls Claude or reads brand sites: every user would need their own API key, and numbers nobody had checked would go straight to people trusting them to pick a size.
