<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MenuEntryKind"></a>

### `MenuEntryKind` (enum)

What a menu's entry is.

A string: one of the variants below. The zero value is `"item"`.

| Variant | Doc |
|---|---|
| `"item"` | A command (the default). |
| `"check"` | A command with a check mark (menuitemcheckbox). |
| `"radio"` | One of a group (menuitemradio). |
| `"separator"` | A hairline between groups. |
| `"header"` | A group's caption. |

```ogt
enum MenuEntryKind {
  doc "What a menu's entry is."
  variant item "A command (the default)."
  variant check "A command with a check mark (menuitemcheckbox)."
  variant radio "One of a group (menuitemradio)."
  variant separator "A hairline between groups."
  variant header "A group's caption."
}
```

<sub>Module `ui` line 2286</sub>
