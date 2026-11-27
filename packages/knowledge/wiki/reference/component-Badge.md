<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Badge"></a>
### `Badge`

A small status tag: words, a count capped at `max+`, or a lone dot, in one of seven tones. The pack dresses it through `Badge.frame` and `Badge.frame.<tone>` (`Badge.dot` for the dot). Read only.

`import { Badge } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/badge.tsx`).

| Prop | Type | What |
|---|---|---|
| `tone?` | `BadgeTone \| undefined` | Its colour: `neutral` (the default), `accent`, `success`, `warn`, `danger`, `info` or `gold`. |
| `count?` | `number \| undefined` | A count to show (unread mail, new items); 0 hides the badge unless `showZero`. |
| `max?` | `number \| undefined` | Counts above this read `max+` (99+). Default 99. |
| `showZero?` | `boolean \| undefined` | Show a count of 0 rather than hiding. |
| `dot?` | `boolean \| undefined` | Only a dot, no words (something new here). |
| `label?` | `string \| undefined` | What a screen reader says (3 unread); the visible text when absent. |
| `class?` | `string \| undefined` | Classes after its own (a placement on a corner). |
| `children?` | `any` | Words instead of a count (New, -20%). |

Pack slots: `Badge.dot.…`, `Badge.frame.…`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `words`, `accent`, `count`, `capped`, `gold`, `dot`.
