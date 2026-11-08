<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-OverlayBarDesc"></a>

### `OverlayBarDesc` (desc struct)

A city or strategy map's overlay picker: the overlays, the one shown and its legend.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `overlays` | `readonly MapOverlay[]` | `[]` | — | — | — | The overlays, start to end. |
| `selected` | `string` | `""` | — | — | — | The overlay shown on the map; empty shows none. |
| `label` | `string` | `"Overlays"` | — | — | — | The bar's name, to a screen reader. |
| `offLabel` | `string` | `"No overlay"` | — | — | — | The button turning the overlay off, to a screen reader. |

```ogt
struct OverlayBarDesc {
  doc "A city or strategy map's overlay picker: the overlays, the one shown and its legend."
  desc true
  field overlays List.MapOverlay "The overlays, start to end."
  field selected String "The overlay shown on the map; empty shows none."
  field label String "The bar's name, to a screen reader." {
    default "Overlays"
  }
  field offLabel String "The button turning the overlay off, to a screen reader." {
    default "No overlay"
  }
}
```

<sub>Module `ui` line 4154</sub>
