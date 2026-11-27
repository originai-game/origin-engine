<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-OverlayBar"></a>
### `OverlayBar`

A map's overlays over the scene: a legend for the selected overlay (its name, its ramp low to high and the words at its ends) over a bar of buttons, one per overlay, its picture or icon beside its name; the bar scrolls along when the buttons run over. Read only; the buttons report their choice. The buttons' pressed state is worded, as the style words it; a pack's picture fills the button's face.

`import { OverlayBar } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/overlay-bar.tsx`).

| Prop | Type | What |
|---|---|---|
| `overlays?` | `ReadonlyArray<OverlayBarOverlay>` | The map overlays, keyed rows. |
| `selected?` | `string` | The id of the overlay shown (its legend over the bar, its button lit and named); empty is none. |
| `label?` | `string` | What a screen reader calls the bar; empty is Overlays. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Show an overlay. |

Each `OverlayBarOverlay`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | Its name (Heat). |
| `icon?` | `string` | An icon name shown when there is no image. |
| `image?` | `Texture \| 0` | A texture handle (0: none). |
| `ramp?` | `string` | The legend's gradient stops low to high (#2b6cb0, #f6e05e, #e53e3e). |
| `low?` | `string` | The word at the ramp's low end. |
| `high?` | `string` | The word at the ramp's high end. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `none`.
