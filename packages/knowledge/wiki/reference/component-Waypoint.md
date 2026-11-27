<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Waypoint"></a>
### `Waypoint`

A place to go: a lit diamond with a dark core (or custom icon) in the kind's colour, optional elevation chevron, its name on a plate under it and the distance on a dark tag. Off screen, a chevron points the way and clamp="edge" holds it inside the screen's edge.

`import { Waypoint } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/waypoint.tsx`).

| Prop | Type | What |
|---|---|---|
| `entity?` | `number` | What it marks, by its bits (0: use at). |
| `at?` | `string` | A world point it marks without an entity (x y z). |
| `height?` | `number` | World units above the point. |
| `kind?` | `string` | What it is: objective (empty, gold), ping (cyan, a ripple), danger (red), ally (green) or loot (violet). |
| `icon?` | `string` | Optional atlas icon rendered inside the marker. |
| `elevation?` | `"above" \| "below" \| "level"` | Optional vertical elevation indicator relative to the player. |
| `label?` | `string` | Its name on a HUD plate under the diamond (The Ninefold Gate); empty is none. It drops when far. |
| `distance?` | `string` | How far it is as the game words it (412), on a dark tag; empty is none. |
| `units?` | `string` | The distance's unit, smaller after it (m). |
| `range?` | `string` | near (a soft halo), mid (empty) or far (the label drops, the distance stays), as the game judges it. |
| `edge?` | `string` | The side of the screen it is off, as the game reads it (left or right): a chevron toward it before the pin; empty is on screen. The anchor holds it inside the screen's edge. |
| `scaleAt?` | `number` | Distance at which the marker is drawn at scale 1. |
| `scaleMin?` | `number` | Minimum world-distance scale. |
| `scaleMax?` | `number` | Maximum world-distance scale. |
| `maxDistance?` | `number` | Maximum world distance before hiding. |
| `occlusion?` | `"fade" \| "hide" \| "none"` | Behaviour when behind world geometry. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `near`, `far`, `ping`, `danger`, `ally`, `loot`, `offscreen`, `offscreenRight`, `overflow`.
