# r4/anim-owed: the rest of og-anim's Owed list

Branch `r4/anim-owed`, cut from `c02c6d79` (`origin/r4/candidate`); it
continues `tests/evidence/r4/anim-procedural`. Everything is in og-anim
and og-cook, with the API surfaced through `api.ogt` (`og api gen`; `og api
check` is clean after every commit). There are no new host functions: the
new component is reflected and declared in the IDL, so both hosts get it
through the generated records. Docs for every commit: the og-anim README
(§Records, the graph grammar, §Retargeting, §Determinism, §Tests, §Owed),
og-cook's README (the `animation { mirror }` setting and the library row),
og-game skill §9, and one CHANGELOG Unreleased line per feature.

## Commits

| Commit | What |
|---|---|
| `5b9aa427` | `og.SpringBone` (spring and jiggle bones, three-vrm's VRMSpringBone as a damped spring). `joint` and its `bones` − 1 parents swing after their parents' motion. Each joint's tail (the next chain joint; for the last joint its first child, else its own rest offset continued) is a point pulled towards its animated place by `frequency` and `damping`, under `gravity`, and held at the bone's length; the joint turns onto it by the shortest arc. `space` is world (lags behind the entity's Transform) or model; `weight` blends; `lag` is written. It steps by implicit Euler, which is stable at any frequency and tick. It is applied last, after foot locks, IK and look-at, top joint first, in query order. IDL: `SpringBone` and `SpringSpace` in module anim. |
| `c8695096` | Retargeting of hands, twist bones and per-character foot heights. **Fingers:** the humanoid map grows to 52 bones: VRM 1.0's 30 finger bones are appended, so the 22 body bones keep their indices. Every scheme's finger names are read: Mixamo `LeftHandIndex1`, VRoid `J_Bip_L_Thumb1`, Rigify `DEF-f_index.01.L`, Unreal `index_01_l`, plain `index.01.L`. A finger turns from the target hand's current rotation, so a hand pose holds on an arm the clip leaves alone. A hand's rest direction is towards its middle finger. **Twist bones:** unmapped nodes named `twist`, or Rigify's `DEF-….001`, on an upper or lower arm or leg get channels. A twist bone at fraction t along the segment A→C twists by `t·τ(C rel A) − (1 − t)·τ(A rel parent)` about A's axis. **Foot heights:** `footHeight <h>` or `footHeight <left> <right>` in a `type rig` file (`HumanoidMap::foot_height`) is each ankle's height above the model's y = 0 ground; the foot goals and the leg ratio read it instead of the rest pose. The cook's retarget stamp is now `retarget-v2`. |
| `cbb639f5` | Mirroring without run-time building. **Graph:** `mirror true` in an animgraph clip block plays the clip mirrored on the skeleton the graph runs on. The system makes the clip with `og_anim::mirror` once per (skeleton, clip) and caches it with its binding, like the bindings; mirrored foot plants are cached too. Markers fire under the original clip's handle, and foot locks read the mirrored clip's plants. **Cook:** `animation { mirror "<clip>" ["<name>"] }` writes the clip (own or a library's) mirrored into the model's `#anim` (default name `<clip>.mirrored`; stamp `mirror-v1`). `og.anim.mirror` is unchanged. |

## Determinism and the state block

- The only new saved state is the spring bones' tails, per `og.SpringBone`
  entity: the skeleton, the space, and each tail's position and velocity.
  They are tagged section 3 after the entities in the `og.anim` block,
  written only when a spring exists, so a world without springs saves the
  same `og.anim` bytes as before. A spring moved to another skeleton,
  space or chain length restarts at the animated pose.
- The mirrored clip cache is derived from the assets, like the bindings,
  and is not saved. Retargeting runs at cook or load time, not in the
  tick, and has no state.
- Twist angles use og-core's bit-exact `atan2` and
  `quat_from_axis_angle`; springs use `+ - * /` and `sqrt` only.

## Tests that fail on the old code

- `tests/spring.rs` (new): all five tests fail to compile on `c02c6d79`,
  because `SpringBone` does not exist. Checked separately:
  `a_restore_mid_swing_continues_the_same` also fails when section 3 is
  not written (restored tails restart at rest). The others are the tail
  trailing a body moving at 2 m/s by over 0.1 m, then overshooting and
  settling to 1e-3 m with its bone lengths kept; model space ignoring the
  entity's motion; a 1 Hz tail drooping over three times a 4 Hz one under
  gravity; a single-joint spring turning only that joint, with a bad
  joint refused.
- `tests/retarget.rs`:
  - `fingers_are_humanoid_bones_and_a_curl_carries_over` fails when run
    against the old `src/retarget.rs` (checked): the finger bones are
    unmapped. A 0.5 rad-per-joint curl on a T-posed source bends the
    A-posed Mixamo target's index by the same angles (within 1e-3) while
    the arm is not animated. Before fingers turned from their hand, the
    first bend came out 0.285, which is why fingers take the
    hand-relative path.
  - `twist_bones_share_their_segments_twist` does not compile on the old
    code (`Retargeted::twists`), and fails with the twist pass disabled
    (checked). The forearm twist bone takes 0.6 of the hand's 1.2 rad
    roll; the upper arm's takes 0.3 of its 0.6 against the spine.
  - `a_rigs_foot_height_puts_its_feet_on_the_ground_not_where_its_rest_pose_has_them`:
    the old rig parser refuses `footHeight`. A rig lifted 0.2 m with
    `footHeight 0.1` plants its feet on that ground (within 2 mm);
    without it they stay over 0.25 m up.
- `tests/graph.rs`, `a_mirrored_graph_clip_plays_as_the_clip_og_anim_mirror_makes`:
  the old parser refuses `mirror`. Tick by tick, the pose, root motion,
  foot-lock flags and marker names and times equal those of a graph
  playing the `og.anim.mirror` clip, and differ from the walk.
- og-cook `tests/animation.rs`, `a_clip_is_mirrored_at_cook_time`: the
  old cook refuses the setting. The cooked `walkMirrored` equals
  `og_anim::mirror` of the cooked walk exactly; a missing clip, a taken
  name and a name given twice are refused.
- og-api `tests/modules.rs` checks `og.SpringBone`'s layout against the
  IDL.
- The knowledge validator holds the skill to both parsers: with the
  skill's `mirror true` line removed, `bun run test` fails ("reads
  `mirror`, which no `type animgraph` block in the skills writes").

Run, one at a time: `cargo test -p og-anim` (debug), `cargo test -p og-cook
--test animation --test retarget`, `cargo test -p og-runtime --test
model_clips`, `cargo test -p og-api --test modules`, `cargo test -p og-idl`,
`cargo clippy -p og-anim -p og-cook -p og-api --all-targets -- -D
warnings`, `cargo fmt --check`, `og api gen` / `og api check`, `bun run
test` in packages/knowledge, and `cargo check -p og-host-web --target
wasm32-unknown-unknown` (a check, not a wasm build). I did not run
`scripts/check.sh`, `og test`, web-reference, reference captures, or
wasm/release builds, and did not run `cargo test -p og-anim --release`.

## World hashes and reference frames

- **Every world that installs `og.anim` changes its `world_hash`**:
  `og.SpringBone` is a new registered record and the registry is hashed,
  as happened with LookAt, JointLimit and FootLock last round. The
  `og.anim` state bytes of a world without springs do not change.
- **`walk.rs`** (the shared scenario now puts a hand on a spring under
  gravity for every walker; new assertion: springs lag over 2 cm at
  most). Debug hashes, 3600 ticks, equal on the second run and after the
  restore at 1800: 600 `1144eae76ae7525f`, 1200 `bb87eb8d6459d6e6`, 1800
  `a2a6cf3dde36f33b`, 3600 `bd6719c3126ba02f`. The base `c02c6d79` gave
  600 `973600375bb4c1b9` and 3600 `9dd1e8df952ee49c` in debug, the same
  as the release values the last NOTES record, so debug and release
  agree there. The retargeting and mirroring commits leave these hashes
  unchanged (rerun after the last commit).
- **`tools/wasm-determinism`'s `anim_hash`** moves by design (the walk
  scenario changed and a record was added). I did not compute it: it has
  its own workspace and target. The wave-end measurement should record
  it; native and wasm must still agree.
- **Retargeting is bit for bit unchanged** on rigs without fingers, twist
  bones or `footHeight`. A hash of every walker→tall retarget (all five
  clips, foot IK on and off) is `7cfae2f4fc240c30` on both the old and
  the new code. No committed glTF has finger or twist bones (the `.001`
  names in `dropship.glb` are meshes, not `DEF-` bones), so no cooked
  `#anim` changes. The library cook stamp moves to `retarget-v2`, so
  models with an `animation { library }` re-cook once, to the same
  clips.
- **No reference frame (Metal, gfx or template playtest) should move.**
  Nothing in rendering changed. Graphs without `mirror true`, worlds
  without springs, and rigs as above take the old paths. I re-recorded no
  frames.

## For the coordinator

- Twist bones are only driven by retargeting (the channels it writes).
  A per-tick twist component for clips authored on the rig itself is
  listed as owed rather than added.
- The graph's `mirror true` mirrors lazily, on first use per skeleton.
  The first tick that plays it pays one `mirror` of the clip (not
  measured). The cook's
  `mirror` is the zero-cost path; the skill recommends it for clips
  known ahead.
- Finger bones use VRM 1.0 names (`ThumbMetacarpal`, `ThumbProximal`,
  `ThumbDistal`). Unity's legacy names (`Thumb Proximal` for the
  metacarpal) read one bone off; owed, with the rig-file workaround.

## Still owed (README §Owed)

- Resident clip compression (the loader rebuilds f32 frames).
- Spring bone colliders and per-joint stiffness along a chain.
- Per-tick twist distribution for clips that are not retargeted.
- Unity's legacy thumb names.
