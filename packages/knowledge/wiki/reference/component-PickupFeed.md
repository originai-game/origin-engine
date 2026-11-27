<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-PickupFeed"></a>
### `PickupFeed`

A HUD plate of what the player just picked up: its heading, then its entries, each its icon, name and caption with the signed change in its trend's colour; the game owns ordering, expiry and retention, and the empty state says so. Read only.

`import { PickupFeed } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/pickup-feed.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string` | The feed heading; empty uses Pickups. |
| `items?` | `ReadonlyArray<PickupFeedItem>` | Keyed entries. The game owns ordering, expiry and retention; quantity is a signed change. No timers run in this document. |
| `loading?` | `boolean` | Waiting for pickup data; stale entries are hidden. |
| `hideWhenEmpty?` | `boolean` | Hide the plate completely when there are no pickups and not loading. Default false. |

Each `PickupFeedItem`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `name` | `string` | Its name (Iron ore). |
| `icon?` | `string` | An icon of the atlas; empty is none. |
| `quantity?` | `number` | A signed change. |
| `detail?` | `string` | Its caption (Crafting material); empty is none. |

Pack slots: `Feed.plate`.

Fixtures: `default`, `loading`, `overflow`.
