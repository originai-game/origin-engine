<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-documentSchema"></a>

### `og.ui.documentSchema`

The reflected schema documents are validated against (docs/ui.md §10): every element kind and every prop, with its value type, the kinds it applies to and whether a control writes it back.

```ts
og.ui.documentSchema(): unknown
```

**Returns** `unknown`.

Binding symbol `og_ui_document_schema` · publish builds: kept

```ogt
fn documentSchema {
  doc "The reflected schema documents are validated against (docs/ui.md §10): every element kind and every prop, with its value type, the kinds it applies to and whether a control writes it back."
  returns Json
}
```

<sub>`api.ogt` line 23902</sub>
