<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-Rig2DInfo"></a>

### `Rig2DInfo` (struct)

What og.load.rig2d registered, or og.anim.rig2dInfo reads back from a handle.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rig` | `Rig2D` | `0` | — | — | — | What Skeleton2D.rig takes. |
| `texture` | `Texture` | `0` | — | — | — | The rig's texture (loaded with it). |
| `unit` | `number` | `0` | — | — | — | Pixels of the rig per world unit. |
| `bones` | `readonly string[]` | `[]` | — | — | — | The bones' names, parents first. |
| `slots` | `readonly string[]` | `[]` | — | — | — | The slots' names, in draw order. |
| `constraints` | `readonly string[]` | `[]` | — | — | — | The IK constraints' names: an index here is IkTarget2D.constraint. |
| `skins` | `readonly string[]` | `[]` | — | — | — | Its skins' names, `default` (the rig's own attachments) first: an index here is Skeleton2D.skin. |
| `clips` | `readonly Rig2DClipInfo[]` | `[]` | — | — | — | Its clips, in file order. |

```ogt
struct Rig2DInfo {
  doc "What og.load.rig2d registered, or og.anim.rig2dInfo reads back from a handle."
  field rig Rig2D "What Skeleton2D.rig takes."
  field texture Texture "The rig's texture (loaded with it)."
  field unit f32 "Pixels of the rig per world unit."
  field bones List.String "The bones' names, parents first."
  field slots List.String "The slots' names, in draw order."
  field constraints List.String "The IK constraints' names: an index here is IkTarget2D.constraint."
  field skins List.String "Its skins' names, `default` (the rig's own attachments) first: an index here is Skeleton2D.skin."
  field clips List.Rig2DClipInfo "Its clips, in file order."
}
```

<sub>`api.ogt` line 12332</sub>
