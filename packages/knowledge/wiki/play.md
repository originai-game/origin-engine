<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `play` module

Mechanisms that span domains, as facts. An og.HitVolume (on a bone through og.BoneAttachment, or on any entity) is swept from where it was at the last tick to where it is, so a fast swing meets what it passed through, and strikes each og.HurtVolume of another team, and the colliders on its layers, once a swing: og.HitEvent. og.play.candidates lists what is in range, in a cone and in sight. Damage, prompts and choices are the game's rules.

- **TypeScript**: functions are the namespace `og.play`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module](index.md).

```ogt
module play {
  doc "Mechanisms that span domains, as facts. An og.HitVolume (on a bone through og.BoneAttachment, or on any entity) is swept from where it was at the last tick to where it is, so a fast swing meets what it passed through, and strikes each og.HurtVolume of another team, and the colliders on its layers, once a swing: og.HitEvent. og.play.candidates lists what is in range, in a cone and in sight. Damage, prompts and choices are the game's rules."
  # declarations follow, each on its own page
```

<sub>Module `play` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`VolumeShape`](play/type-VolumeShape.md) | enum |  | A hit or hurt volume's shape: og.HitVolume.shape and og.HurtVolume.shape. |
| [`VolumeShape()`](#column-VolumeShape) | column kind |  | A VolumeShape field: the variant's index. |
| [`og.HitVolume`](play/record-HitVolume.md) | component |  | What strikes: a shape at its entity's simulation pose (its og.Transform after its parents', a posed bone's frame with og.BoneAttachment; og.motion's drawn pose is never read). |
| [`og.HurtVolume`](play/record-HurtVolume.md) | component |  | What can be struck: a shape at its entity's simulation pose, as og.HitVolume's (a bone through og.BoneAttachment). |
| [`og.HitEvent`](play/record-HitEvent.md) | event |  | A hit volume struck a target for the first time in its owner's swing, sent the tick it met it, in hit volume entity order, then where along its sweep, then target. |
| [`CandidatesDesc`](play/type-CandidatesDesc.md) | desc struct |  | Where og.play.candidates looks from, how far, in which cone, among which entities and whether it looks for walls in between. |
| [`Candidate`](play/type-Candidate.md) | struct |  | An entity og.play.candidates found. |
| [`og.play.candidates`](play/fn-candidates.md) | function | `(world: World, desc?: CandidatesDesc): Result<readonly Candidate[]>` | Every entity carrying a component within a range and a cone of a point, nearest first (equal distances in entity order), with its distance, angle and whether it is in sight: what an interaction promp… |

## Column kinds

Each is a factory, `VolumeShape(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-VolumeShape"></a>`VolumeShape` | `VolumeShape` | `Uint32Array` | 1 | A VolumeShape field: the variant's index. |

```ogt
column VolumeShape {
  doc "A VolumeShape field: the variant's index."
  array Uint32Array
  stride 1
  value VolumeShape
}
```

<sub>Module `play` line 11</sub>
