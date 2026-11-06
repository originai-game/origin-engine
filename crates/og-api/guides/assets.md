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

## 3. Publish pruning

`og build --publish` omits unreachable outputs from the exported bundle and its
manifest. The source `assets/` and development `cooked/` trees remain intact.
`og-build.json` records omitted outputs and removed files under `pruning`, with
the computed loads that retained a prefix or every asset.

The project's `publish` block accepts `keep "audio/**" "textures/icons/*"`.
Patterns match asset ids or source paths under `assets/`: `*` stays within one
path segment, `**` crosses segments, and a trailing `/` keeps a folder's contents.
Each match and its dependencies become roots. Multiple `keep` lines are allowed.

A retained streaming pack keeps all its members and dependencies without changing
the pack's bytes. A shared output file remains while any retained id names it.
Escaped loader functions and namespaces retain every asset. Baked reflection
probes remain because their ids depend on runtime placement; this is reported.
The local `.og/publish-assets.json` plan is not shipped.

## 4. Cook cache

Native cooks share BLAKE3-keyed actions and contents in
`~/.cache/origingame/cook-v1/ac` and `cas`. Texture and prefracture results use
this store. A prefracture action includes the contents of its external dependencies,
so renaming a source does not invalidate the fracture. Other outputs include
their project inputs and asset id when their bytes depend on names or references.
Video work still uses its existing encoder path and incremental stamps.

`OG_COOK_CACHE` selects another directory; `off` disables the shared store.
The older `OG_TEXTURE_CACHE` variable remains a fallback. The default size cap
is 10 GiB. `OG_COOK_CACHE_MAX_BYTES` changes it. Cooks trim the store after
work; `og cache gc [--max-bytes N] [--json]` also trims it on demand.
Eviction removes disposable cache files, never source assets or cooked outputs.
Missing or corrupt cache content causes a recook.

The project's `.og/cook-index` records file stats and BLAKE3 content hashes.
Unchanged inputs and outputs reuse the previous cook result without reading source
bytes. A changed project is planned again; unchanged scanned sources can read from
the content store. Video projects and models with external dependencies outside
`assets/` are planned again. Removing the index forces a full source check.
Neither cache changes FNV world hashes, schema hashes, in-blob stamps, or the
web service worker's SHA256 hashes.

## 5. Move source assets

`og assets mv <from> <to> [--project DIR] [--json]` moves one file under
`assets/` and its `type import` sidecar. Paths accept the `assets/` prefix.
The destination and its import sidecar must not exist.

The command rewrites exact matching string literals in asset OGT documents,
`project.ogt`, and game TypeScript, including `#subasset` suffixes. It preserves
comments, regular expressions, and partial filename matches. All matching
literals are rewritten, including strings outside asset fields.
Computed loads remain unchanged and produce warnings when their prefix covers
the moved file. Source-relative dependencies inside models and map formats
are not rewritten; moves between directories report that limitation.

Parsing and path checks finish before files change. Writes are staged under
`.og/asset-move`; a failed installation restores originals when possible.
If recovery cannot finish, the command retains backups and reports their path.
An interrupted move's journal must be recovered before another move starts.
The command does not recook. Run `og cook` afterward to refresh outputs and
the inventory. `--json` returns `from`, `to`, `rewritten`, and `warnings`.
