<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-selectionRing"></a>

### `og.ui.selectionRing`

A selection ring to the bar of StarCraft II and Baldur's Gate 3: an ellipse on the ground under the unit (the ground circle projected, lying on the terrain or colliders under it with desc.ground), a bright edge and a soft inner glow in its reaction's colour, springing in from larger when selected, thin and dim on hover, or broken into arcs turning slowly.

Parts: SelectionRing.ring[<reaction>][hover], SelectionRing.glow. Accessibility: none (the unit's frame is selected). Probe ids: <id>, its shapes <id>/Box/Ring (or Arc#i) and Glow, each bounded by its points.

```ts
og.ui.selectionRing(world: World, id: string, desc?: SelectionRingDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world: its og.Camera projects the ground, so the calling system declares og.Camera among its reads. |
| `id` | `string` | no | Its id. |
| `desc` | `SelectionRingDesc` | yes | Where and how. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn selectionRing {
  doc "A selection ring to the bar of StarCraft II and Baldur's Gate 3: an ellipse on the ground under the unit (the ground circle projected, lying on the terrain or colliders under it with desc.ground), a bright edge and a soft inner glow in its reaction's colour, springing in from larger when selected, thin and dim on hover, or broken into arcs turning slowly."
  doc "Parts: SelectionRing.ring[<reaction>][hover], SelectionRing.glow. Accessibility: none (the unit's frame is selected). Probe ids: <id>, its shapes <id>/Box/Ring (or Arc#i) and Glow, each bounded by its points."
  host false
  param world World "The world: its og.Camera projects the ground, so the calling system declares og.Camera among its reads."
  param id String "Its id."
  param desc SelectionRingDesc "Where and how."
  returns Void
}
```

<sub>Module `ui` line 6470</sub>
