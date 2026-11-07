# Downloadable effect packs

Each `<pack>/` here (an `extension.json` of `kind` `"vfx"`, docs/vfx-packs.md
§1) ships with every engine release as `<version>/vfx/<pack>.tar.gz`, not in
the engine archives, and a game installs it with `og vfx add <pack>`
(docs/vfx-packs.md §5, docs/release.md §1). Plain Git files, no LFS; nothing
under a pack's `preview/` is archived. The few packs every engine carries are
in `packages/engine/extensions/` instead; a pack is in one place, never both.

| Pack | Effects |
|---|---|
| (none yet) | |
