<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-InteractionPrompt"></a>
### `InteractionPrompt`

What the player can do with what is in front of them: a key cap (or device-adaptive action glyph) beside a frosted glass plate, the verb in condensed capitals and the object under it; a hold's ring round the cap.

`import { InteractionPrompt } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/interaction-prompt.tsx`).

| Prop | Type | What |
|---|---|---|
| `entity?` | `number` | What it is over, by its bits (0: use at). |
| `at?` | `string` | A world point it sits over without an entity (x y z). |
| `height?` | `number` | World units above the point. |
| `action?` | `string` | The action whose glyph the key cap shows, its binding on the device used last (interact); empty shows key. |
| `key?` | `string` | The key as the game names it (E), shown when action is empty. |
| `pad?` | `boolean` | A gamepad's face button: a round cap. |
| `verb` | `string` | What it does, in condensed capitals (Open, Talk, Revive); it always fits whole. |
| `name?` | `string` | What it is under the verb (Gilded Chest); empty is none. |
| `pressed?` | `boolean` | The key is down: the cap sinks 2 px and lights. |
| `hold?` | `number` | A hold's progress 0 to 1: an accent ring round the cap on a dark disc (1 is done: the cap lights); negative draws none. |
| `disabled?` | `boolean` | It cannot be used now: the cap greys and the reason shows in red with a lock. |
| `reason?` | `string` | Why it cannot (Needs the Warden's key). |
| `maxDistance?` | `number` | Hidden past this many world units (6). |
| `scaleAt?` | `number` | Distance at which the prompt is drawn at scale 1. |
| `scaleMin?` | `number` | Minimum world-distance scale. |
| `scaleMax?` | `number` | Maximum world-distance scale. |
| `occlusion?` | `"fade" \| "hide" \| "none"` | Behaviour when behind world geometry. |
| `onPress?` | `(e: UiEvent) => void` | Optional tap/click handler for direct touch or pointer interaction. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `pressed`, `hold`, `done`, `disabled`, `gamepad`, `action`, `overflow`.
