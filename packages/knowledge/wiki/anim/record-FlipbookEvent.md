<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-FlipbookEvent"></a>

### `og.FlipbookEvent` (event)

A flipbook entered a frame its clip names in an `event` line (`event 2 "step"`): footsteps, the frame a hit lands, a pickup's sparkle. Read in the tick after, like every event; a clip started at 0 fires its first frame's events.

Token `og.FlipbookEvent`, registered under the schema name `og.FlipbookEvent`; its value type is `FlipbookEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity with the og.Flipbook. |
| `clip` | `number` | `0` | — | — | — | The clip index. |
| `frame` | `number` | `0` | — | — | — | The clip frame entered. |
| `name` | `string` | `""` | — | — | — | The event's name (at most 63 bytes). |

```ogt
event FlipbookEvent {
  doc "A flipbook entered a frame its clip names in an `event` line (`event 2 \"step\"`): footsteps, the frame a hit lands, a pickup's sparkle. Read in the tick after, like every event; a clip started at 0 fires its first frame's events."
  field entity Entity "The entity with the og.Flipbook."
  field clip u32 "The clip index."
  field frame u32 "The clip frame entered."
  field name String "The event's name (at most 63 bytes)."
}
```

<sub>`api.ogt` line 12166</sub>
