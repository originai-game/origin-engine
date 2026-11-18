<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-documentSchema"></a>

### `og.ui.documentSchema`

The reflected schema documents are validated against (docs/ui.md §2.1): every element kind and prop, its type, applicable kinds and writeback. World anchors share scale-at/scale-min/scale-max with mass anchors. clamp accepts none, edge or contain; clamp-margin and arrow-size are CSS px times user text scale, arrow-color is ink. Mass bars accept bar-ghost ink.

```ts
og.ui.documentSchema(): unknown
```

**Returns** `unknown`.

Binding symbol `og_ui_document_schema` · publish builds: kept

```ogt
fn documentSchema {
  doc "The reflected schema documents are validated against (docs/ui.md §2.1): every element kind and prop, its type, applicable kinds and writeback. World anchors share scale-at/scale-min/scale-max with mass anchors. clamp accepts none, edge or contain; clamp-margin and arrow-size are CSS px times user text scale, arrow-color is ink. Mass bars accept bar-ghost ink."
  returns Json
}
```

<sub>Module `ui` line 1909</sub>
