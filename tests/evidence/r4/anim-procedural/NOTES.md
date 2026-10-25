# r4/anim-procedural: og-anim's Owed list as engine features

Branch `r4/anim-procedural`, cut from `42206f80` (`origin/r4/candidate`).
Everything is in og-anim, with the API surfaced through `api.ogt`
(`og api gen`; `og api check` is clean after every commit), og-runtime and
both hosts' trait impls where a function was added.

## Commits

| Commit | What |
|---|---|
| `62805e93` | `og.LookAt` (aim constraint: `axis` onto `target`, `upAxis` towards the `up` hint, held within `maxYaw`/`maxPitch`, shared over `bones` joints, `clamped` written) and `og.JointLimit` (swing cone plus twist range from rest; `JointLimit::clamp`). Two-bone IK clamps each joint as it is turned; FABRIK with a limited joint turns and clamps after every pass (an unlimited chain takes the old path, bit for bit); LookAt clamps each joint it turns. |
| `3faeb1f0` | `og.FootLock` plus `og_anim::FootPlants` / `PLANT_HEIGHT`: "planted" is the one notion retargeting uses (within 5 mm of the foot's lowest in the clip), and `tests/retarget.rs`'s contact check now reads it. The system records the clips that posed the legs each tick, with their weights. A foot planted by at least half of them is held in the skeleton entity's Transform frame (two-bone IK, joint limits respected, the foot keeping its animated orientation). It is released over `release`, or when it is past `maxDistance`. |
| `1908829e` | animgraph `mask "<node>" [weight]` (per-joint layer weights: the nearest named ancestor's weight, 0 elsewhere; override and additive), `sync <group>` on states (the first layer in a group leads and later layers play at its phase; a transition inside a group keeps the phase fraction), and `og_anim::mirror` / `mirror_name` / `mirror_pairs` plus `og.anim.mirror` in the IDL, runtime, both hosts and the TS runtime. Sides are paired by the schemes' side tokens and reflected in model space. |
| `95b1284e` | Transition interruption rules: `interrupt any \| none \| source \| next` on a transition (the default `any` is the old behaviour). |

Docs for every commit: the og-anim README (§Records, the graph grammar,
§Mirroring, §Determinism, §Tests; items moved out of §Owed), the og-game
skill §9 (the anim module's skill), and one CHANGELOG Unreleased line per
feature.

## Determinism and the state block

- New saved state is limited to the foot locks (the point, held or
  releasing, and the release time) and the interrupt guard of a fade in
  progress. Both are tagged sections after the entities in the `og.anim`
  block, each written only when non-empty, so a world using neither saves
  the same bytes as before. LookAt, JointLimit, masks, sync and mirroring
  add no state: they are pure functions of the pose, the components and
  the already-saved phases.
- Angle math (`JointLimit::clamp`, LookAt's yaw/pitch) uses og-core's
  bit-exact `atan2` and `sin_cos`. There is no host trigonometry and no
  clock.
- `walk.rs` (same hash twice, restore at 1800 continues to the same hashes
  and palettes) is extended: every walker has hinged knees and a shoulder
  cone, a two-joint look-at circling its head, both feet locked, and an
  additive wave that is masked to the arm and synced to the gait. The
  players also cycle a mirrored walk, and the idle-to-move transition is
  `interrupt none`. New assertions: heads both held at their angles and
  free (over 10,000 each); feet held over 1,000 times.
  Release hashes, 3600 ticks: 600 `973600375bb4c1b9`, 1800
  `e518a9b1204306b1`, 3600 `9dd1e8df952ee49c`, equal on the second run and
  after the restore. The baseline at `42206f80` was 600 `a03deeb38ca17e09`,
  3600 `800580b83b954cf1`. The 3600-tick hash did not change in the
  interrupt commit: the walk's guarded fade never meets a competing
  transition. The guard path is still exercised (the snapshot test below
  covers it directly).

## Tests that fail on the old code

Every test below fails on `42206f80`. Most fail because the types,
grammar or functions they use do not exist there, so they do not compile
or the parser refuses the text. Two were also checked against a broken
build with the new code otherwise in place:

- `ik.rs`: `a_joint_limit_splits_swing_from_twist_and_clamps_each`,
  `a_hinged_knee_never_bends_backwards_for_two_bone_ik` (unlimited, the
  knee bends backwards by more than 0.3 rad),
  `fabrik_keeps_a_limited_shoulder_inside_its_cone` (unlimited swing is
  above 0.9 rad), `a_head_looks_at_a_world_target_with_its_up_towards_the_hint`,
  `a_look_outside_its_angles_is_clamped_and_shared_down_the_spine`.
- `footlock.rs`: `the_walks_plants_are_retargetings`,
  `a_planted_foot_stays_put_through_root_motion_and_a_cross_fade` (a
  locked foot moves 2e-7 m; without the lock it slides 0.25 m), and
  `a_restore_mid_plant_keeps_the_foot_where_it_was`. The restore test
  also fails when the lock section is not written (checked).
- `layers.rs`: `a_mask_limits_an_override_layer_to_its_branch_by_weight`,
  `a_synced_layer_plays_at_its_leaders_phase`,
  `a_transition_inside_a_group_keeps_the_phase`. The old parser refuses
  `mask` and `sync`.
- `retarget.rs`: `sides_swap_in_every_schemes_names`,
  `a_mirrored_walk_moves_each_foot_as_the_other_did_reflected` (within
  1e-5 on the walker and 1e-4 on the tall Mixamo rig; mirroring twice
  gives the clip back).
- `graph.rs`: `an_interrupt_rule_says_which_transitions_may_cut_a_fade`,
  and `a_guarded_fade_survives_a_snapshot`, which also fails when the
  guard section is not written (checked).
- og-runtime `model_clips.rs`: `a_mirrored_clip_is_registered_once_and_plays`.
- og-api `modules.rs` now checks the LookAt, JointLimit and FootLock
  layouts against the IDL.

Run: `cargo test -p og-anim` (debug and release), `cargo test -p og-api
--test modules`, `cargo test -p og-runtime --test model_clips`, `cargo
clippy -p og-anim -p og-runtime --all-targets -D warnings`, `cargo fmt
--check`, `og api check`, and `cargo check -p og-host-web --target
wasm32-unknown-unknown` (a check only, not a wasm build). I did not run
`scripts/check.sh`, `og test`, web-reference, reference captures or
wasm/release builds.

## World hashes and reference frames

- Every world that installs `og.anim` gets three new registered records
  (`og.LookAt`, `og.JointLimit`, `og.FootLock`). The registry is hashed,
  so **every such world's `world_hash` moves**, including worlds that use
  none of the features. No hashes are committed in this repository except
  history in `docs/phase-1.md`.
- `tools/wasm-determinism`'s `anim_hash` (the walk scenario after 600
  ticks) **moves by design**: the scenario changed and the records were
  added. `docs/phase-1.md` records `6030136770684287795` for it. I did not
  compute the new value: that crate has its own workspace, and building it
  would need a second target directory. The wave-end measurement will
  record it. Its native and wasm builds must still agree.
- `packages/engine/tests/anim.test.ts` compares the node host with the web
  host and holds no committed value.
- **No reference frame (Metal, gfx or template playtest) should move.**
  Nothing in rendering changed. A clip, graph or IK chain without the new
  components or grammar takes the same code path, bit for bit: the
  unlimited FABRIK path and the unmasked, unsynced layer paths are
  unchanged. og-tests' skinning scene poses joints directly. I re-recorded
  no frames.

## For the coordinator

- The walk scenario is shared with `tools/wasm-determinism` (it includes
  `tests/support/mod.rs`), which now builds the mirrored walk at
  registration. It is pure Rust with no new dependencies.
- A hinge set by JointLimit with `swing: 0` under two-bone IK misses its
  target by the off-plane part when the solve's bend plane is not exactly
  the hinge. The docs advise 0.1 to 0.2 rad of play.
- FootLock only holds feet the clips actually rest. The procedural walker
  walk plants a foot for one or two frames, so `footlock.rs` plays a
  stomp clip.

## Still owed (README §Owed)

- Retargeting: hands and fingers, twist bones, per-character foot
  heights.
- Springs and jiggle bones.
- Mirroring at cook time or per graph clip (today it happens at run time
  through `og.anim.mirror`).
