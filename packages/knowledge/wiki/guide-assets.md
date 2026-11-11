<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `assets` guide: Asset queries and references

`og assets` searches the cooked asset inventory for reusable assets, unused outputs,
dependencies, and duplicate content. `og cook` writes the inventory to
`cooked/.assets.ogt`. The sidecar is not part of a web export.

<a id="guide-assets-1"></a>
### `assets §1` Query options

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

<a id="guide-assets-2"></a>
### `assets §2` Reference graph

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

<a id="guide-assets-3"></a>
### `assets §3` Publish pruning

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

<a id="guide-assets-4"></a>
### `assets §4` Cook cache

Native cooks share BLAKE3-keyed actions and contents in
`~/.cache/origingame/cook-v1/ac` and `cas`. Texture and prefracture results use
this store. A prefracture action includes the contents of its external dependencies,
so renaming a source does not invalidate the fracture. Other outputs include
their project inputs and asset id when their bytes depend on names or references.
Video work still uses its existing encoder path and incremental stamps.

`OG_COOK_CACHE` selects another directory; `off` disables the shared store.
The older `OG_TEXTURE_CACHE` variable remains a fallback. The default size cap
is 10 GiB. `OG_COOK_CACHE_MAX_BYTES` changes it. A cook trims the store after
its work at most once an hour, so the store can pass its cap by an hour's
cooks; `og cache gc [--max-bytes N] [--json]` trims it on demand.
Eviction removes disposable cache files, never source assets or cooked outputs.
Missing or corrupt cache content causes a recook.

The project's `.og/cook-index` records file stats and BLAKE3 content hashes.
Unchanged inputs and outputs reuse the previous cook result without reading source
bytes. A source or cooked file written in the same timestamp tick as the cook
that saw it is compared by content on the next cook, so an edit made straight
after a cook is cooked even when it keeps the file's size and times. A changed project is planned again; unchanged scanned sources can read from
the content store. Video projects and models with external dependencies outside
`assets/` are planned again. Removing the index forces a full source check.
Neither cache changes FNV world hashes, schema hashes, in-blob stamps, or the
web service worker's SHA256 hashes.

<a id="guide-assets-5"></a>
### `assets §5` Move source assets

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

<a id="guide-assets-6"></a>
### `assets §6` Format OGT text

`og fmt [path…] [--check] [--json]` rewrites `.ogt` files in the canonical
layout: two spaces per level, one space between tokens, canonical strings,
`\n` line ends. Comments stay on their lines; a trailing comment follows its
tokens after two spaces. Runs of blank lines become one, and blank lines at
the start or end of a block are removed. Entries keep their order, because
order is data.

A path is a file, a directory, or a project. For a project the command
formats `project.ogt` and every `.ogt` under `assets/`; with no path it uses
the current directory. It skips styles, UI screens and components, and 2D
effects, whose bodies are not OGT lines, and installed packs (a directory
with an `extension.json`). A file named outright is always formatted.

The cooked output of a formatted file is unchanged: the command refuses to
write text that reads back as different data. A file the reader refuses is
reported with its line and left as it is.

`--check` writes nothing, lists the files that would change, and exits 1
when there are any. `--json` returns `files`, `check`, `changed`, and
`errors`.

<a id="guide-assets-7"></a>
### `assets §7` Name an OGT file for its type

Every OGT asset has the same extension, so its name can say what it is:
`<name>.<type>.ogt`, with the word on its `type` line (`level1.scene.ogt`,
`hero.flipbook.ogt`). A glob such as `*.scene.ogt` then finds every scene
without opening a file. The name is optional and decides nothing: the `type`
line does, and the asset id is the path as written, name and all.

A cook warns when the last part of a name is a type the file does not cook
as (`menu.prefab.ogt` holding a scene). Any other last part is free
(`sparks.emitter.ogt`). Rename an existing file with `og assets mv`, which
rewrites the references to it.
