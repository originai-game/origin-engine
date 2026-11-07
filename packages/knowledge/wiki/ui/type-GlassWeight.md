<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-GlassWeight"></a>

### `GlassWeight` (enum)

How heavy a glass surface is: flat glass, clear and coloured by what passes under it, with no rim, sheen or refraction.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No weight: the backdrop's own fields. |
| `"thin"` | 10 px of blur and the lightest tint: icons and play discs over art, never text (it does not hold text at 4.5:1). |
| `"regular"` | 22 px: bars, tab bars, capsules; holds the scheme's text at 4.5:1. |
| `"thick"` | 32 px and a denser tint: menus, popovers, dialogs, sheets, toasts. |

```ogt
enum GlassWeight {
  doc "How heavy a glass surface is: flat glass, clear and coloured by what passes under it, with no rim, sheen or refraction."
  variant none "No weight: the backdrop's own fields."
  variant thin "10 px of blur and the lightest tint: icons and play discs over art, never text (it does not hold text at 4.5:1)."
  variant regular "22 px: bars, tab bars, capsules; holds the scheme's text at 4.5:1."
  variant thick "32 px and a denser tint: menus, popovers, dialogs, sheets, toasts."
}
```

<sub>Module `ui` line 7200</sub>
