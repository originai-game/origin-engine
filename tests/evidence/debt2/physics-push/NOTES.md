# Debt round 2, `physics-push`

The `physics-push` row of "Plan 2026-10-23, debt round 2" (docs/phase-1.md),
from b4694e67. Four items, one commit each. Every number below comes from
this orb (Intel Xeon 2.6 GHz, the dev profile). The stream rules allow no
release builds and no gate, so the perf lane's baselines are owed.

## 1. The raycast record's hierarchy refresh

The probe's `before_tick` calls `geometry::before_tick`, which builds the
list of drawn meshes for a raycast inside a system. To build it,
`Hierarchy::refresh` read every og.Transform and og.Parent and then
composed every world matrix again, every tick.

**Measured on its own** (`crates/og-runtime/tests/raycast_record_cost.rs`).
The read is og.render.raycast outside a system with `layers: 0`, which tests
no mesh, so what is left is the refresh and the target list. The scene is
20,000 meshes, every fifth parented. Best of 50 reads, or of 7 when every
mesh moves:

| code | still (nothing moved) | every mesh moved |
| --- | --- | --- |
| b4694e67 (full refresh) | 14.2 ms | 15.3 ms |
| this branch | 4.0 ms | 16.1 ms |

Phase split on the old code (scratch timing, removed): reading the columns
3.3 ms, composing world matrices 8.9 ms, the mesh rows 1.1 ms.

**Incremental.** og-ecs keeps no change ticks (only `structural_changes`,
which ignores overwrites), so the refresh compares the data itself:

- per og.Transform archetype, its entity list and its three columns against
  the last read (one memcmp each). An unchanged archetype marks its rows
  unchanged without touching them;
- otherwise, per row, the 40 bytes of position, quaternion and scale. A row
  that changed gets a new local matrix and loses its world matrix;
- og.Parent is compared the same way. If the parents changed, every world
  matrix is composed again; if not, a kept child whose ancestor changed
  loses its world matrix;
- `world_of` composes only what is not kept.

Where everything moves, the extra comparisons cost 0.8 ms (5 %) over the
old 15.3 ms. Extraction uses the same `Hierarchy`, so a frame gets the same
saving.

**Tests.** `geometry::tests::the_raycast_record_composes_only_what_moved`
runs in the gate. Over 350 meshes in three levels, it checks that the
record stays bit-identical to a fresh read through moves, a moved parent,
a moved child, reparenting, a removed transform and a despawn, and it
counts the matrices composed: 0 on a still tick, 1 for a leaf, 3 for a
parent and its two levels, 350 after a reparent. It fails on the old code
(350 composed on a still tick). It also failed with the ancestor
invalidation turned off ("kept a stale world matrix"). `render_raycast.rs`
(render raycast equals physics raycast) passes, and so does every
og-runtime test.

Owed: `perf/apple-m4-pro/raycast-record-20k.json` and
`raycast-record-20k-moving.json`, recorded on the M4 Pro with
`OG_PERF_RECORD=1 cargo test -p og-runtime --release -- --ignored
perf_raycast`. No baseline is committed for this orb's CPU.

## 2. `ext::push` splits the tick's budget

Before, the bodies were taken in hit order and each took
`min(ask, budget left)`, so the first one took the whole push. Now each
dynamic body asks once (its largest ask of the tick: 0.8 × mass × closing
speed). If the asks add up to more than `pushStrength × dt`, every body
gets the same fraction of its ask. The total stays at most
`pushStrength × dt`, as api.ogt's "the bodies it walks into share an
impulse" says.

| case | before | after |
| --- | --- | --- |
| walker into the gap between two 40 kg balls, first tick of contact | 0.335 m/s and 0 | 0.148 and 0.149 m/s |
| 2 kg crate shoved (platformer, 1 s) | 4.884 m | 4.884 m |
| 100 kg | 1.620 m | 1.620 m |
| 1,000 kg | 0.000 m | 0.000 m |

`controllers.rs::two_bodies_walked_into_at_once_share_the_push` checks
that both balls are pushed on the first tick, alike (ratio 0.9 to 1.1),
with 40 × (|va| + |vb|) ≤ 1000/60 N·s. It also runs the scenario twice and
compares the positions, rotations and velocities bit for bit. It fails on
the old code (ball b 0 m/s). All 113 og-physics tests pass.

## 3. brasswick: the `pushSpeed` workaround stays (decision for the coordinator)

I removed the slowdown and `Tuning.pushSpeed` and ran every playtest
rendered on lavapipe through playtest-run.ts. puzzle.og.ts and
frames.og.ts failed:

- the first level's crate stops at x 34.75; the plate is at 31.5 ± 0.85.
  The door is not held, and the gate's plate is never held (0.80 m of the
  joint's travel);
- door.png is 10.86 % outliers against Metal (it was 0.047 % with the
  slowdown); gate.png 0.56 %, pause.png 0.26 %, settings-es.png 0.21 %,
  results.png 0.14 %.

The cause is that the 8f226ceb push is a capped force, not a speed. A 2 kg
crate (density 2) takes it at Wick's run speed, 6.5 m/s, and slides past
its plate. I tried other crates on puzzle.og.ts:

| crate | result |
| --- | --- |
| density 40 | overshoots (34.82), gate 0.96 m |
| density 100 | 32.43 (out of tolerance), the pit's crate misses plate b |
| density 160 | holds: nothing moves |
| density 40, linearDamping 8 or 10 | levels 1 and 3 pass; the slab crate of level 2 falls at about g/8, 1.2 m/s, and misses plate a |

