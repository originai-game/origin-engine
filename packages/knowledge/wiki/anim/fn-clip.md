<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="fn-clip"></a>

### `og.anim.clip`

The index of a flipbook clip by name, for Flipbook.clip: `og.anim.clip(hero, "walk_down")`. -1 when the sheet has no such clip.

```ts
og.anim.clip(info: FlipbookInfo, name: string): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `info` | `FlipbookInfo` | no | What og.load.flipbook answered. |
| `name` | `string` | no | The clip's name. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn clip {
  doc "The index of a flipbook clip by name, for Flipbook.clip: `og.anim.clip(hero, \"walk_down\")`. -1 when the sheet has no such clip."
  host false
  param info FlipbookInfo "What og.load.flipbook answered."
  param name String "The clip's name."
  returns i32
}
```

<sub>Module `anim` line 731</sub>

<a id="fn-clip-2"></a>

### `og.anim.clip`

A loaded model's clip by name, for AnimationPlayer.clip: `og.anim.clip(og.load.animation(world, "models/slime.glb").value, "chase")`, the same lookup as a flipbook's (Animation.clips by ClipInfo.name). A skinned glTF's clips are its animations' names; an animated `.vox`'s are `frames` or its `voxClip` names. 0 (plays nothing) when the model has no such clip.

```ts
og.anim.clip(info: Animation, name: string): ClipAsset
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `info` | `Animation` | no | What og.load.animation answered. |
| `name` | `string` | no | The clip's name. |

**Returns** `ClipAsset`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn clip {
  doc "A loaded model's clip by name, for AnimationPlayer.clip: `og.anim.clip(og.load.animation(world, \"models/slime.glb\").value, \"chase\")`, the same lookup as a flipbook's (Animation.clips by ClipInfo.name). A skinned glTF's clips are its animations' names; an animated `.vox`'s are `frames` or its `voxClip` names. 0 (plays nothing) when the model has no such clip."
  host false
  param info Animation "What og.load.animation answered."
  param name String "The clip's name."
  returns ClipAsset
}
```

<sub>Module `anim` line 739</sub>

<a id="fn-clip-3"></a>

### `og.anim.clip`

The index of a 2D rig's clip by name, for Skeleton2D.clip: `og.anim.clip(og.load.rig2d(world, "rigs/hero.ogt").value, "walk")`. -1 when the rig has no such clip.

```ts
og.anim.clip(info: Rig2DInfo, name: string): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `info` | `Rig2DInfo` | no | What og.load.rig2d or og.anim.rig2dInfo answered. |
| `name` | `string` | no | The clip's name. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn clip {
  doc "The index of a 2D rig's clip by name, for Skeleton2D.clip: `og.anim.clip(og.load.rig2d(world, \"rigs/hero.ogt\").value, \"walk\")`. -1 when the rig has no such clip."
  host false
  param info Rig2DInfo "What og.load.rig2d or og.anim.rig2dInfo answered."
  param name String "The clip's name."
  returns i32
}
```

<sub>Module `anim` line 747</sub>
