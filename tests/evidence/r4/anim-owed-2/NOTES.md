# r4/anim-owed-2: og-anim's owed list, closed

Base `a347cca3` (origin/r4/candidate). Branch `r4/anim-owed-2`; head is the
commit that adds this file, on top of `b470ea2f`.

## Commits

| Commit | What | Test that fails on the old code |
|---|---|---|
| `13d96432` | **Spring colliders and falloff.** New `og.SpringCollider { skeleton, joint, shape (sphere/capsule), offset, tail, radius, layers }` (three-vrm's VRMSpringBoneCollider). `og.SpringBone` gains `tipFrequency`, `tipDamping` (the last joint's frequency/damping as multiples of the chain's, linear along the chain: the simplest Desc-shaped falloff, zero value-free defaults of 1), `radius` (hitRadius) and `colliders` (layer bits, default all). A tail is pushed onto the circle where its bone's sphere meets the grown collider, so the bone keeps its length. IDL: `SpringColliderShape` enum and column, `SpringColliderShape` export in `packages/engine/src/index.ts`; `api gen` run. | `tests/spring.rs` `colliders_keep_a_drooping_tail_outside_them` (no colliders: "tick 18 … 0.128 m from the centre"; a capsule read as its first end fails the same way) and `tip_frequency_and_damping_soften_a_chain_towards_its_tip` (falloff ignored: "tip droops 0.0917 against 0.0917") |
| `67f4446f` | **Per-tick twist distribution.** Retargeting's twist share moved into `retarget::share_twist` (same arithmetic); `og.anim` runs it each tick after IK/look-at and before springs on every twist bone whose segment a posing clip turns while no posing clip keys the twist bone. Caches per skeleton and per (skeleton, clip, mirrored). | `tests/retarget.rs` `twist_bones_share_their_segments_twist_when_a_clip_for_the_rig_leaves_them_still` ("forearm twist 0") |
| `afc3112d` | **Unity legacy thumbs.** On a side whose thumb has an `Intermediate` node and no `Metacarpal`, Proximal → metacarpal, Intermediate → proximal. VRM 1.0 names unchanged; rig files still override. Cook stamp `retarget-v2` → `retarget-v3`. | `tests/retarget.rs` `unitys_legacy_thumb_names_start_at_the_metacarpal` (`leftThumbMetacarpal` None, not Some(2)) |
| `595dd0e4` | **Static visibility.** `NodeRow` (`og.ModelNode`) schema 1 → 2 with `visible` from `KHR_node_visibility` (filled in `node_order`, so `#anim`'s rows carry it too); `CookedNode::visible`, `CookedModel::node_shown`, `ModelParts::shown`; og-runtime's static draw skips unshown parts. Model, proxy and `#anim` stamps gain `nodes-v2`. | og-runtime `tests/model_clips.rs` `a_vox_without_a_skeleton_draws_frame_0_only` (draws heights [2, 3, 3, 4], not [2, 3]) |
| `b470ea2f` | **KHR_animation_pointer.** `model::load` rewrites pointer channels onto `/nodes/<n>/extensions/KHR_node_visibility/visible` into `OG_animation_visibility` entries before the gltf crate parses; the crate's validation is unchanged (no `from_slice_without_validation`). Other pointers: dropped with a warning, or refused when the file requires the extension. Visibility reader accepts u8 boolean outputs. Files never naming `"pointer"` are not reparsed. | og-cook `tests/animation.rs` `khr_animation_pointer_onto_node_visibility_cooks_to_a_visibility_track` (old code refuses the file: "missing field `node`") |

Each failing assertion was checked by reverting or mutating the change
with the new test in place.

## What moves

- **World hashes.** Every world that installs og-anim hashes differently
  once: the type table gains `og.SpringCollider` and `og.SpringBone`'s
  layout changes (four fields). Snapshots taken before this branch with
  an `og.SpringBone` do not restore by layout (a migrating restore maps
  by name). The `og.anim` state block's bytes are unchanged (colliders
  hold no state; no new section). Beyond the type table, poses are bit
  for bit: springs at default falloff take the old arithmetic, and the
  twist pass only touches skeletons with `twist`/`DEF-….001` nodes, of
  which no committed rig or model has one.
- **Retargeting** is bit-identical on rigs this does not touch: a hash of
  retargeted clips (walker → tall and → itself, all clips, plus a
  twist-boned Unreal arm) is `36e6854e2f6ecc60` before and after the
  `share_twist` refactor. Only rigs using Unity's legacy thumb names map
  differently (none committed).
- **Blobs.** Every `#model`, `#proxy` and `#anim` re-cooks once: the
  model type hash moves (`og.ModelNode` v2) and the stamps carry
  `nodes-v2`. Models with an `animation { library }` re-cook for
  `retarget-v3` too. Cooked contents are the same apart from the new
  `visible` column (1 everywhere except `.vox` frames after 0).
- **Frames.** None should move: no committed glTF uses
  `KHR_node_visibility`, and the hidden `.vox` frames were drawn at scale
  0 (no pixels); now they are not drawn (fewer zero-size draws). No
  reference frame was re-recorded.
- **API.** `api.ogt` changed; generated `.d.ts`, `bindings.json`,
  `api.schema.json`, `host.rs` and the wiki regenerated (most wiki pages
  move only by `api.ogt` line numbers).

## Decisions for the coordinator

1. **Falloff form**: two scalars (`tipFrequency`, `tipDamping`, linear
   from the top joint) rather than a per-joint list: fixed-size records,
   one tunable each, and defaults that are bit-identical.
2. **Colliders on their own entity** (like `og.JointLimit`), matched by
   skeleton and layer bits, rather than a list inside `og.SpringBone`.
   Colliders are placed from the pose when each chain is swung.
3. **Twist pass trigger**: a twist bone is driven only when no clip
   posing the skeleton keys it, so retargeted clips (which write them)
   are untouched. It reads the stance the foot locks use (the base
   layer's and the player's clips), so an override layer that turns an
   arm without the base doing so does not drive it.
4. **Unity thumbs**: detected per side by an `Intermediate` node with no
   `Metacarpal`; a VRM 1.0 rig is never reread.
5. **Pointer rewrite**: a JSON patch before parse rather than loading
   without validation; refuses only when the file requires the extension.

## What was run, and what was not

Run (one at a time): `cargo fmt`; `cargo clippy --all-targets -D
warnings` on og-anim, og-api, og-cook, og-runtime; `cargo check` on
og-cli; `cargo test -p og-anim` (all, the walk included), `-p og-cook`
(all), `-p og-runtime` (all), `-p og-api --test modules`; `bun test` in
packages/knowledge (49 pass). Not run, per AGENTS.md: `scripts/check.sh`
(any lane), `og test`, web-reference, reference captures, wasm or release
builds, whole-workspace clippy. `packages/engine/tests/api-surface.test.ts`
could not run (it needs the native addon, which this stream does not
build); the new export follows `SpringSpace`'s exactly.
