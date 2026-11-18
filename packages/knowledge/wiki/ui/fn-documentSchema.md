<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-documentSchema"></a>

### `og.ui.documentSchema`

The reflected document schema (docs/ui.md §2.1): kinds, props, types and writeback. World/mass anchors share scale-at/scale-min/scale-max. clamp: none, edge, contain; clamp-margin/arrow-size follow text scale, arrow-color sets ink, contain-fade=none disables edge fade. Root CSS reads data-clamped/data-edge/--anchor-angle/data-lod; lod sets distance thresholds, lod-lock forces level 0 only. text-floor opts into a text-only legibility floor capped per run's designed size. Mass bars accept bar-ghost ink.

```ts
og.ui.documentSchema(): unknown
```

**Returns** `unknown`.

Binding symbol `og_ui_document_schema` · publish builds: kept

```ogt
fn documentSchema {
  doc "The reflected document schema (docs/ui.md §2.1): kinds, props, types and writeback. World/mass anchors share scale-at/scale-min/scale-max. clamp: none, edge, contain; clamp-margin/arrow-size follow text scale, arrow-color sets ink, contain-fade=none disables edge fade. Root CSS reads data-clamped/data-edge/--anchor-angle/data-lod; lod sets distance thresholds, lod-lock forces level 0 only. text-floor opts into a text-only legibility floor capped per run's designed size. Mass bars accept bar-ghost ink."
  returns Json
}
```

<sub>Module `ui` line 1909</sub>