No crate mass reproduces a push speed. A force either speeds the crate up
to the run speed or loses to its friction. Damping gives a terminal push
speed but also a feather fall. So the slowdown is the game's push speed and
not a workaround for the missing mass. I kept it and `Tuning.pushSpeed`,
replaced the stale comment ("has no pushBodies") with the reason, and added
that to FINDINGS.md #6. The README only names the push clip, which stays,
so it is not touched.

With the slowdown kept and the engine changes of items 1 and 2, every
brasswick playtest passes (actors, audio, locales, move, puzzle, replay,
save). frames.og.ts misses only on lavapipe against the Metal references,
at RMSE 0.0101 to 0.0102 against a 0.01 limit and under 0.1 % outliers
(title 0.021 %, levels 0.007 %, start 0.037 %, door 0.047 %, stomp
0.057 %, gate 0.020 %, pause 0.080 %, settings-es 0.099 %, results
0.000 %). That is the tier difference, not a moved frame. I read door.png:
Wick beside the crate on its plate, the door up. No brasswick reference
moves.

**To decide:** keep the push speed (this branch), or redesign the crates
and levels so that a force-limited push lands them (say, crates heavy
enough to creep plus a plate stop). The second changes the game, its
playtests and frames.og.ts's door, gate and later frames.

## 4. Lantern Vale's pile

**Why it slumped.** The rocks were spheres (r 0.42) laid 1.05 m apart on a
sloping meadow, so the top two sat over 0.21 m gaps. I first laid touching
spheres with the top two in the hollows of four (r√2 over their centres).
That still slumped: rockMoved 0.51 m by 2.6 s and 0.97 m by 9.5 s. The
bottom layer alone, touching, crept 0.26 m by 9.5 s, and angularDamping up
to 12 only slowed it (0.65 m by 9.5 s). A sphere on any slope rolls, and
og.RigidBody has no sleep or rolling resistance to set.

**The pile now.** Each rock collides as the convex hull of its own voxel
mesh (SHAPE_CONVEX, the mesh from og.load.gltfInfo, offset by the mesh's
own bounds), so it rests on a face as a boulder does:

- eight on the ground, in two rows of three and a front pair over the
  seams, 0.25 m apart and turned alternately;
- two on top across the seams of four;
- density 50, friction 0.9.

The brief asked for bottom rocks that touch. Touching hulls stand still
(rockMoved 0.076 m), but the hero is shoved back as from one block: parted
0.036 m, rock-4 moved 0.020 m, hero z 11.50. The block's friction holds
more than 1,000 N. So the gaps stay.

**No hero** (`0.3:run` and nothing else): rockMoved 0.0096 m at 2.6, 4.5
and 9.5 s, against 0.60, 0.90 and 0.97 m before. The game's playtests run
11.3 s at most.

**physics.og.ts** (no renderer), measured from 0.5 s to 2.6 s:

| run | front gap widened | rock-4 moved | rock-4 rose | hero z | trunk z |
| --- | --- | --- | --- | --- | --- |
| before (spheres, density 150) | 1.106 | 1.174 | 0.029 | 9.663 | 4.508 |
| after (hulls, density 50) | 0.302 | 1.276 | 0.023 | 9.523 | 4.508 |
| after, nobody walking in | 0.000 | 0.000 | 0.000 | 13.500 | 4.508 |
| hulls density 100 | 0.28 | 0.02 | | 10.74 | |
| hulls density 70 | 0.28 | 0.40 | | 10.35 | |

The parted threshold moves from 0.5 m to 0.15 m (hulls slide apart less
than spheres roll); the others are unchanged.

**replay.og.ts.** The shove check is now physics.og.ts's: rock-4, read at
0.5 and 2.6 s, slid more than 0.5 m and rose less than 0.3 m, and the hero
is past z 10.2 at 2.6 s. A copy without the shove's keys fails it ("slid
0.00 m, the hero walked on to z 13.50"). The old `rockMoved > 0.3` also
fails on the new pile without the shove (0.01). The replay has no recorded
hash in the repo (it compares two fresh runs), so there was nothing to
re-record. Its two runs now end on `tick 576 hash 6ca9f8757e264abf`
(before: `6cc33b60f0e7fd02`), with lit 4, hurts 2, rockMoved 0.91,
farLoads 1 and look 1.

All 11 Lantern Vale playtests pass without a renderer (flow, frames, locales, lose, music, physics, replay, settings, slimes, stream, win; 10 frames unchecked).

**Not read here.** A rendered Lantern Vale run crashes in this orb, as the
earlier stream recorded, so no frame was read. The pile is near the path
north of the start, so the Metal references owed are probably
`references/metal/title.png` and `start.png`, and any other frame that
shows the meadow at (4.5, 10). The coordinator's functional lane says which.

## Files touched

- `crates/og-runtime/src/extract.rs` (`Hierarchy`; shared with extraction
  and stream.rs's volumes)
- `crates/og-runtime/src/geometry.rs` (test only)
- `crates/og-runtime/tests/raycast_record_cost.rs` (new)
- `crates/og-physics/src/ext.rs` (`push`, shared by both controllers)
- `crates/og-physics/tests/controllers.rs`
- `examples/brasswick/src/hero.ts`, `examples/brasswick/FINDINGS.md`
- `examples/lantern-vale/src/world.ts`, `tests/physics.og.ts`,
  `tests/replay.og.ts`
- this file

## Left out

- The perf baselines (release builds are the coordinator's).
- The brasswick workaround removal (see 3, a decision).
- Lantern Vale's frames, which cannot be rendered in this orb.
- `make-assets.ts` for brasswick rewrites the committed `style.ogt` (gold
  `#7a4f12` to `#7a520f`). That is generator drift unrelated to this
  stream, so I restored the file and did not commit it.
