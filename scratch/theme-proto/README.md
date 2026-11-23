# Theme recipe prototype (reference only, never landed)

The prototype behind the UI theme system plan
(https://ampcode.com/artifacts/ui-855f0773-62c3-4be6-bac8-c93e13c9170b). It is
a reference for streams A–H, not code to merge.

- `compile.ts`: resolves a recipe (`extends`, `with` mix-ins, own keys) from
  `themes/<name>.json` and compiles it into a pack under
  `templates/styles/proto-<name>/` (pack.json, art, sounds). The dimension
  tables at its top stand in for `templates/themes/dimensions/`. It also does
  ground registration: inks are solved by OKLCH lightness against each
  scheme's grounds, art faces sampled from the rendered PNGs (`sample.py`).
- `color.ts`: OKLCH↔sRGB, WCAG contrast, the lightness search (`ensure`).
- `art.py`: parametric frame art (material × shape × ornament) on
  templates/tools/make-frame-art.py's distance fields. Needs numpy and Pillow.
- `themes/`: storybook, tactical, candy, pixel, and tactical-arctic
  (`extends: tactical`, three keys).
- `frames/`: the art sheet, gallery rows (columns storybook, tactical,
  tactical-arctic, candy, pixel), every 390×844 fixture frame per theme, and
  `lys-party.png`. CPU renders: layout only; quality frames come from L4.

Run: `bun scratch/theme-proto/compile.ts tactical tactical-arctic`, then
`bun packages/engine/src/node/gallery.ts --packs proto-tactical,...`. art.py
is run with /tmp/tvenv/bin/python (a uv venv with numpy, pillow); change the
path in compile.ts for another machine.
