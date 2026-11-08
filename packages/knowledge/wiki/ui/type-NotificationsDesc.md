<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NotificationsDesc"></a>

### `NotificationsDesc` (desc struct)

A stack of notifications (toasts) anchored in the safe area.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `items` | `readonly NotificationItem[]` | `[]` | — | — | — | The game's notifications, oldest first; past their time they leave on their own. |
| `anchor` | `Anchor` | `"topRight"` | — | — | — | Where the stack sits; the default is the top right (the top centre on a phone in portrait). |
| `max` | `number` | `0` | — | — | — | The most shown at once (later ones queue); 0 is 3 (2 on a phone). |
| `width` | `number` | `0` | ≥ 0 | `cssPx` | — | A card's width; 0 is 380 (the safe area's width less its gutters on a phone in portrait). |
| `dismissLabel` | `string` | `"Dismiss"` | — | — | — | What a screen reader calls the swipe or press that dismisses one. |

```ogt
struct NotificationsDesc {
  doc "A stack of notifications (toasts) anchored in the safe area."
  desc true
  field items List.NotificationItem "The game's notifications, oldest first; past their time they leave on their own."
  field anchor Anchor "Where the stack sits; the default is the top right (the top centre on a phone in portrait)." {
    default topRight
  }
  field max u32 "The most shown at once (later ones queue); 0 is 3 (2 on a phone)."
  field width f32 "A card's width; 0 is 380 (the safe area's width less its gutters on a phone in portrait)." {
    unit cssPx
    min 0
  }
  field dismissLabel String "What a screen reader calls the swipe or press that dismisses one." {
    default "Dismiss"
  }
}
```

<sub>Module `ui` line 5240</sub>
