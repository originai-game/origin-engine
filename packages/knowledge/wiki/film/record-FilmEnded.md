<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="record-FilmEnded"></a>

### `og.FilmEnded` (event)

A film's last segment reached its cut with the conversation ended.

Token `og.FilmEnded`, registered under the schema name `og.FilmEnded`; its value type is `FilmEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `film` | `Entity` | `0` | — | — | — | The film. |
| `node` | `string` | `""` | — | — | — | The last node (at most 63 bytes). |
| `ending` | `string` | `""` | — | — | — | Its `og-film-ending`, empty without one (at most 63 bytes). |

```ogt
event FilmEnded {
  doc "A film's last segment reached its cut with the conversation ended."
  field film Entity "The film."
  field node String "The last node (at most 63 bytes)."
  field ending String "Its `og-film-ending`, empty without one (at most 63 bytes)."
}
```

<sub>Module `film` line 118</sub>
