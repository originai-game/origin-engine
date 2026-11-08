<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setProp"></a>

### `og.ui.setProp`

Set a prop of a document element live (Tune, the probe's ui:set): presentation only, over the document's own line until og.ui.saveEdits writes it into the file. The value is written as the document's line would be (`token primary`, `grow`, `bind gold compact`).

```ts
og.ui.setProp(world: World, path: string, prop: string, value: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The element's full path. |
| `prop` | `string` | no | The prop (og.ui.documentSchema). |
| `value` | `string` | no | Its value as a document line writes it. |

**Returns** `Result<void>`: Refused for an unknown prop or a value it does not take.

Binding symbol `og_ui_set_prop` · publish builds: kept

```ogt
fn setProp {
  doc "Set a prop of a document element live (Tune, the probe's ui:set): presentation only, over the document's own line until og.ui.saveEdits writes it into the file. The value is written as the document's line would be (`token primary`, `grow`, `bind gold compact`)."
  param world World "The world."
  param path String "The element's full path."
  param prop String "The prop (og.ui.documentSchema)."
  param value String "Its value as a document line writes it."
  returns Result.Void "Refused for an unknown prop or a value it does not take."
}
```

<sub>Module `ui` line 8132</sub>
