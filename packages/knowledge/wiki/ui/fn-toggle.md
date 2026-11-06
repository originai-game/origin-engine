<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-toggle"></a>

### `og.ui.toggle`

A switch with a label; a click flips it. Pass back the state it answers next tick. The probe reads its label.

```ts
og.ui.toggle(id: string, on: boolean, label: string): ToggleResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `on` | `boolean` | no | The state from game state. |
| `label` | `string` | no | The label. |

**Returns** `ToggleResult`.

Binding symbol `og_ui_toggle` · publish builds: kept

```ogt
fn toggle {
  doc "A switch with a label; a click flips it. Pass back the state it answers next tick. The probe reads its label."
  param id String "Its id."
  param on bool "The state from game state."
  param label String "The label."
  returns ToggleResult
}
```

<sub>Module `ui` line 753</sub>
