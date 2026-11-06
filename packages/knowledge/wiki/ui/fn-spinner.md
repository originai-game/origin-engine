<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-spinner"></a>

### `og.ui.spinner`

A loading spinner: eight dots around a circle, the bright one turning by the tick.

```ts
og.ui.spinner(id: string, size: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `size` | `number` | no | Its diameter in CSS px; 0 is the small control height. |

**Returns** `void`.

Binding symbol `og_ui_spinner` · publish builds: kept

```ogt
fn spinner {
  doc "A loading spinner: eight dots around a circle, the bright one turning by the tick."
  param id String "Its id."
  param size f32 "Its diameter in CSS px; 0 is the small control height."
  returns Void
}
```

<sub>`api.ogt` line 17229</sub>
