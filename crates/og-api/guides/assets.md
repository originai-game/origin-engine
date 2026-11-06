# Asset queries and references

`og assets` searches the cooked asset inventory for reusable assets, unused outputs,
dependencies, and duplicate content. `og cook` writes the inventory to
`cooked/.assets.ogt`. The sidecar is not part of a web export.

## 1. Query options

`og assets [--kind KIND] [--group GROUP] [--unused] [--refs] [--by] [--dupes [--near N]] [--json] [text]`

- `text` matches an asset id or source path, ignoring case.
- `--kind` selects an output kind, such as `texture`, `sound`, `model`, or `scene`.
- `--group` selects a streaming group.
- `--unused` selects outputs unreachable from the recorded roots.
- `--refs` includes each output's dependencies.
- `--by` includes the assets and game source files that reference each output.
- `--dupes` lists pairs that share exactly the same cooked file.
- `--dupes --near N` also lists texture or sound pairs with fingerprint distance at most `N`.
  `N` is a Hamming distance from 0 to 64. There is no default similarity threshold.
- `--json` returns `assets`, `duplicates`, and `computed` arrays. Fingerprints are hexadecimal strings.
- `--project DIR` selects a project. The default is the current directory.

The cook records a 64-bit dHash for textures and a 64-bit loudness-envelope hash
for sounds. Near matches are candidates for inspection, not evidence that two
assets are interchangeable. Flat images and steady sounds can have the same fingerprint.

## 2. Reference graph

Edges include dependencies resolved by the cook, asset paths in asset documents,
and matching string literals in game TypeScript. The roots include project asset
references, placed scenes, and TypeScript asset literals. Comments and regular
expression literals do not create edges.

A computed `og.load` path retains all assets under its static prefix. A path
without a static prefix retains every asset. The command reports the source line
and retained prefix. Code that cannot be parsed also retains every asset and
reports the parsing failure. The graph is conservative; `unused` is not permission
to delete source files.

The sidecar describes the last cook. A subsequent cook refreshes the graph after
TypeScript changes even when every cooked blob is unchanged.
