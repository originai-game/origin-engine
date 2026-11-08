<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NotificationItem"></a>

### `NotificationItem` (desc struct)

One notification of a stack, as the game's viewmodel holds it. The game appends one when something happens and drops the keys the stack reports expired or dismissed.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key, stable while it lives (also its probe id): posting a key again with a higher count merges it (×3). |
| `title` | `string` | `""` | — | — | — | The headline. |
| `body` | `string` | `""` | — | — | — | A line under it; empty is none (two lines at most, then an ellipsis). |
| `eyebrow` | `string` | `""` | — | — | — | A small line over the title (Achievement unlocked); empty is none. |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas in its well; empty draws the tone's mark. |
| `image` | `Texture \| null` | `null` | — | — | — | An art slot in the well instead of the icon (an item, an avatar). |
| `tone` | `Tone` | `"neutral"` | — | — | — | Its tone: the accent edge and the icon well. |
| `variant` | `NotificationVariant` | `"standard"` | — | — | — | What it is. |
| `rarity` | `Rarity` | `"none"` | — | — | — | A reward's tier: the well's glow and pips. |
| `at` | `number` | `0` | — | — | — | og.ui.tick() when it was posted: its time runs from when it first shows (a queued one waits). |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long it shows; 0 is the variant's time. |
| `sticky` | `boolean` | `false` | — | — | — | It stays until dismissed or its action is pressed. |
| `count` | `number` | `0` | — | — | — | How many times it happened (a ×3 chip from 2 on). |
| `progress` | `number \| null` | `null` | — | `ratio` | — | A progress bar under the body, 0 to 1 (an achievement 3 of 10 of the way). |
| `progressText` | `string` | `""` | — | — | — | The progress in words (3 / 10); empty is none. |
| `points` | `string` | `""` | — | — | — | A value at the end (+50 G, 120 XP); empty is none. |
| `action` | `string` | `""` | — | — | — | A button's label (Accept, View); empty is none. |

```ogt
struct NotificationItem {
  doc "One notification of a stack, as the game's viewmodel holds it. The game appends one when something happens and drops the keys the stack reports expired or dismissed."
  desc true
  field key String "Its key, stable while it lives (also its probe id): posting a key again with a higher count merges it (×3)."
  field title String "The headline."
  field body String "A line under it; empty is none (two lines at most, then an ellipsis)."
  field eyebrow String "A small line over the title (Achievement unlocked); empty is none."
  field icon String "An icon of the atlas in its well; empty draws the tone's mark."
  field image Option.Texture "An art slot in the well instead of the icon (an item, an avatar)."
  field tone Tone "Its tone: the accent edge and the icon well."
  field variant NotificationVariant "What it is."
  field rarity Rarity "A reward's tier: the well's glow and pips."
  field at u32 "og.ui.tick() when it was posted: its time runs from when it first shows (a queued one waits)."
  field durationMs f32 "How long it shows; 0 is the variant's time." {
    unit ms
    min 0
  }
  field sticky bool "It stays until dismissed or its action is pressed."
  field count u32 "How many times it happened (a ×3 chip from 2 on)."
  field progress Option.f32 "A progress bar under the body, 0 to 1 (an achievement 3 of 10 of the way)." {
    unit ratio
  }
  field progressText String "The progress in words (3 / 10); empty is none."
  field points String "A value at the end (+50 G, 120 XP); empty is none."
  field action String "A button's label (Accept, View); empty is none."
}
```

<sub>Module `ui` line 5213</sub>
