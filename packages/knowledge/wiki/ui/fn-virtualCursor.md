<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-virtualCursor"></a>

### `og.ui.virtualCursor`

The virtual cursor as the last frame left it; null when none is declared.

```ts
og.ui.virtualCursor(): VirtualCursor | null
```

**Returns** `VirtualCursor | null`.

Binding symbol `og_ui_virtual_cursor` · publish builds: kept

```ogt
fn virtualCursor {
  doc "The virtual cursor as the last frame left it; null when none is declared."
  returns Option.VirtualCursor
}
```

<sub>`api.ogt` line 17382</sub>
