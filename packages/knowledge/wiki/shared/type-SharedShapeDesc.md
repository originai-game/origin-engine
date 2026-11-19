<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="type-SharedShapeDesc"></a>

### `SharedShapeDesc` (desc struct)

What the game shares besides the transform (og.shared.shape).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `avatar` | `AnyComponent \| null` | `null` | — | — | — | The game's component whose fields every state carries (a locomotion mode, an appearance id): on the og.SharedAvatar entity, and written on each og.RemoteAvatar. Its packed bytes are at most 32. Null: none. |
| `event` | `AnyComponent \| null` | `null` | — | — | — | The game's component used as an event record (og.shared.send, og.shared.events): its packed bytes are at most 1024, and an Entity field named `from` is set to the sender's og.RemoteAvatar. Used as a shape only. Null: no events. |

```ogt
struct SharedShapeDesc {
  doc "What the game shares besides the transform (og.shared.shape)."
  desc true
  field avatar Option.AnyComponent "The game's component whose fields every state carries (a locomotion mode, an appearance id): on the og.SharedAvatar entity, and written on each og.RemoteAvatar. Its packed bytes are at most 32. Null: none."
  field event Option.AnyComponent "The game's component used as an event record (og.shared.send, og.shared.events): its packed bytes are at most 1024, and an Entity field named `from` is set to the sender's og.RemoteAvatar. Used as a shape only. Null: no events."
}
```

<sub>Module `shared` line 112</sub>
