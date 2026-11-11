<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-Modifiers"></a>

### `Modifiers` (struct)

Modifier keys held during a key event.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `alt` | `boolean` | `false` | — | — | — | Alt or Option. |
| `control` | `boolean` | `false` | — | — | — | Control. |
| `platform` | `boolean` | `false` | — | — | — | Command on macOS, the Windows key on Windows. |
| `shift` | `boolean` | `false` | — | — | — | Shift. |

```ogt
struct Modifiers {
  doc "Modifier keys held during a key event."
  field alt bool "Alt or Option."
  field control bool "Control."
  field platform bool "Command on macOS, the Windows key on Windows."
  field shift bool "Shift."
}
```

<sub>Module `input` line 634</sub>
