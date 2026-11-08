<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CallbackEvent"></a>

### `CallbackEvent` (struct)

A callback to game code: it arrives in og.ui.motionEvents.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |

```ogt
struct CallbackEvent {
  doc "A callback to game code: it arrives in og.ui.motionEvents."
  field name String "Its name."
}
```

<sub>Module `ui` line 4928</sub>
