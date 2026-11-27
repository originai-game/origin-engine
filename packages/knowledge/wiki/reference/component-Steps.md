<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Steps"></a>
### `Steps`

A rail of numbered marks: each step is a mark, the reach to the next one (full once the step is done) and its words. It lies along a row and stands up as a column where `column` asks or the container is narrow.

`import { Steps } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/steps.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Optional visible heading; the accessible fallback is Progress steps. |
| `steps?` | `ReadonlyArray<{ id: string; label: string; detail?: string; number: string; state: string; disabled?: boolean }>` | Keyed { id, label, detail, number, state, disabled } rows. number is the game's formatted marker, state upcoming/current/done/error. The game chooses order and allowed navigation. Default []. |
| `column?` | `boolean` | Stack steps vertically instead of a wrapping horizontal sequence. Default false. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `empty`, `column`, `error`, `overflow`.

Actions: `choose` (Choose a step, by its stable key).
