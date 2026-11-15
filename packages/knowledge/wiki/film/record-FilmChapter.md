<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="record-FilmChapter"></a>

### `og.FilmChapter` (event)

A film entered a chapter's node (`og-film-chapter`): a place to save.

Token `og.FilmChapter`, registered under the schema name `og.FilmChapter`; its value type is `FilmChapter`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `film` | `Entity` | `0` | — | — | — | The film. |
| `node` | `string` | `""` | — | — | — | The node (at most 63 bytes). |
| `chapter` | `string` | `""` | — | — | — | The chapter's i18n key (at most 63 bytes). |

```ogt
event FilmChapter {
  doc "A film entered a chapter's node (`og-film-chapter`): a place to save."
  field film Entity "The film."
  field node String "The node (at most 63 bytes)."
  field chapter String "The chapter's i18n key (at most 63 bytes)."
}
```

<sub>Module `film` line 111</sub>
