<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-HapticEvent"></a>

### `HapticEvent` (struct)

A haptic pattern for the host.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | light, medium, heavy, success, warning or error. |

```ogt
struct HapticEvent {
  doc "A haptic pattern for the host."
  field name String "light, medium, heavy, success, warning or error."
}
```

<sub>Module `ui` line 4778</sub>
