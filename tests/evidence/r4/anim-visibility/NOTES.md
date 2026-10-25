# r4/anim-visibility: a visibility track in og-anim, so a `.vox` fold is free

Base: `15ca814c4637` (`origin/r4/candidate`). The rest of owed row 63
(docs/phase-1.md; og-cook README §What phase 2 owes): a `.vox` animation's
frames were mesh swaps by a STEP scale of 0 or 1 on each frame's joint, so
every folded frame still cost a joint and a skinned draw.

## What landed

- **og-anim** (`ChannelPath::Visibility`, code 4): one lane, 0 or 1,
  always STEP (validated), sampled held. `Pose::visible` per node, from
  the rest visibility (`SkeletonAsset::hidden`); `SkeletonAsset::drawn`
  folds in the ancestors (a hidden node hides its subtree, as
  `KHR_node_visibility` and three.js do). Blend rule: a flag is chosen,
  not mixed. `lerp_to` by `t` takes the other pose's flags from
  `t ≥ 0.5` (per node by `t × mask` for a masked layer); `Blend` (blend
  trees, the player's cross-fade) takes the heaviest input's, the later on
  a tie; an additive layer sets a flag only where its clip's frame differs
  from its reference frame and its weight is ≥ 0.5. So a cross-fade
  between two mesh swaps swaps at its middle, never both frames or none.
  Mirroring moves a channel to the counterpart node; retargeting puts it
  on the target's node of the same humanoid bone, else the same name.
  `AnimOutput::drawn` hands the renderer the per-node result;
  `AnimationSystem::visible(entity)` the pose's flags. State block: tagged
  section 4 (per skeleton entity with a hidden node, its hidden nodes;
  3 is r4/anim-owed's spring tails, merged first),
  written only when some node is hidden.
- **og-cook**: the `#anim` blob reads glTF `KHR_node_visibility`
  (`visible: false` at rest → a new `hidden` table, `AnimMeta.hidden_count`,
  schema 2) and the engine's animation extension
  `OG_animation_visibility` (`animations[i].extensions.OG_animation_visibility.channels:
  [{sampler, node}]`, a float scalar per key, visible from 0.5) as STEP
  channels on `path::VISIBILITY`. `anim_codec` keeps a visibility channel
  exact at any tolerance (tolerance forced to 0: the reduction keeps only
  the frames where the flag changes and the ends). `ModelParts::nodes`:
  each part's node, so the runtime can match parts to pose nodes.
- **`.vox` cook** (`cook/vox.rs`): each frame model's mesh node gets a
  visibility track per clip instead of its joint a scale track; joints rest
  at scale 1; the mesh nodes not shown on frame 0 carry
  `KHR_node_visibility visible false` (and keep their rest scale 0, so the
  static draw without an `og.Skeleton` is unchanged). Translation tracks of
  keyed transforms stay on the joints.
- **og-runtime** (`fx::posed_model`): a part whose node is not drawn emits
  no instance and no skinned instance. Both hosts share this extraction.

## Draw count on a folded `.vox`, before and after

`crates/og-runtime/tests/model_clips.rs` cooks a three-frame `.vox`
(frames 2, 3 and 4 voxels tall), plays its `grow` clip (0 to 2) on an
`og.AnimationPlayer` at 60 ticks a second, and records what a
draw-nothing renderer receives (`RenderScene::skinned_instances`) at the
middle of each frame:

| | skinned draws a tick | frame shown at ticks 7, 22, 37, 52 | palette joint x-scales |
|---|---:|---|---|
| before (base, the test as it was) | 3 | frame 0, 1, 2, 0 (joint scales `[1,0,0]`, `[0,1,0]`, `[0,0,1]`, `[1,0,0]`) | 0 on folded frames |
| after | 1 | frame 0, 1, 2, 0 (the one drawn mesh is 2, 3, 4, 2 m tall) | all 1 |

The frames shown per tick are unchanged; the two folded frames' draws are
gone. The new assertions run against the base's sources fail
(`no joint is scaled away: [1.0, 0.0, 0.0]`, and three draws where one is
expected).

## What moves

- `og.Animation` type hash `87855139a1fc25f8` → `0dfbdcc6d6c507bf`, schema
  version 1 → 2 (`AnimMeta.hidden_count` added). Every `#anim` blob of
  every project is stale and re-cooks on the next `og cook` (the
  incremental cook checks a kept output's header against
  `expected_header`); no `COOKER_VERSION` bump was needed.
- A folded `.vox`'s blobs: its in-memory glTF changed (joint scales,
  extensions), so its source hash moves and both outputs re-cook. On the
  three-frame file above: `blob.vox.anim.ogb` 2073 → 2057 B (three
  constant-or-f32 visibility lanes of 1 float instead of scale lanes of 3),
  `blob.vox.ogb` 2662 → 2662 B, different bytes (the joints' rest scale).
  `og.Model`'s type hash does not move.
- A world with a hidden node saves a new tagged section in `og.anim`, so
  its snapshot bytes and `world_hash` differ from before; a world with no
  hidden node saves and hashes exactly as before (og-anim's `walk.rs`
  determinism hashes are unchanged: that test passes as is).
- Reference frames: none should move. The `vox-keyframes` scene of
  og-tests (`crates/og-tests/src/vox_scenes.rs`) poses its block with the
  palette, and its block's joint is unchanged (it showed on every frame:
  scale 1 then, scale 1 now, no fold); Lantern Vale's `.vox` frames
  folded at scale 0 drew zero-area triangles, which now are not drawn at
  all. Not re-recorded and not run here (see below).

## Decisions for the coordinator

1. **Blend rule**: the heavier pose's flag (ties to the later input),
   rather than any-visible; a cross-fade between mesh swaps swaps at its
   middle. Any-visible would draw both frames during a fade.
2. **Extension**: the engine's own `OG_animation_visibility` animation
   extension rather than glTF's `KHR_animation_pointer` onto
   `/nodes/{n}/extensions/KHR_node_visibility/visible`: the gltf crate
   (1.4.1) refuses a channel with the `pointer` path, so a Blender export
   using it does not load at all today. Reading `KHR_animation_pointer`
   would mean loading through `from_slice_without_validation` or patching
   the JSON first; owed if the editors' exports matter.
3. **Static draw**: a `.vox` without an `og.Skeleton` still draws its
   frames after 0 at scale 0 (zero-size draws); the model blob carries no
   visibility, and `KHR_node_visibility` on a plain glTF's node is honoured
   only when the model is posed by og-anim. Making the static path read it
   would be a model blob schema change (a `NodeRow` flag).
4. **Joints**: each frame model keeps a joint of its own (one palette
   matrix, now at scale 1). A frame that never moves could be an
   unskinned mesh node with only a visibility track; left as is so the
   skinning path, its tests and the og-tests frame stay what they were.

## Files shared with r4/anim-owed

`crates/og-anim/src/retarget.rs` (one `channels.extend(...)` line in
`mirror` and one in `retarget`, plus their doc comments; the logic is in
the new `crates/og-anim/src/visibility.rs`), `crates/og-anim/src/asset.rs`
(`ChannelPath` gains `Visibility`; `SkeletonAsset` gains `hidden`;
`ClipAsset::validate`/`visible`), `crates/og-anim/src/pose.rs` (`Pose`
gains `visible`; `lerp_to`, `lerp_to_masked`, `add`, `add_masked`,
`Blend::push`, `sample`, `RootMotion::sample_model`),
`crates/og-anim/src/system.rs` (state block section 4, `AnimOutput::drawn`),
`crates/og-anim/tests/support/mod.rs` (two `SkeletonAsset` literals now
end in `..Default::default()`), `crates/og-anim/README.md`, `CHANGELOG.md`,
`docs/phase-1.md` row 63. A `match` on `ChannelPath` without a wildcard,
a `SkeletonAsset { nodes, skins }` or a `Pose { .. }` literal written on
the other branch will need the new variant or field after the merge.

## What was run, and what was not

Run: `cargo fmt`, `cargo clippy --all-targets -- -D warnings` (whole
workspace, clean), `cargo test -p og-anim` (all, the 3600-tick walk
included), `cargo test -p og-cook` (all), `cargo test -p og-runtime`
(all). The new og-anim test `tests/visibility.rs` was checked against two
mutations (the state block not saving the flags; a blend that ORs them):
each fails it. Not run, per AGENTS.md: `scripts/check.sh` (any lane),
`og test`, web-reference, wasm or release builds, reference captures,
og-tests' gfx references, the bun/TS packages (no IDL or grammar change:
`api.ogt` and the knowledge skills are untouched, so no `api gen` and no
`bun run test` in packages/knowledge).
