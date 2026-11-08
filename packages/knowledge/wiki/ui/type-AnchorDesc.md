<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-AnchorDesc"></a>

### `AnchorDesc` (desc struct)

An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `anchor` | `Anchor` | `"topLeft"` | — | — | — | Where it sits. |
| `margin` | `number \| null` | `null` | — | — | — | The inset from the safe area's edges, the same on every edge: a margin wide enough to keep a top-left HUD over a centred portrait play field also pushes it that far down (use offsetX for the sideways shift, or a width); null is the density's gutter. |
| `offsetX` | `number` | `0` | — | `cssPx` | — | Moves it right after placing. |
| `offsetY` | `number` | `0` | — | `cssPx` | — | Moves it down after placing. |
| `direction` | `Direction` | `"row"` | — | — | — | Which way it lays out its children. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between children. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside its edges. |
| `align` | `Align` | `"start"` | — | — | — | Where children sit along the main axis. |
| `cross` | `Cross` | `"start"` | — | — | — | Where children sit across it. |
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null draws none. |
| `radius` | `number` | `0` | — | `cssPx` | — | Corner radius. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the content. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits the content. |
| `over` | `boolean` | `false` | — | — | — | Draw over dialogs and spotlights (a tutorial callout, a dragged item); the default draws under them. |
| `tooltip` | `boolean` | `false` | — | — | — | Draw in the tooltip layer, over every other layer including the toast stack and over groups: a card the player opened (og.ui.tooltipCard). |
| `avoid` | `boolean` | `false` | — | — | — | Step clear of the other anchored groups and the host's control, to the nearest free place on the safe area: a popup placed at a point (a drift score over the car) that must not cover the map or the tower. |
| `frame` | `StylePart \| null` | `null` | — | — | — | The style frame its background takes; null is the panel's. |
| `instant` | `boolean` | `false` | — | — | — | Shown settled the first time it is built: no style entrance or stagger, its controls answering at once. For the screen a game opens on while it is that: the web draws it behind the play gate, where an entrance would stay part-way until the tap. |

```ogt
struct AnchorDesc {
  doc "An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls)."
  desc true
  field anchor Anchor "Where it sits."
  field margin Option.f32 "The inset from the safe area's edges, the same on every edge: a margin wide enough to keep a top-left HUD over a centred portrait play field also pushes it that far down (use offsetX for the sideways shift, or a width); null is the density's gutter."
  field offsetX f32 "Moves it right after placing." {
    unit cssPx
  }
  field offsetY f32 "Moves it down after placing." {
    unit cssPx
  }
  field direction Direction "Which way it lays out its children."
  field gap f32 "Space between children." {
    unit cssPx
  }
  field padding Insets "Space inside its edges."
  field align Align "Where children sit along the main axis."
  field cross Cross "Where children sit across it."
  field background Option.Paint "Its fill; null draws none."
  field radius f32 "Corner radius." {
    unit cssPx
  }
  field width Option.Size "Its width; null fits the content."
  field height Option.Size "Its height; null fits the content."
  field over bool "Draw over dialogs and spotlights (a tutorial callout, a dragged item); the default draws under them."
  field tooltip bool "Draw in the tooltip layer, over every other layer including the toast stack and over groups: a card the player opened (og.ui.tooltipCard)."
  field avoid bool "Step clear of the other anchored groups and the host's control, to the nearest free place on the safe area: a popup placed at a point (a drift score over the car) that must not cover the map or the tower."
  field frame Option.StylePart "The style frame its background takes; null is the panel's."
  field instant bool "Shown settled the first time it is built: no style entrance or stagger, its controls answering at once. For the screen a game opens on while it is that: the web draws it behind the play gate, where an entrance would stay part-way until the tap."
}
```

<sub>Module `ui` line 825</sub>
