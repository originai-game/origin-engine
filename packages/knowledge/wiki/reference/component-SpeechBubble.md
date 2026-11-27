<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-SpeechBubble"></a>
### `SpeechBubble`

A line spoken over a head: a frosted glass plate, the speaker in their colour, the line wrapping cleanly and a thin line of the time left. Bubbles stack clear of each other (the anchor's declutter).

`import { SpeechBubble } from "@origingame/engine/ui/components"` (kit `world`, `packages/engine/src/ui/components/world/speech-bubble.tsx`).

| Prop | Type | What |
|---|---|---|
| `entity?` | `number` | Who speaks, by its bits (0: use at). |
| `at?` | `string` | A world point it speaks from without an entity (x y z). |
| `height?` | `number` | World units above the point (over the head). |
| `speaker?` | `string` | Who speaks, over the line in their reaction's ink; empty is none. |
| `reaction?` | `string` | The speaker's colour: friendly (empty), hostile, neutral or player. |
| `text` | `string` | The line; it wraps at 27 characters. |
| `shout?` | `boolean` | A shout: a warm rim and a bolder line. |
| `left?` | `number` | How much of the line's time is left, 0 to 1, a thin line along the bubble's foot; negative draws none. The game times the line out and removes the bubble. |
| `maxDistance?` | `number` | Maximum world distance before hiding. |
| `scaleAt?` | `number` | Distance at which the bubble is drawn at scale 1. |
| `scaleMin?` | `number` | Minimum world-distance scale. |
| `scaleMax?` | `number` | Maximum world-distance scale. |
| `occlusion?` | `"fade" \| "hide" \| "none"` | Behaviour when behind world geometry. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `short`, `shout`, `nameless`, `overflow`.
