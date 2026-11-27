<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ErrorState"></a>
### `ErrorState`

EmptyState's recess with the failure on a disc tinted and ringed in danger (quieter than the action it leads to): what went wrong, the support reference set apart as a code, then retry (the primary action) and the way out.

`import { ErrorState } from "@origingame/engine/ui/components"` (kit `base`, `packages/engine/src/ui/components/base/error-state.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | Heading; empty uses Something went wrong. |
| `body?` | `string` | The game's explanation and recovery advice. |
| `code?` | `string` | Optional support reference; never a raw exception or credential. |
| `retryLabel?` | `string` | Retry wording; empty uses Try again. |
| `canRetry?` | `boolean` | Offer the retry action. Default true. |
| `busy?` | `boolean` | Retry is in progress; both actions are disabled and a wait is announced. Default false. |
| `dismissLabel?` | `string` | Optional secondary action; empty hides it. |
| `onDismiss?` | `(e: UiEvent, key?: string) => void` | The dismiss event, with the row's key where it comes from a row. |
| `onRetry?` | `(e: UiEvent, key?: string) => void` | The retry event, with the row's key where it comes from a row. |

Fixtures: `default`, `busy`, `terminal`, `overflow`.

Actions: `retry` (Retry the failed operation), `dismiss` (Leave the error state).
