<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-documentSchema"></a>

### `og.ui.documentSchema`

Reflected document kinds, attributes and writeback (docs/ui.md §2.1). Anchors share scale-at/scale-min/scale-max; clamp/contain-fade, arrow-size/color and clamp-margin control edges. Root feedback: data-clamped/data-edge/--anchor-angle/data-lod/data-blocked. lod-lock and text-floor protect detail and text. reserve=hud marks screen boxes; reserve=player uses reserve-radius and reserve-height world units. avoid=player/danger opts into volumes or ground danger bounds; keep-visible preserves an unplaceable plate. Mass bars accept bar-ghost ink.

```ts
og.ui.documentSchema(): unknown
```

**Returns** `unknown`.

Binding symbol `og_ui_document_schema` · publish builds: kept

```ogt
fn documentSchema {
  doc "Reflected document kinds, attributes and writeback (docs/ui.md §2.1). Anchors share scale-at/scale-min/scale-max; clamp/contain-fade, arrow-size/color and clamp-margin control edges. Root feedback: data-clamped/data-edge/--anchor-angle/data-lod/data-blocked. lod-lock and text-floor protect detail and text. reserve=hud marks screen boxes; reserve=player uses reserve-radius and reserve-height world units. avoid=player/danger opts into volumes or ground danger bounds; keep-visible preserves an unplaceable plate. Mass bars accept bar-ghost ink."
  returns Json
}
```

<sub>Module `ui` line 1909</sub>
