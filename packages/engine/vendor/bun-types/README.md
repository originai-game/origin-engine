# bun:test declarations (vendored)

`test.d.ts` is the `bun:test` module of the npm package `bun-types` 1.4.4
(https://github.com/oven-sh/bun, `packages/bun-types/test.d.ts`, MIT, see
LICENSE), the version `scripts/install-tools.sh` pins for bun. A playtest
file (`tests/*.og.ts`) imports `describe`, `test` and `expect` from it; a
game's tsconfig maps `bun:test` here (`.og/engine/vendor/bun-types/test.d.ts`)
so it type-checks without bun-types' globals or node's declarations.

Changed from upstream, only so it is a module file under the games' ES2022
lib: the `declare module "bun:test" { ... }` wrapper is removed (its body is
the file's top level, dedented), its two inner namespaces take `declare`,
and the mock's `[Symbol.dispose](): void` member is dropped (ES2022 has no
`Symbol.dispose`). `vendor/expect-type/` is upstream's, unchanged
(Apache-2.0, its licence in `vendor/expect-type/index.d.ts`'s header).

Moving bun moves this file: copy the new version's `test.d.ts` and
`vendor/expect-type/` and make the same three changes.
