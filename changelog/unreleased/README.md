# Unreleased changes

One JSON file per merge: what it adds, breaks or fixes for a game, in the
shape `scripts/changelog.ts` describes. Name it after your branch and
topic (`ui-render-rive-images.json`), so two branches never write the
same file. Never edit `packages/engine/changelog.json` by hand:
`scripts/release.sh fold` and `release.sh bump` fold these files into it
and delete them (docs/release.md §3). The quick gate checks their shape.
