# vehicle-terrain

A driven `og.Vehicle` (a raycast car, grip 1.2, drift grip 0.6) on cooked
terrain (`assets/terrain/hills.ogt`, generated from its own numbers: height
0 to 6 m, two colour layers, a `SHAPE_HEIGHTFIELD` collider, the road
flattened with `og.terrain.edit` under an asphalt slab), with a HUD speed
readout (`og.ui`, the look's `ui "scifi"`), a chase camera that eases after
the car, tyre smoke while the rear slides (`assets/fx/smoke.ogt`), a ring of
24 skid-mark decals, and an engine tone whose pitch follows the speed
(`og.audio.createSound`). It bundles these on purpose: it is the
presentation gate's moving scene, so it stays short in ticks instead.

What it checks:

- `tests/drive.og.ts` (`og test tests/fixtures/vehicle-terrain`): throttle,
  then a handbrake turn at 2 s; the car stays above the terrain on every
  tick, it reaches speed, it slides, smoke and marks appear, the HUD reads
  the speed, and one frame matches `references/metal/drive.png`.
- `scripts/check.sh web-reference` (`web_vehicle_terrain`): the same
  playtest's frame in the browser tiers (`packages/engine/tests/web/reference.ts`).
- `packages/engine/tests/presentation.test.ts`: the presentation gate
  (docs/design.md §2.4). The capture at tick 156, mid-slide, has the same
  PNG bytes, world hash and presentation hash whether every frame is drawn,
  only the captured tick is, or a snapshot at 140 is restored (after a fork
  at 60) and the run goes on; and a SyncTest rolls back four ticks and
  plays them again on every tick from 110 with no mismatch.

Budget: under 10 s warm (measured 4.4 to 5.2 s).
