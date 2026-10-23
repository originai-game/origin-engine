# Debt round `two-d-math`: evidence

Branch `debt/two-d-math`, from main `e568f91f`. It closes what the depth
stream S9 left owed (`tests/evidence/depth/two-d-math/NOTES.md`, "Owed and
left out"). Every frame here was rendered on lavapipe (`vulkan-cpu`,
llvmpipe 15.0.6, this orb) with `og run --headless --commands`, using a
native addon built from this branch (`OG_NATIVE_ADDON`). The one exception
is `og-gfx skeleton-2d`, which was rendered only to compare bytes and was
not captured as a reference.

## Commits

| Commit | Item | What |
|---|---|---|
| `46d9d8c4` | 1 | The DragonBones import keeps skew, colour offsets, zOrder and IK timelines, nested armatures, bounding boxes and Bézier curves. The rig2d grammar gains: `skew` on bones and regions; `offset` on slots; `box` and `child` attachments; `armature` blocks; `skew`, `offset`, `order` and `ik` keys; and `curve` eases. og-anim poses and draws all of it. og-render's sprite mesh adds a corner's colour offset after its multiplier (`fs_mesh`). The API gains `og.anim.skeleton2dBoxes` and `Skeleton2DBox`. |
| `b6947e39` | 2 | IK over a chain of any length: one bone and two bones keep their solvers; three or more bones are solved by FABRIK. The DragonBones import takes any `chain`. |
| `eefddf89` | 3 | The lamp keeper walks: a `walk` clip, the `Walk` system and `KeeperWalk` resource, the feet stepping with the gait in `Plant`, and the playtest extended. |

## The test that fails on the old code, per item

**Item 1.** In og-anim `rig2d.rs`:
- `skew_turns_a_bones_y_axis_apart_from_its_x_axis`
- `bezier_keys_follow_their_curve_not_the_line`: ease-in at half time is 0.3153568 of the way, a value solved independently in Python. The line would give 0.5.
- `colour_offsets_add_after_the_multiplier_and_animate`
- `ik_keys_animate_the_weight_and_bend_and_the_target_weight_multiplies`
- `a_child_armature_is_posed_and_drawn_in_its_slot`
- `order_keys_redraw_the_slots_back_to_front`
- `bounding_boxes_are_the_polygons_the_pose_shows`

In og-anim `tests/skeleton2d.rs`:
- `bounding_boxes_follow_the_pose_in_the_entitys_frame_and_restore_with_snapshots`

In og-cook `tests/rig2d.rs`:
- `a_dragonbones_export_keeps_skew_offsets_order_ik_keys_children_boxes_and_curves`

The old parser refuses every new line as an unknown key. The old import drops each feature this test asserts.

**Item 2.** These fail on the old code:
- `a_chain_of_any_length_reaches_and_curls_the_asked_way` (og-anim `rig2d.rs`)
- `an_ik_target_on_a_chain_of_three_bones_is_reached_and_its_error_is_the_tips_distance` (og-anim `tests/skeleton2d.rs`)
- `a_dragonbones_ik_chain_of_any_length_is_imported_root_first` (og-cook; the old import refused `chain` 2)

`one_and_two_bone_ik_give_the_poses_they_always_gave` holds the old solver's numbers to the bit: the four two-bone and two one-bone cases were printed from `e568f91f` before any change. It passes on both the old and the new code.

**Item 3.** In `templates/game-2d/tests/keeper.og.ts`, these checks fail on the old code:
- "walking, the keeper plays its walk clip". The old `keeper:7` teleported the keeper, and the rig had no clip 1.
- "walking, it is part of the way along the ramp".
- "the planted foot stays put while the keeper passes over it". The old target jumped by 1 m.
- "the walk clip's events counted 5 steps".

The playtest was not run with `og test` (as briefed). Its exact command script was run through `og run --headless --commands`, and its 16 conditions were evaluated on the dumps by a script: all 16 hold.

## Why FABRIK (item 2)

I chose FABRIK over CCD, for these reasons:

