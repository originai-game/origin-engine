# bun:test declarations (vendored)

`test.d.ts` is the `bun:test` module of npm `bun-types` at the version its
first line names, pinned beside bun in docs/ci.md §Toolchain; `vendor/
expect-type/` is upstream's, unchanged (Apache-2.0, its licence in
`index.d.ts`'s header). A playtest file (`tests/*.og.ts`) imports
`describe`, `test` and `expect` from it: a game's tsconfig maps `bun:test`
here (`.og/engine/vendor/bun-types/test.d.ts`), so it type-checks without
bun-types' globals or node's declarations.

The file's header lists every local patch and why a game's tsconfig needs
it. To move bun: copy the new version's `test.d.ts` and
`vendor/expect-type/`, keep the header with its version updated, reapply
only the patches the new version still needs (type-check a playtest under
the template's tsconfig without each), and move `BUN_VERSION` and the
docs/ci.md row in the same commit. `scripts/toolchain.test.ts` holds the
three equal.
