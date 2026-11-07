# vfx-release

A downloadable effect pack and the release around it, for the tests of
`og vfx add <pack>`'s download (docs/vfx-packs.md §5). Not a project: the
fixtures step never runs it.

- `spark-pack/`: a `kind vfx` pack of two tiny effects (`burst.ogt`, a
  flash and an emitter; `glint.ogt`, a flash), their particles
  (`sparks.ogt`) and one texture (`spark.png`, vfx-hit's), and a
  `preview/` that no archive or game may carry.
- `channel.ts <downloads> <version> [--evil traversal|symlink]`: a
  downloads tree holding that version's `vfx/spark-pack.tar.gz`,
  `manifest.json` and `SHA256SUMS`, written by `scripts/release-dist.ts`'s
  own `writeVfxArchives` and `finish`, signed with a fresh test key (it
  prints `pubkey <key>`); `--evil` signs an archive with one bad entry.

Read by `crates/og-cli/tests/cli/ui.rs`
(`vfx_add_downloads_a_signed_pack_and_refuses_anything_else`),
`scripts/release-dist.test.ts` and the install test's step 10
(`tests/install/run.ts`).
