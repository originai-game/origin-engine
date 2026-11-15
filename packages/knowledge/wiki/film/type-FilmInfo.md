<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="type-FilmInfo"></a>

### `FilmInfo` (struct)

What og.film.load prepared.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `dialogue` | `string` | `""` | — | — | — | The dialogue. |
| `segments` | `readonly string[]` | `[]` | — | — | — | Its film nodes (those with `og-film`), in file order. |
| `chapters` | `readonly string[]` | `[]` | — | — | — | Its chapters' i18n keys (`og-film-chapter`), in file order. |
| `endings` | `readonly string[]` | `[]` | — | — | — | Its endings (`og-film-ending`), in file order. |

```ogt
struct FilmInfo {
  doc "What og.film.load prepared."
  field dialogue String "The dialogue."
  field segments List.String "Its film nodes (those with `og-film`), in file order."
  field chapters List.String "Its chapters' i18n keys (`og-film-chapter`), in file order."
  field endings List.String "Its endings (`og-film-ending`), in file order."
}
```

<sub>Module `film` line 17</sub>
