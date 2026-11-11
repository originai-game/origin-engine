<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="fn-mirror"></a>

### `og.anim.mirror`

A new clip that plays `clip` on the other side: every frame reflected in the skeleton's x = 0 plane, each node taking its counterpart's motion (a left swing becomes the right's), rotations moved in model space so left and right bone frames need not mirror each other, root motion's sideways travel and turns flipped. Assumes a symmetric rest pose; a sided node without a counterpart keeps its rest pose. Same name, duration and markers. Mirroring twice gives the clip back.

See `animation §7`.

```ts
og.anim.mirror(world: World, desc?: MirrorDesc): Result<ClipInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the skeleton and the clip. |
| `desc` | `MirrorDesc` | yes | The clip and its skeleton. |

**Returns** `Result<ClipInfo>`: The new clip (what AnimationPlayer.clip takes). Refused for an unregistered clip or skeleton.

Binding symbol `og_anim_mirror` · publish builds: kept

```ogt
fn mirror {
  doc "A new clip that plays `clip` on the other side: every frame reflected in the skeleton's x = 0 plane, each node taking its counterpart's motion (a left swing becomes the right's), rotations moved in model space so left and right bone frames need not mirror each other, root motion's sideways travel and turns flipped. Assumes a symmetric rest pose; a sided node without a counterpart keeps its rest pose. Same name, duration and markers. Mirroring twice gives the clip back."
  guide "animation §7"
  param world World "The world whose animation owns the skeleton and the clip."
  param desc MirrorDesc "The clip and its skeleton."
  returns Result.ClipInfo "The new clip (what AnimationPlayer.clip takes). Refused for an unregistered clip or skeleton."
}
```

<sub>Module `anim` line 654</sub>
