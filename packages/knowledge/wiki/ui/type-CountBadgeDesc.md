<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CountBadgeDesc"></a>

### `CountBadgeDesc` (desc struct)

A count badge on the corner of the container it is built in (a tab, an inventory icon, a mailbox button).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `BadgeKind` | `"count"` | — | — | — | What it shows. |
| `count` | `number` | `0` | — | — | — | The count; 0 hides a count badge. |
| `max` | `number` | `0` | — | — | — | Past this it shows max+; 0 is 99. |
| `text` | `string` | `""` | — | — | — | The label badge's word. |
| `tone` | `Tone` | `"danger"` | — | — | — | Its tone; the default is danger, the convention for unread counts. |
| `corner` | `Anchor` | `"topRight"` | — | — | — | The corner of the container it sits on: topRight is the end side's top (mirrored right to left). |
| `pulse` | `boolean` | `true` | — | — | — | Pulse once when the count rises (the default). |
| `name` | `string` | `""` | — | — | — | What a screen reader says (3 unread); empty is the count. |

```ogt
struct CountBadgeDesc {
  doc "A count badge on the corner of the container it is built in (a tab, an inventory icon, a mailbox button)."
  desc true
  field kind BadgeKind "What it shows."
  field count u32 "The count; 0 hides a count badge."
  field max u32 "Past this it shows max+; 0 is 99."
  field text String "The label badge's word."
  field tone Tone "Its tone; the default is danger, the convention for unread counts." {
    default danger
  }
  field corner Anchor "The corner of the container it sits on: topRight is the end side's top (mirrored right to left)." {
    default topRight
  }
  field pulse bool "Pulse once when the count rises (the default)." {
    default true
  }
  field name String "What a screen reader says (3 unread); empty is the count."
}
```

<sub>Module `ui` line 5513</sub>
