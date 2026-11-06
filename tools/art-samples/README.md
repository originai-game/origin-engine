# Art samples: one scene per family, one frame per style

Stream A7 of the art wave (docs/art.md §2.6). The proof that a style reads
is its family's sample scene with conformed assets, one frame per style,
not the fx-showcase's grey diorama under 37 filters (which stays as the
frame regression it is). Evidence and the per-style table:
`tests/evidence/art/samples/NOTES.md`.

## The scenes

The art-sample scenes (`examples/art-<scene>`) were deleted on 2026-11-06
with the rest of `examples/`, and their committed Metal references with
them. `capture.ts` now draws each style on the project its family's
profile draws (`families.json`: `templates/game-2d` for the 2D families,
`templates/game` for the 3D ones), with the look set to extend the style,
for the art director to read: wave-end's art stage writes the frames under
`target/evidence/wave-end/art/` and compares nothing. The authored art a
content style relies on is kept beside its look, `templates/looks/<slug>/`
(`templates/looks/AUTHORED.md`); `prepare.ts`, `slice.py`, `tiles.py`,
`pack.py`, `make3d.py` and `neon.py` are how that art was made.

## The tools

- `capture.ts [--out DIR] [--size WxH] [--only slug|scene,...] [--mood
  MOOD,...]`: for each style, copies its scene, sets the look to extend the style,
  runs `prepare.ts`, runs `og run --headless` for one frame at 1.5 s
  (`DIR/<slug>.png`) and `og style check --no-frames --json`
  (`DIR/<slug>.style.json`). `--mood bright,dark,...` (and `own`, the
  preset's own mood) runs each mood the sample's look asks for
  (docs/art.md §2.10) into `DIR/<slug>@<mood>.png`. Use the `og` and addon of the commit under
  test (`OG_NATIVE_ADDON`, `OG_ENGINE_PACKAGE`).
- `prepare.ts <project>`: briefs every beat with `artBrief` under the
  project's look and writes each image's import settings (`conform { size
  N }` from the brief's grid; a one-tile tile set for what repeats) and
  `src/cells.ts`.
- `slice.py`: cuts a generated 2 x 2 sheet (hero, tree / crate, lantern)
  into sprites and scales far layers and ground tiles.
- `tiles.py`: the pixel top-down tile set: a generated 4 x 4 sheet sampled
  onto its logical grid, then an atlas of its tiles and three blob47
  terrains (path, pond, ledge) with the tile set's text.
- `pack.py`: a downloaded model (glTF or glb) packed into one glb sized to
  its beat, its maps re-encoded and sized (the cinematic-3d family's
  realistic set, `templates/looks/cinematic-realism/realistic-set/README.md`).
- `make3d.py`: writes the crate and ground glbs around their textures;
  `neon.py`: neon-vector's sources from the base models.
- `catalogue.py <capture-dir> <evidence-dir>`: `look.support` and
  `missing` per style from `verdicts.json` (whether the frame reads, read
  beside the preview) and the check's result; og-taste §12's table
  follows.

## Frames

No frame here is a reference. `bun tools/art-samples/capture.ts --out DIR`
draws every style (or `--only <slug|family>,...`) for the art director to
read; the wave's end runs it into `target/evidence/wave-end/art/`.
