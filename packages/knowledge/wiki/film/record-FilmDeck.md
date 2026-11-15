<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="record-FilmDeck"></a>

### `og.FilmDeck` (component)

An entity og.film plays segments on, with an og.Timeline, an og.VideoPlayer (its `wait` set) and an og.AudioSource. The film's current deck plays; the others hold a segment it may cut to, paused at its first frame.

Token `og.FilmDeck`, registered under the schema name `og.FilmDeck`; its value type is `FilmDeck`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `film` | `Entity` | `0` | — | — | read only | Its film. |
| `node` | `number` | `0` | — | — | read only | The film node whose segment it holds (its id, as og.Film.node); 0: none. |

```ogt
component FilmDeck {
  doc "An entity og.film plays segments on, with an og.Timeline, an og.VideoPlayer (its `wait` set) and an og.AudioSource. The film's current deck plays; the others hold a segment it may cut to, paused at its first frame."
  field film Entity "Its film." {
    readonly true
  }
  field node u32 "The film node whose segment it holds (its id, as og.Film.node); 0: none." {
    readonly true
  }
}
```

<sub>Module `film` line 69</sub>
