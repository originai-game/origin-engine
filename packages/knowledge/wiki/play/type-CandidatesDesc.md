<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`play`](../play.md) · [index](../index.md)

<a id="type-CandidatesDesc"></a>

### `CandidatesDesc` (desc struct)

Where og.play.candidates looks from, how far, in which cone, among which entities and whether it looks for walls in between.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The point looked from, in world space. |
| `forward` | `Vector3` | `{ x: 0, y: 0, z: -1 }` | — | — | — | The direction angles are measured from; it need not be normalised and must not be zero. |
| `range` | `number` | `0` | ≥ 0 | `m` | — | How far it looks; 0 finds nothing. |
| `cone` | `number` | `0` | ≥ 0 | `rad` | — | The cone's half angle about `forward`; 0, or π and more, takes every direction. |
| `with` | `string` | `""` | — | — | — | The component the entities carry (og.Interactable, a game's component), by its name. |
| `sight` | `boolean` | `false` | — | — | — | Cast a ray to each one through `filter`: Candidate.visible is false when a collider stands in between. |
| `filter` | `QueryFilter` | `{}` | — | — | — | The colliders that block sight (exclude: the looker's own body). |

```ogt
struct CandidatesDesc {
  doc "Where og.play.candidates looks from, how far, in which cone, among which entities and whether it looks for walls in between."
  desc true
  field from Vector3 "The point looked from, in world space." {
    unit m
  }
  field forward Vector3 "The direction angles are measured from; it need not be normalised and must not be zero." {
    default 0 0 -1
  }
  field range f32 "How far it looks; 0 finds nothing." {
    min 0
    unit m
  }
  field cone f32 "The cone's half angle about `forward`; 0, or π and more, takes every direction." {
    min 0
    unit rad
  }
  field with String "The component the entities carry (og.Interactable, a game's component), by its name."
  field sight bool "Cast a ray to each one through `filter`: Candidate.visible is false when a collider stands in between."
  field filter QueryFilter "The colliders that block sight (exclude: the looker's own body)."
}
```

<sub>Module `play` line 107</sub>
