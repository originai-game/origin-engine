# Base components

Small, composable documents for items, identity, state and progress. Every
prop has a default. They display game-owned data; they do not consume
items, advance cooldowns, grant levels or infer presence.

| Component | Inputs | Intents |
| --- | --- | --- |
| `ItemSlot` | `name`, `icon`, `count`, `rarity`, `cooldown` and `cooldownMax` in seconds, `keyLabel`, `selected`, `disabled`, `locked`, `loading`, `empty`, `showName` | `choose`; keeps an enclosing keyed row's id. Locked, disabled and loading slots do not activate. A cooling or empty slot can be selected for inspection or equipping. |
| `Avatar` | `name`, `portrait` texture, fallback `icon`, `level`, `status` (online, away, busy, offline), optional `statusText`, style-pack `frame` key, `loading` | None; wrap it in a game-owned action when identity is interactive. |
| `EmptyState` | `title`, `body`, `hint`, optional `icon`, `actionLabel`, `disabled` | `activate`; no action when its wording is empty, silent when disabled. An empty title uses translated fallback wording. |
| `ErrorState` | `title`, `body`, support `code`, `retryLabel`, `canRetry`, `busy`, optional `dismissLabel` | `retry`, `dismiss`; busy disables both actions. Terminal errors omit retry. The game owns recovery and supplies safe support wording. |
| `ListItem` | `title`, `subtitle`, `icon` or `image`, trailing `value` and `badge`, `selected`, `disabled`, `loading` | `choose`; preserves the enclosing keyed row's id. Disabled/loading rows cannot activate. Selection also has a check and `aria-pressed`. |
| `Steps` | optional `title`, keyed `steps` of `{ id, label, detail, number, state, disabled }`, `column` | `choose` with the step id; disabled steps cannot activate. The game supplies order, formatted numbers and `upcoming/current/done/error` state; current has `aria-current=step`. |

`--item-slot-size`, `--avatar-size` and `--avatar-radius` control their
dimensions. Every visible piece is a named part. Rarity, frame and status
are exposed as data attributes; presence is also worded so it does not rely
on colour. Native buttons supply focus, hover, press and keyboard/pad
activation. A pack may replace the look without changing the documents.

Component CSS keeps dimensions and placement. Shared `data-layout` and
`data-gap` primitives arrange the boxes; the final role in each part list
(`card`, `row`, `cell`, `badge`, `headline`, `caption`, `dot`, `track`,
`fill`) supplies its dress. Existing semantic parts remain available.
Loading uses the shared Spinner or a clipped `skeleton`/`sheen` with
`data-motion="shimmer"`, one band of light crossing the whole group;
ribbons use shared enter/stamp motion. Spinner always shows its track ring;
`size` is its width, and a host that holds nothing else while it waits (a
portrait, a slot) sets `--spinner-size` on the box round it. Ribbon's box
draws nothing: put it in the corner it names of a positioned card that
clips (`overflow: hidden`), so the card's corner cuts the strip's ends. Packs own
those role and motion tokens, not a component's private animations.

An element wears one role, so a part that a role dresses is named for it:
a name is `headline` (Avatar, ListItem, Steps, EmptyState, ErrorState) or
`caption` (ItemSlot, Currency, Pips), with no second `name`, `title` or
`label` part beside it. Steps keeps `title` for its heading, which wears
the title role a step down.

Avatar's `showName` defaults to true; set it to false beside a player heading to hide only the caption while preserving the figure's accessible name, level and status.

Review: the RPG sheet's `primitives` page in the default style shows ready,
selected/cooling, locked, unavailable, loading and empty slots; portrait and
fallback avatars, level, four presence states and loading.

The components sheet also reviews the four state/navigation documents with
`style:default;sample:<member>:<fixture>` at desktop and `phonePortrait`:

| Member | Fixtures |
| --- | --- |
| `emptyState` | `default`, `minimal`, `disabled`, `overflow` |
| `errorState` | `default`, `busy`, `terminal`, `overflow` |
| `listItem` | `default`, `disabled`, `loading`, `overflow` |
| `steps` | `default`, `empty`, `column`, `error`, `overflow` |

These are small building blocks, not virtualised lists. ListItem wraps long
titles and keeps its trailing value over its badge at the row's end at any
width; the check or the loading spinner stays beside the content. Steps is
a rail of marks, each joined to the next by a reach that fills once the
step is done; it lies along a row and stands up as a column with `column`
or in a narrow container. State words and marks make progress readable
without colour. EmptyState and ErrorState are a recess where the content
would be, with their mark on a disc. Every visible piece has a named part. Buttons
inherit shared hover/focus/press/disabled states; the game owns all
selection, progress and retry transitions.
