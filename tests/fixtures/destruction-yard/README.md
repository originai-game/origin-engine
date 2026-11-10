# destruction-yard

og.destruction on three props a game breaks: a brick wall (a charge at its
foot, `boom`), a concrete pillar (a blow, `strike`) and a glass pane (a
thrown ball, `throw`), on a static slab, with dust (`assets/fx/dust.ogt`).
Every asset is `.ogt` text; the fracture runs in the cook.

What it checks:

- `tests/yard.og.ts` (`og test tests/fixtures/destruction-yard`): each prop
  breaks on its beat, the game's `og.assert` sites hold on every tick (no
  chunk falls through the slab, read from `og.MatrixWorld` inside the
  slab's footprint), and one frame of the settled yard matches
  `references/vulkan/yard.png` (rmse ≤ 3).
- `packages/engine/tests/destruction.test.ts` with
  `support/destruction-hash.ts` and `support/yard-piers.ts`: the yard's
  world hash after the beats (`BEATS` in `src/main.ts`) is the same on the
  native and the wasm host, and the wall's piers and the pillar's stump
  settle where they did (the determinism golden: tick 330, hash
  `470e7c1791d227d5`).
- `crates/og-runtime/tests/destruction.rs`: the pillar's cooked fracture.

Budget: under 10 s warm for `og test` (measured 3.5 to 6 s of run plus the
cook's 3.7 s, which reruns the fracture before it can see the cook is up
to date).
