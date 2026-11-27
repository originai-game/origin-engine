<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ShooterPrompt"></a>
### `ShooterPrompt`

What the player can do with what is in front of them, under the aim: the key cap and the action in words. The game decides when it shows and what the input does; read only. The key cap's scrolling overflow (the svelte's -og-scroll marquee) has no TSX surface: long values truncate.

`import { ShooterPrompt } from "@origingame/engine/ui/components"` (kit `shooter`, `packages/engine/src/ui/components/shooter/prompt.tsx`).

| Prop | Type | What |
|---|---|---|
| `key?` | `string` | The input that does it as the game names it (E, X, Hold F); empty hides the key cap. |
| `text` | `string` | What it does as the game words it (Open supply bin); empty hides the prompt. |
| `disabled?` | `boolean` | It cannot be done now: the prompt dims and the reason shows. |
| `reason?` | `string` | Why it cannot (Locked); shown while disabled. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `disabled`, `empty`, `overflow`.
