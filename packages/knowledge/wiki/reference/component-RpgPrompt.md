<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgPrompt"></a>
### `RpgPrompt`

What the player can do now, over the ability bar: a key cap and the action in words (a finisher, an interaction). The game decides when it shows and what the input does; read only.

`import { RpgPrompt } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/prompt.tsx`).

| Prop | Type | What |
|---|---|---|
| `key` | `string` | The input that does it as the game names it (F, Hold E); empty hides the key cap. |
| `text` | `string` | What it does as the game words it (Finisher: Sunder the guard); empty hides the prompt. |
| `disabled` | `boolean` | It cannot be done now: the prompt dims and the reason shows. |
| `reason` | `string` | Why it cannot (Guard holds); shown while disabled. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `disabled`, `overflow`.
