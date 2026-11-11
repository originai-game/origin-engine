<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-CaptureDesc"></a>

### `CaptureDesc` (desc struct)

A capture of the player's next input for a slot of an action: a controls screen's "press a key". From the next tick every action reads nothing until it ends; what was pressed for it moves no action until it is let go.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The action. |
| `slot` | `number` | `0` | — | — | — | Which binding: 0 the first; one past the last adds one. |
| `part` | `string` | `""` | — | — | — | A composite's part (`up` of WASD); empty: the whole binding (a two-axis action then takes a stick or the d-pad). |
| `accept` | `CaptureAccept` | `"any"` | — | — | — | The devices it takes. |
| `cancel` | `readonly string[]` | `[]` | — | — | — | Controls that cancel it (empty: `Escape` and `pad.back`). |
| `timeoutMs` | `number` | `0` | ≥ 0 | `ms` | — | It cancels itself after this long (0: never), counted in ticks. |

```ogt
struct CaptureDesc {
  doc "A capture of the player's next input for a slot of an action: a controls screen's \"press a key\". From the next tick every action reads nothing until it ends; what was pressed for it moves no action until it is let go."
  desc true
  field action String "The action."
  field slot u32 "Which binding: 0 the first; one past the last adds one."
  field part String "A composite's part (`up` of WASD); empty: the whole binding (a two-axis action then takes a stick or the d-pad)."
  field accept CaptureAccept "The devices it takes."
  field cancel List.String "Controls that cancel it (empty: `Escape` and `pad.back`)."
  field timeoutMs f32 "It cancels itself after this long (0: never), counted in ticks." {
    min 0
    unit ms
  }
}
```

<sub>Module `input` line 211</sub>
