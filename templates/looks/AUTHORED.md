# Authored art kept with its look

The catalogue's content styles need authored art; conformance and frame
effects do not invent illustration. The preview's art stays beside its
checked preset under the plain look slug:

- `pixel/art/`, `cartoon/art/`, `anime/art/`, `neon/art/`: side-view
  hero, tree, crate, lantern, slime, gem, bush, heart, platform, ground
  tile set (`tiles`), tree line (`mid`) and far valley (`backdrop`).
- `pixel/top-down/art/`, `cartoon/top-down/art/`, `neon/top-down/art/`:
  the top-down drawing of the same beats, with floor, path and cliff
  tile sets (`grass`, `path`, `wall`) instead of the side-view layers.
  Top-down is a view, not a variant or another preset.
- `neon/models/`: the 3D beats built as glowing vector geometry, in
  checked `file.model` TypeScript.

`preview/preview.json` selects the scene and art for each view.
`preview/2d/tools/art.ts` prepares generated art to the look's contract.
Binary art is pinned in `templates/og-assets.json`, never committed.
The cook reads only the checked presets, built-in assets and mood layers;
the preview scene and its authored art are not gameplay templates.
