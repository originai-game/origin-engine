<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setHighlight"></a>

### `og.ui.setHighlight`

Register a named custom highlight, like CSS.highlights.set: from the next frame, its ranges paint with `::highlight(<name>)` rules. Each property comes from the nearest element or ancestor declaring it; `:root::highlight(found) { … }` styles every element. No rule means no paint. Setting a name again replaces its highlight without changing its registration order. It lasts until replaced or removed and is presentation only: snapshots do not carry it. Use it for search hits, tutorial words or spell-check marks.

```ts
og.ui.setHighlight(name: string, highlight: UiHighlight | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | Its name, as `::highlight(<name>)` writes it (case matters). |
| `highlight` | `UiHighlight \| null` | no | Its ranges and priority; null removes it. |

**Returns** `void`.

Binding symbol `og_ui_set_highlight` · publish builds: kept

```ogt
fn setHighlight {
  doc "Register a named custom highlight, like CSS.highlights.set: from the next frame, its ranges paint with `::highlight(<name>)` rules. Each property comes from the nearest element or ancestor declaring it; `:root::highlight(found) { … }` styles every element. No rule means no paint. Setting a name again replaces its highlight without changing its registration order. It lasts until replaced or removed and is presentation only: snapshots do not carry it. Use it for search hits, tutorial words or spell-check marks."
  param name String "Its name, as `::highlight(<name>)` writes it (case matters)."
  param highlight Option.UiHighlight "Its ranges and priority; null removes it."
  returns Void
}
```

<sub>Module `ui` line 535</sub>
