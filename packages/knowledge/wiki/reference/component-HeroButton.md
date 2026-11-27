<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HeroButton"></a>
### `HeroButton`

A hero action's button: its words in capitals and a keycap beside them, on a face that stands on its shadow (primary's primary, premium's gold, secondary's glass, danger's danger); it lifts under a pointer and sinks while held, and shows inert when disabled. With a charge, a bar fills along its foot and the button lights when it is full. Its face, words and charge are the Hero component's pack slots.

`import { HeroButton } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/hero-button.tsx`).

| Prop | Type | What |
|---|---|---|
| `label` | `string` | Its words (End turn). |
| `tone?` | `string` | primary (empty), premium, secondary or danger. |
| `disabled?` | `boolean` | Shown but inert (Waiting). |
| `key?` | `string` | A keycap beside the words (E); empty is none. |
| `charge?` | `number` | A charge filling along its foot, 0 to 1 (a super, a focus), lit and breathing when full; negative is none. Default -1. |
| `onPress?` | `(e: UiEvent) => void` | Press. |

Pack slots: `Hero.charge`, `Hero.charge.fill`, `Hero.frame`, `Hero.frame.ready`, `Hero.frame.…`, `Hero.label`.

Fixtures: `default`, `premium`, `secondary`, `danger`, `charging`, `charged`, `disabled`.
