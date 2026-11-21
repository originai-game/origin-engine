<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-documentSchema"></a>

### `og.ui.documentSchema`

Reflected document kinds, attributes and writeback. Anchors expose scaling, containment, LOD, text floors, obstacle avoidance, bounded placement and keyed stacks. placement-box selects descendant collision bounds; max-shift caps travel independently of text scale. Mass bars expose shield, segments, resources and timed ghost trails. See guide components for attribute types, placement feedback and fallback behavior.

```ts
og.ui.documentSchema(): unknown
```

**Returns** `unknown`.

Binding symbol `og_ui_document_schema` · publish builds: kept

```ogt
fn documentSchema {
  doc "Reflected document kinds, attributes and writeback. Anchors expose scaling, containment, LOD, text floors, obstacle avoidance, bounded placement and keyed stacks. placement-box selects descendant collision bounds; max-shift caps travel independently of text scale. Mass bars expose shield, segments, resources and timed ghost trails. See guide components for attribute types, placement feedback and fallback behavior."
  returns Json
}
```

<sub>Module `ui` line 1849</sub>
