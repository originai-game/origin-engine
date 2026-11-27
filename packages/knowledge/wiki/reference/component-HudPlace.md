<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HudPlace"></a>
### `HudPlace`

One HUD element placed where a region says: its centre at the region's `x`, `y` across the safe area, at its `scale` and `opacity`, absent while `hidden`. Dressed by `HudLayout.region`.

`import { HudPlace } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/hud-layout.tsx`).

| Prop | Type | What |
|---|---|---|
| `region` | `HudRegion` | The region to place the children in. |
| `class?` | `string \| undefined` | Classes after its own. |
| `children?` | `any` |  |

Pack slots: `HudLayout.region`.
