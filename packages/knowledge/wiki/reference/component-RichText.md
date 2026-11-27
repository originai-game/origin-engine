<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RichText"></a>
### `RichText`

Structured inline text for tooltips, card rules and item descriptions: runs of words in a tone, bold or italic, inline icons, and keywords that take focus and show their gloss in a tooltip (`RichText.keyword`, `RichText.gloss`). The runs wrap as one paragraph. It decides nothing: the game writes the runs.

`import { RichText } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/rich-text.tsx`).

| Prop | Type | What |
|---|---|---|
| `spans` | `ReadonlyArray<RichSpan>` | The runs in reading order. |
| `class?` | `string \| undefined` | Classes after its own (type size, alignment). |
| `onKeyword?` | `((e: UiEvent, keyword: string) => void) \| undefined` | A keyword focused or hovered (the game may show a glossary page). |

Each `RichSpan`:

| Field | Type | What |
|---|---|---|
| `text?` | `string \| undefined` | Its words (empty with `icon` alone). |
| `tone?` | `RichTone \| undefined` | Its colour; `fg` (the default) reads in the ground's ink. |
| `bold?` | `boolean \| undefined` |  |
| `italic?` | `boolean \| undefined` |  |
| `icon?` | `string \| undefined` | An atlas icon drawn inline, one em square, before the words. |
| `keyword?` | `string \| undefined` | A keyword the game explains: underlined, focusable, and its `gloss` shown in a tooltip on hover or focus. |
| `gloss?` | `string \| undefined` | What the keyword means (Burn: 4 damage a second for 3 s). |

Pack slots: `RichText.gloss`, `RichText.keyword`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `rarity`.
