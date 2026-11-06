<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`i18n`](../i18n.md) · [index](../index.md)

<a id="type-LocaleInfo"></a>

### `LocaleInfo` (struct)

The localization state.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `requested` | `string` | `""` | — | — | — | The locale the host or the game asked for (empty: none). |
| `active` | `string` | `""` | — | — | — | The table in use (empty: none loaded). |
| `direction` | `string` | `""` | — | — | — | `ltr` or `rtl`. |
| `locales` | `readonly string[]` | `[]` | — | — | — | Every loaded table's locale. |
| `fonts` | `readonly string[]` | `[]` | — | — | — | The UI's font chain, first first. |

```ogt
struct LocaleInfo {
  doc "The localization state."
  field requested String "The locale the host or the game asked for (empty: none)."
  field active String "The table in use (empty: none loaded)."
  field direction String "`ltr` or `rtl`."
  field locales List.String "Every loaded table's locale."
  field fonts List.String "The UI's font chain, first first."
}
```

<sub>`api.ogt` line 15547</sub>
