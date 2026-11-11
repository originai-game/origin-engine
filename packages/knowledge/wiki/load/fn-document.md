<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-document"></a>

### `og.load.document`

Load a cooked UI document (an `.ogt` of `type screen` or `type component`, validated by og cook against og-ui's reflected props) into the world's UI, where og.ui.push and og.ui.replace name a screen and a document's markup names a component by its `name` line; its timelines are `<name>/<timeline>`. Loading one of the same name again replaces it in place, the UI's state kept.

See `loading §6`.

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
  doc "Load a cooked UI document (an `.ogt` of `type screen` or `type component`, validated by og cook against og-ui's reflected props) into the world's UI, where og.ui.push and og.ui.replace name a screen and a document's markup names a component by its `name` line; its timelines are `<name>/<timeline>`. Loading one of the same name again replaces it in place, the UI's state kept."
  guide "loading §6"
  param world World "The world whose UI uses it."
  param path String "The asset id, such as `ui/shop.ogt`."
  returns Result.Void "Refused for a path not in the bundle or a document that does not parse (the reason names the line)."
}
```

<sub>Module `load` line 344</sub>
