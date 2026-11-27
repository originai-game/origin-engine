<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HudLayout"></a>
### `HudLayout`

The gameplay HUD laid out from the regions a player saved in the HudEditorScreen. The stage is the screen's safe area (`env(hud-inset-*)` plus a margin), the same frame the editor shows, so a region sits where the player put it. It takes no input itself; its elements keep theirs.

`import { HudLayout } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/hud-layout.tsx`).

| Prop | Type | What |
|---|---|---|
| `regions?` | `ReadonlyArray<HudRegion> \| undefined` | The regions as the player placed them (the HudEditorScreen's output). |
| `render?` | `((region: HudRegion) => JSX.Element) \| undefined` | Draws the HUD element a region holds; a region with no element draws nothing. |
| `class?` | `string \| undefined` | Classes after its own. |

Pack slots: `HudLayout.region`.

Functions beside it, imported by name from `@origingame/engine/ui/components`:

- `deserializeHudLayout(text: string, defaults: ReadonlyArray<HudRegion>): HudRegion[]`: Regions read back from `serializeHudLayout` text, merged over `defaults`: a saved region whose id is no longer in `defaults` is dropped, and a default the save lacks keeps its default placement. Malformed text returns `defaults`.
- `serializeHudLayout(regions: ReadonlyArray<HudRegion>): string`: The regions as text for a save or a settings file: a JSON array of the fields that matter.
