<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DescribedView"></a>

### `DescribedView` (struct)

A secondary camera (og.CameraView, og.Mirror) and what its image shows: its own entity-id pass, from its eye and on its layers, at a quarter of its image's pixels, whatever its `every`. An entity only its layers admit (a layer the main camera leaves out) is listed here and not in Description.entities.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The view entity's probe id. |
| `name` | `string \| null` | `null` | — | — | — | Its name. |
| `kind` | `string` | `""` | — | — | — | `camera` (og.CameraView) or `mirror` (og.Mirror). |
| `size` | `PassSize` | `{ width: 0, height: 0 }` | — | — | — | The pixels its image covers: its render target's, or its viewport's rect of the frame. |
| `entities` | `readonly DescribedEntity[]` | `[]` | — | — | — | What it shows, nearest its eye first, boxes and points in its image's pixels. |

```ogt
struct DescribedView {
  doc "A secondary camera (og.CameraView, og.Mirror) and what its image shows: its own entity-id pass, from its eye and on its layers, at a quarter of its image's pixels, whatever its `every`. An entity only its layers admit (a layer the main camera leaves out) is listed here and not in Description.entities."
  field id String "The view entity's probe id."
  field name Option.String "Its name."
  field kind String "`camera` (og.CameraView) or `mirror` (og.Mirror)."
  field size PassSize "The pixels its image covers: its render target's, or its viewport's rect of the frame."
  field entities List.DescribedEntity "What it shows, nearest its eye first, boxes and points in its image's pixels."
}
```

<sub>Module `probe` line 465</sub>
