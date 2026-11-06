<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="fn-retarget"></a>

### `og.anim.retarget`

A new clip for `target` from a clip of `source`: each animated humanoid bone turns as the source's does from its rest pose (after a rest correction, so a T-pose drives an A-pose and other bone frames agree; fingers turn from their hand, so a hand pose holds whatever the arm does), the target keeps its bone lengths, the hips travel the source's distance scaled by the leg ratio (root motion kept, hip height scaled), and with footIk the feet land where the source's do, scaled, on the ground each rig's `footHeight` names (else its rest pose's foot heights). The target's twist bones (unmapped nodes named `twist`, or Rigify's `DEF-….001`, on an upper or lower arm or leg) take a share of their segment's twist by how far along it they sit. Same name, duration and markers. The cook does the same for a model's `animation { library }` import setting; this is for clips chosen at run time.

```ts
og.anim.retarget(world: World, desc?: RetargetDesc): Result<ClipInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the skeletons and the clip. |
| `desc` | `RetargetDesc` | yes | The clip, the two skeletons and their rigs. |

**Returns** `Result<ClipInfo>`: The new clip (what AnimationPlayer.clip takes). Refused for an unregistered clip, skeleton or rig, a skeleton without recognisable hips (name them in a `type rig` file), or a rig naming a missing node.

Binding symbol `og_anim_retarget` · publish builds: kept

```ogt
fn retarget {
  doc "A new clip for `target` from a clip of `source`: each animated humanoid bone turns as the source's does from its rest pose (after a rest correction, so a T-pose drives an A-pose and other bone frames agree; fingers turn from their hand, so a hand pose holds whatever the arm does), the target keeps its bone lengths, the hips travel the source's distance scaled by the leg ratio (root motion kept, hip height scaled), and with footIk the feet land where the source's do, scaled, on the ground each rig's `footHeight` names (else its rest pose's foot heights). The target's twist bones (unmapped nodes named `twist`, or Rigify's `DEF-….001`, on an upper or lower arm or leg) take a share of their segment's twist by how far along it they sit. Same name, duration and markers. The cook does the same for a model's `animation { library }` import setting; this is for clips chosen at run time."
  param world World "The world whose animation owns the skeletons and the clip."
  param desc RetargetDesc "The clip, the two skeletons and their rigs."
  returns Result.ClipInfo "The new clip (what AnimationPlayer.clip takes). Refused for an unregistered clip, skeleton or rig, a skeleton without recognisable hips (name them in a `type rig` file), or a rig naming a missing node."
}
```

<sub>`api.ogt` line 12106</sub>
