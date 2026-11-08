<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-document"></a>

### `og.load.document`

Load a cooked UI document (an `.ogt` of `type screen` or `type component`, validated by og cook against og-ui's reflected props; the grammar is og-ui's README §Documents: version 2, CSS declarations for an element's look and layout, docs/ui.md §3.2; `og ui migrate` writes a version 1 document as version 2) into the world's UI, where og.ui.screen and og.ui.push name a screen and a document's own markup names a component by its `name` line; its timelines are defined as `<name>/<timeline>`. Loading one of the same name again replaces it in place, the UI's state kept; `og dev` does this for an edited document.

```ts
og.load.document(world: World, path: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI uses it. |
| `path` | `string` | no | The asset id, such as `ui/shop.ogt`. |

**Returns** `Result<void>`: Refused for a path not in the bundle or a document that does not parse (the reason names the line).

Binding symbol `og_load_document` · publish builds: kept

```ogt
fn document {
  doc "Load a cooked UI document (an `.ogt` of `type screen` or `type component`, validated by og cook against og-ui's reflected props; the grammar is og-ui's README §Documents: version 2, CSS declarations for an element's look and layout, docs/ui.md §3.2; `og ui migrate` writes a version 1 document as version 2) into the world's UI, where og.ui.screen and og.ui.push name a screen and a document's own markup names a component by its `name` line; its timelines are defined as `<name>/<timeline>`. Loading one of the same name again replaces it in place, the UI's state kept; `og dev` does this for an edited document."
  param world World "The world whose UI uses it."
  param path String "The asset id, such as `ui/shop.ogt`."
  returns Result.Void "Refused for a path not in the bundle or a document that does not parse (the reason names the line)."
}
```

<sub>Module `load` line 324</sub>