1. **It handles the whole chain together.** FABRIK spreads the bend along the chain. CCD turns the last joints most and curls the tip first, so a tail or tentacle looks hooked unless every joint is damped and limited.
2. **Its passes contain no trigonometry.** They are position passes with `sqrt` only; each bone's turn is read once at the end through the engine's arithmetic `atan2`. CCD needs an `atan2` per joint per iteration, so its rounding builds up over the iterations; FABRIK's does not.
3. **It converges in fewer iterations** to the same tolerance, and an out-of-reach target becomes one straight line exactly.
4. **It matches the 3D side.** og-anim's 3D chains (`ik.rs`, `fabrik`) are already solved this way, so both dimensions share one model.

FABRIK has no bend direction of its own. The solver seeds the interior joints on the `bend` side of the line from the root to the target before the passes: it mirrors them across the line when they lie on the other side, and pushes them off it when the chain is straight. The test checks that every interior joint ends on the asked side, for both bends and for a target behind the root.

One bone and two bones keep their closed-form solvers. These are exact: the two-bone triangle solution picks the side precisely, and is what the keeper's legs use. Their test holds the old numbers.

## Frames read

**`game-2d-keeper-idle-walk-arrive.png`** shows four 260×200 crops of the plank at 2× zoom, from the playtest's script, left to right:
- **Tick 60, idle.** The keeper faces left and both boots are on the plank. This frame is byte-identical to the frame from `e568f91f`'s code (`cmp`).
- **2.6 s, walking.** It now faces right and the crimson cape trails left, behind it (cape-1 at -10° to -14°, cape-2 at -12° to -20°). Dumps: keeper x 6.20833 and clip time 0.41667, both 25 ticks at 0.5 m/s. The back boot is planted at (6.1499968, 2.4707234), 0.0 above the plank. The front boot swings at x 6.2666636, 0.01278 above it.
- **2.9 s.** The front boot is lifted visibly off the plank: x 6.5666614, 0.05278 up, against a hand-computed 0.0527. The back boot has not moved: (6.1499944, 2.470723).
- **4.6 s, arrived.** The keeper is at x 7 and y 2.6114, higher up the ramp, facing left again. It is back on the idle clip, at time 0.41667 since arrival. Both targets are on the ramp, (6.85, 2.619516) and (7.15, 2.683284), reached with errors 4.9e-8 and 5.3e-8.

Game counts: `inhales` 2 at 2.1 s, then 3 at the end (the idle clip restarts with its breath), and `steps` 5 (walk events at 0, 0.4, 0.8, 1.2 and 1.6 s of a 2 s walk). The lantern's yellow light moves with the hand to the keeper's front on each side.

**`game-2d-keeper-walk-tick156.png`** is the playtest's `walk.png` at full frame: the rooftop level with the keeper mid-plank, facing right.

**`dragonbones-import-tick15.png` and `-tick45.png`.** These come from a demo project, not committed. A hand-written DragonBones 5.5 export (24 fps) over the keeper's texture was imported by the cook and played on native, at unit 8:
- **Skew.** The arm bone has `skX` 30, `skY` 0, so it draws as a parallelogram: the teal arm leans right of the torso.
- **Colour offset.** The torso slot's `rO` 150 fades to 0 over the clip. A torso pixel at (530, 440) reads red 203 at 0.25 s and 139 at 0.75 s, against 37 where no offset applies (620, 415).
- **zOrder.** At 0.5 s a key moves the torso two places up. At 0.25 s the arm draws over the torso; at 0.75 s it is hidden behind it, with only its tip showing.
- **Child armature.** The `flame` armature display draws the lantern region with `gO` 90. A pixel at (740, 340) reads (232, 237, 210) at 0.25 s and (232, 232, 127) at 0.75 s, as the region turns 22.5° to 67.5° on the child's `burn` clip (1 s, 0 to 90°). Its green cap turns from the top to the right side.
- **Bounding box.** `og.anim.skeleton2dBoxes` answered one box, a 12×20 px rectangle on the body. Opposite corners are at (-0.955666, 0.149226) and (0.955666, 2.350774). That is (1.5, 2.5) world units turned -10°, the body's `curve` rotation at the middle of its first half (0 to 20° with a symmetric curve, so half the value), and it is the same at 0.25 s and 0.75 s.

