<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-EmptyState"></a>
### `EmptyState`

Where content would be, there is a recess: a mark on a disc, why the area is empty, and the one thing to do about it.

`import { EmptyState } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/empty-state.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Nothing here yet. |
| `body?` | `string` | Why this area is empty. |
| `hint?` | `string` | An optional next step, in quieter text. |
| `icon?` | `string` | An icon above the heading; empty hides it. Default "bag". |
| `actionLabel?` | `string` | The optional action's words; empty hides the button. |
| `disabled?` | `boolean` | The action is unavailable. Default false. |
| `onActivate?` | `(e: UiEvent, key?: string) => void` | The activate event, with the row's key where it comes from a row. |

Fixtures: `default`, `minimal`, `disabled`, `overflow`.

Actions: `activate` (Take the suggested action).
