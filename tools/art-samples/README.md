# Art samples: one scene per family, one frame per style

Stream A7 of the art wave (docs/art.md §2.6). The proof that a style reads
is its family's sample scene with conformed assets, one frame per style,
not the fx-showcase's grey diorama under 37 filters (which stays as the
frame regression it is). Evidence and the per-style table:
`tests/evidence/art/samples/NOTES.md`.

## The scenes

`examples/art-<family>` for the eight `look.family` values of the style
catalogue: pixel-side, pixel-topdown, pixel-3d, paint-2d, ink-3d,
shape-3d, toon-3d, cinematic-3d. Each is a small game project (no game
logic) with the same beats: a hero, a stepped tree, a crate, a lantern,
the ground and a far layer. Its `project.ogt` names `looks/sample.ogt`,
which extends the family's first style; `beats.json` lists each asset with
the art brief it was requested with (kind, subject, metres).

- **2D families** (pixel-side, pixel-topdown, paint-2d): every beat is a
  sprite (`src/main.ts`), sized from `src/cells.ts` (the brief's cells).
- **3D families**: every beat is a cooked model (`assets/models/`: `type
  model` text built with og-model §5's generators, and two glbs whose
  textures the cook conforms); `src/scene.ts` places them, `src/main.ts`
  gives the family's camera. pixel-3d stands its hero and tree as
  billboarded pixel sprites; shape-3d takes an untextured field.
- **`styles/<slug>/`**: a style whose direction differs from its family's
  base brings its own sources, laid over `assets/` when it runs (the
  pixel-side and paint-2d styles, neon-vector's black-and-glow models).

## The tools

- `capture.ts [--out DIR] [--size WxH] [--only slug|family,...]`: for each
  style, copies its family's scene, sets the look to extend the style,
  runs `prepare.ts`, runs `og run --headless` for one frame at 1.5 s
  (`DIR/<slug>.png`) and `og style check --no-frames --json`
  (`DIR/<slug>.style.json`). Use the `og` and addon of the commit under
  test (`OG_NATIVE_ADDON`, `OG_ENGINE_PACKAGE`).
- `prepare.ts <project>`: briefs every beat with `artBrief` under the
  project's look and writes each image's import settings (`conform { size
  N }` from the brief's grid; a one-tile tile set for what repeats) and
  `src/cells.ts`.
- `slice.py`: cuts a generated 2 x 2 sheet (hero, tree / crate, lantern)
  into sprites and scales far layers and ground tiles.
- `make3d.py`: writes the crate and ground glbs around their textures;
  `neon.py`: neon-vector's sources from the base models.
- `catalogue.py <capture-dir> <evidence-dir>`: `look.support` and
  `missing` per style from `verdicts.json` (whether the frame reads, read
  beside the preview) and the check's result; og-taste §12's table
  follows.

## Reference frames

The frame of style `<slug>` is `examples/art-<family>/references/<tier>/<slug>.png`
at 1280x720: `bun tools/art-samples/capture.ts --only <family> --out
examples/art-<family>/references/metal` on macrunner records a family's.