**`dragonbones-chain-tick15.png`** is the same demo with three 14 px tail bones (`chain` 2, `bendPositive` true, imported as `bend clockwise`) reaching an `og.IkTarget2D` at (-3.5, 1.5). The teal tail zigzags left of the torso, its tip at the target. The IkTarget2D reads `reached` 1 with error 6.1e-6.

**`og-gfx skeleton-2d`** was rendered on lavapipe at `e568f91f` and on this branch, to `/tmp`. The two frames are byte-identical (`cmp`; `magick compare -metric AE` gives 0). Stats: sprites 3, sprite_draws 4. The new `fs_mesh` gives the old shader's pixels where the offset is zero, and adding the walk clip to the og-tests copy of the rig leaves its idle clip (index 0) as it was.

## Shared files touched

- `crates/og-api/api.ogt`: `Skeleton2DBox` and `fn skeleton2dBoxes` in `anim`; the docs of `Rig2D`, `Skeleton2D`, `IkTarget2D` and `IkTarget2D.weight`. The generated outputs were regenerated (`og api gen`, `og api gen --wiki`) and `og api check` is clean. The wiki moved many line numbers.
- Both hosts' `host.rs` (`og_anim_skeleton2d_boxes`); `wasm32-unknown-unknown` and native both check.
- `packages/engine/src/runtime.ts` (`anim.skeleton2dBoxes`).
- og-render: `sprite_mesh.rs` (`SpriteMeshVertex.offset`, now `Default`; the vertex layout; `fs_mesh`), `shaders/sprite_mesh.wgsl`, and the test helper in `sprites.rs`.
- og-runtime: `skeleton2d.rs` and `lib.rs`.
- og-tests: `skeleton_scenes.rs` (passes the offset) and `assets/rig2d/keeper.ogt` (takes the walk clip).
- `docs/design.md` §2.7 and the og-game skill's "2D skeletons: the rig2d grammar".
- `templates/game-2d`: `README.md`, `src/main.ts`, `assets/rigs/keeper.ogt`, `tests/keeper.og.ts`.
- `CHANGELOG.md` is untouched.

## Metal references the coordinator owes

- `crates/og-tests/references/metal/skeleton-2d.png`: **none new.** The gfx scene's frame did not change (byte-identical on lavapipe). The reference S9 already owed is still owed, if it has not been captured since.
- templates/game-2d's playtest frames: `keeper.png` is unchanged, and `walk.png` is new. Neither has a committed reference: `t.frame` only checks the size.

## Left out, and why

- **Not run (as briefed):** `og test`, `scripts/check.sh`, web-reference, and wasm and release builds. `packages/engine/native/` was not rebuilt.
- **The turn and the stop pop.** On setting off, the keeper turns in one tick (flipX) and its feet swap sides. On arrival, the feet go from wherever the gait left them back to standing (at most a quarter stride, 0.1 m, in a tick). A turn clip and a stop that waits for both feet to land would remove the pops; that is template polish, not owed math.
- **DragonBones' `clockwise` rotation turns** (more than half a turn between keys) play the shorter way, with a cook note. The rig2d rotate key interpolates the shorter way.
- **`tweenEasing` above 1** (sine in-out) has no cubic Bézier form. It plays as smoothstep, which is within 0.011 of it, with a cook note. Below 1 the quadratic eases are exact Béziers.
- **Path displays and transform or path constraints** are still not imported (noted). Neither has a line in rig2d.
- **Only the first skin** is read, and armatures that no display shows are noted and not imported.
- **A child armature's events** do not fire as og.Skeleton2DEvent. Only the parent clip's events fire, as in DragonBones, where a child's events go to its own armature.
