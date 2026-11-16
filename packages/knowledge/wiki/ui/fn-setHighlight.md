<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setHighlight"></a>

### `og.ui.setHighlight`

Register a custom highlight under a name, as a browser's CSS.highlights.set does: from the next frame, its ranges paint with the `::highlight(<name>)` rules of the elements they cross. Each property comes from the nearest of the element and its ancestors that declares it, so `:root::highlight(found) { … }` styles every element. A highlight with no rule paints nothing. Setting a name again replaces its highlight in its place of registration. It holds until it is set again or removed, and it is presentation: snapshots do not carry it. Search hits, a tutorial's pointed-out words and a spell checker's marks are what it is for.

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
  doc "Register a custom highlight under a name, as a browser's CSS.highlights.set does: from the next frame, its ranges paint with the `::highlight(<name>)` rules of the elements they cross. Each property comes from the nearest of the element and its ancestors that declares it, so `:root::highlight(found) { … }` styles every element. A highlight with no rule paints nothing. Setting a name again replaces its highlight in its place of registration. It holds until it is set again or removed, and it is presentation: snapshots do not carry it. Search hits, a tutorial's pointed-out words and a spell checker's marks are what it is for."
  param name String "Its name, as `::highlight(<name>)` writes it (case matters)."
  param highlight Option.UiHighlight "Its ranges and priority; null removes it."
  returns Void
}
```

<sub>Module `ui` line 535</sub>
