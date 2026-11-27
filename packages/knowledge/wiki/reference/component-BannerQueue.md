<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-BannerQueue"></a>
### `BannerQueue`

A stack of non-blocking banners at the top centre of the screen (achievements, milestones, a quest unlocked). Each keyed item enters with the unlock cue and plays its exit while `leaving`; the game decides when each one arrives and goes. The stack never takes input. Each banner is dressed by `BannerQueue.item`.

`import { BannerQueue } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/banner-queue.tsx`).

| Prop | Type | What |
|---|---|---|
| `items?` | `ReadonlyArray<BannerItem> \| undefined` | The banners showing, oldest first; the game owns timing and the queue's length. |
| `sound?` | `boolean \| undefined` | Play the unlock cue as each banner enters. Default true. |
| `class?` | `string \| undefined` | Classes after its own (a placement). |

Each `BannerItem`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | A stable id; a new id enters, a removed one leaves. |
| `icon?` | `Texture \| 0` | A texture (og.load.texture); 0 or absent draws `glyph` instead. |
| `glyph?` | `string` | An icon name, in art's place. |
| `kicker?` | `string` | The small line above the title (Achievement unlocked, New quest). |
| `title` | `string` | The banner's headline. |
| `subtitle?` | `string` | A line under the title; empty hides it. |
| `tone?` | `BannerTone` | The kicker's and icon's colour. |
| `leaving?` | `boolean` | Set while the banner plays its exit, before the game removes it. |

Pack slots: `BannerQueue.item`.

Fixtures: `default`, `stack`, `long`.
