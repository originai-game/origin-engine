<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-overlayBar"></a>

### `og.ui.overlayBar`

A map's overlay picker (Cities: Skylines' info views, Civilization's lenses): a row of round buttons, each its overlay's art, the shown one lit with its name, and over the bar the shown overlay's legend, its name, its ramp of colours from low to high with the two words at its ends. A press shows an overlay, a second press on the shown one turns it off; a finger, keys and a pad reach each button. The game draws the overlay on its map; the bar draws the picker and the legend only. On a phone the bar scrolls sideways and keeps its buttons whole touch targets.

Parts: Overlay.bar, Overlay.button[selected], Overlay.legend, Overlay.ramp. Slots: each overlay's art (itemIcon). Sounds: toggle on a press. Accessibility: the bar a toolbar, each button a toggle button with its pressed state; the legend a status named with the overlay and its ends. Probe ids: <id>/Bar/<key>, <id>/Legend.

```ts
og.ui.overlayBar(id: string, desc?: OverlayBarDesc): OverlayBarResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `OverlayBarDesc` | yes | The overlays and the shown one. |

**Returns** `OverlayBarResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn overlayBar {
  doc "A map's overlay picker (Cities: Skylines' info views, Civilization's lenses): a row of round buttons, each its overlay's art, the shown one lit with its name, and over the bar the shown overlay's legend, its name, its ramp of colours from low to high with the two words at its ends. A press shows an overlay, a second press on the shown one turns it off; a finger, keys and a pad reach each button. The game draws the overlay on its map; the bar draws the picker and the legend only. On a phone the bar scrolls sideways and keeps its buttons whole touch targets."
  doc "Parts: Overlay.bar, Overlay.button[selected], Overlay.legend, Overlay.ramp. Slots: each overlay's art (itemIcon). Sounds: toggle on a press. Accessibility: the bar a toolbar, each button a toggle button with its pressed state; the legend a status named with the overlay and its ends. Probe ids: <id>/Bar/<key>, <id>/Legend."
  host false
  param id String "Its id."
  param desc OverlayBarDesc "The overlays and the shown one."
  returns OverlayBarResult
}
```

<sub>Module `ui` line 4290</sub>
