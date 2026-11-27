<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-IdlePrestige"></a>
### `IdlePrestige`

The prestige panel of an idle game: the currency in a socket glowing in the epic purple, what the player holds and, in that purple, what a reset would give; each changed bonus now and after the reset; and the requirement with its leaning bar, green once met, and Ready or Not yet in words. Prestige reports request; the game then sets confirming and the panel asks inline, Reset (confirm) and Cancel (cancel, or Escape and pad B while asking). Cancel comes first so no key resets by accident. The game decides whether a reset is allowed and what it gives.

`import { IdlePrestige } from "@origingame/engine/ui/components"` (kit `idle`, `packages/engine/src/ui/components/idle/prestige.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The panel's heading; empty takes the group's (Prestige). |
| `currency` | `string` | The prestige currency's name (Souls). |
| `icon` | `string` | The prestige currency's icon. |
| `held` | `string` | How much of it the player holds now, as the game formats it. |
| `gain` | `string` | How much a reset now would give, as the game formats it. |
| `rows?` | `ReadonlyArray<{ id: string; icon: string; label: string; now: string; after: string }>` | What changes, keyed rows of { id, icon, label, now, after }: now the value before the reset and after the value it would be after, both as the game words them. |
| `requirement` | `string` | What a reset needs, as the game words it (Reach 1M gold); empty hides it. |
| `progress?` | `number` | Toward the requirement, 0 to 1, for its bar; negative hides the bar. Default -1. |
| `progressText` | `string` | That progress in words (640K / 1M); empty shows the percent. |
| `met?` | `boolean` | The requirement is met: Prestige is enabled and the requirement reads Ready (else Not yet). |
| `confirming?` | `boolean` | The game is asking the player to confirm (set after request): Reset and Cancel stand in for Prestige. |
| `onCancel?` | `(e: UiEvent, key?: string) => void` | The cancel event, with the row's key where it comes from a row. |
| `onConfirm?` | `(e: UiEvent, key?: string) => void` | The confirm event, with the row's key where it comes from a row. |
| `onRequest?` | `(e: UiEvent, key?: string) => void` | The request event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Panel.frame`, `Row.frame`.

Fixtures: `default`, `disabled`, `confirming`, `empty`, `overflow`.

Actions: `back` (Cancel).
