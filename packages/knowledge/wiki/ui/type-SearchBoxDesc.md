<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SearchBoxDesc"></a>

### `SearchBoxDesc` (desc struct)

A search box and its suggestions.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `placeholder` | `string` | `""` | — | — | — | Shown while it is empty; empty is `Search`. |
| `suggestions` | `readonly string[]` | `[]` | — | — | — | What to suggest for the query now (the game's, already filtered), or with filter every candidate. |
| `filter` | `boolean` | `false` | — | — | — | Filter suggestions itself: those containing the query, case and accents ignored, those starting with it first. |
| `recent` | `readonly string[]` | `[]` | — | — | — | Shown while the query is empty and it has the focus (recent searches); empty for none. |
| `maxSuggestions` | `number` | `6` | — | — | — | The most shown. |
| `label` | `string` | `""` | — | — | — | Its accessible name; empty is the placeholder. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |

```ogt
struct SearchBoxDesc {
  doc "A search box and its suggestions."
  desc true
  field placeholder String "Shown while it is empty; empty is `Search`."
  field suggestions List.String "What to suggest for the query now (the game's, already filtered), or with filter every candidate."
  field filter bool "Filter suggestions itself: those containing the query, case and accents ignored, those starting with it first."
  field recent List.String "Shown while the query is empty and it has the focus (recent searches); empty for none."
  field maxSuggestions u32 "The most shown." {
    default 6
  }
  field label String "Its accessible name; empty is the placeholder."
  field width Option.Size "Its width; null grows."
  field disabled bool "Shown but inert."
}
```

<sub>Module `ui` line 3916</sub>
