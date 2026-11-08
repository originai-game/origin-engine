<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NotificationsResult"></a>

### `NotificationsResult` (struct)

What a notification stack answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `expired` | `readonly string[]` | `[]` | — | — | — | Keys whose time ran out this tick: drop them from the viewmodel. |
| `dismissed` | `readonly string[]` | `[]` | — | — | — | Keys the player dismissed (a press on the card, a swipe, Back while it had focus). |
| `action` | `string \| null` | `null` | — | — | — | The key whose action button was pressed this tick. |
| `shown` | `readonly string[]` | `[]` | — | — | — | The keys on screen, newest first. |

```ogt
struct NotificationsResult {
  doc "What a notification stack answered this tick."
  field expired List.String "Keys whose time ran out this tick: drop them from the viewmodel."
  field dismissed List.String "Keys the player dismissed (a press on the card, a swipe, Back while it had focus)."
  field action Option.String "The key whose action button was pressed this tick."
  field shown List.String "The keys on screen, newest first."
}
```

<sub>Module `ui` line 5280</sub>
