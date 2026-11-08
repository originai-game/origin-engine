<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-saveEdits"></a>

### `og.ui.saveEdits`

Write the live edits into their documents' `.ogt` files under the project's assets (each edited element's line of that prop replaced, or one added; every other line kept), as save_scene writes scenes; the documents take the new text at once. Answers the files written.

```ts
og.ui.saveEdits(world: World): Result<readonly string[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `Result<readonly string[]>`: Refused on a host with no project directory, or a write that fails.

Binding symbol `og_ui_save_edits` · publish builds: kept

```ogt
fn saveEdits {
  doc "Write the live edits into their documents' `.ogt` files under the project's assets (each edited element's line of that prop replaced, or one added; every other line kept), as save_scene writes scenes; the documents take the new text at once. Answers the files written."
  param world World "The world."
  returns Result.List.String "Refused on a host with no project directory, or a write that fails."
}
```

<sub>Module `ui` line 4363</sub>
