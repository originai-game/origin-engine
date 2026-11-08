<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-tooltip"></a>

### `og.ui.tooltip`

A tooltip for the element just built: shown below it (above without room) once the pointer has rested on it for the slow motion duration, drawn over everything. The probe reads its text.

```ts
og.ui.tooltip(text: string): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `text` | `string` | no | The text. |

**Returns** `void`.

Binding symbol `og_ui_tooltip` · publish builds: kept

```ogt
fn tooltip {
  doc "A tooltip for the element just built: shown below it (above without room) once the pointer has rested on it for the slow motion duration, drawn over everything. The probe reads its text."
  param text String "The text."
  returns Void
}
```

<sub>Module `ui` line 707</sub>
