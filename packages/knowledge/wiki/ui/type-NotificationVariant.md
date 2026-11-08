<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NotificationVariant"></a>

### `NotificationVariant` (enum)

What a notification is, which sets its look, sound and default time.

A string: one of the variants below. The zero value is `"standard"`.

| Variant | Doc |
|---|---|
| `"standard"` | A plain message: a tone's accent and icon well (4 s). |
| `"achievement"` | An achievement: a medal well in gold, an eyebrow over the title, points (6 s); plays reward. |
| `"reward"` | An item or currency received: the icon well glows in the rarity's colour (5 s); plays reward. |
| `"alert"` | A warning that must be read: danger accent, read at once by a screen reader (8 s); plays warning. |
| `"social"` | A friend, invite or message: a round avatar slot (5 s). |

```ogt
enum NotificationVariant {
  doc "What a notification is, which sets its look, sound and default time."
  variant standard "A plain message: a tone's accent and icon well (4 s)."
  variant achievement "An achievement: a medal well in gold, an eyebrow over the title, points (6 s); plays reward."
  variant reward "An item or currency received: the icon well glows in the rarity's colour (5 s); plays reward."
  variant alert "A warning that must be read: danger accent, read at once by a screen reader (8 s); plays warning."
  variant social "A friend, invite or message: a round avatar slot (5 s)."
}
```

<sub>Module `ui` line 5193</sub>
