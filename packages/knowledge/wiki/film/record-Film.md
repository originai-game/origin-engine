<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="record-Film"></a>

### `og.Film` (component)

A film og.film.start made: the conversation it paces and its decks. og.film writes it; read og.film.view.

Token `og.Film`, registered under the schema name `og.Film`; its value type is `Film`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | read only | Its og.story conversation. |
| `decks` | `number` | `0` | — | — | read only | Its decks. |
| `current` | `Entity` | `0` | — | — | read only | The deck playing now (an og.FilmDeck). |
| `node` | `number` | `0` | — | — | read only | The film node playing, as its id (a hash of its title; og.film.view names it); 0 before the first. |
| `pending` | `number` | `-1` | — | — | read only | The choice taken in the window (ChoiceView.index); -1: none yet. |
| `window` | `number` | `0` | — | — | read only | 0 closed, 1 open (choices shown), 2 locked (past film.lock: the cut takes pending, else the default). |
| `paused` | `boolean` | `false` | — | — | read only | Held by og.film.pause (or FilmDesc.paused) until og.film.play. |
| `chapter` | `number` | `0` | — | — | read only | The node of the last chapter entered (its id, as `node`); 0: none. |
| `ended` | `boolean` | `false` | — | — | read only | The last segment played to its cut and the conversation ended. |
| `previous` | `Entity` | `0` | — | — | read only | The deck dissolving out (`og-film-dissolve`), playing on under the current one; 0: none. |
| `dissolve` | `number` | `0` | — | — | read only | The dissolve's length, ticks. |
| `dissolveLeft` | `number` | `0` | — | — | read only | Ticks left of the dissolve; 0: none. |
| `qte` | `readonly number[]` | `[]` | — | — | a list of at most 8 `u32` (`List.u32.8`): reads a frozen copy, writes a whole array, read only | Per choice (ChoiceView.index, the first 8): a quick-time event's progress this window, ticks held or presses. |

```ogt
component Film {
  doc "A film og.film.start made: the conversation it paces and its decks. og.film writes it; read og.film.view."
  field conversation u32 "Its og.story conversation." {
    readonly true
  }
  field decks u32 "Its decks." {
    readonly true
  }
  field current Entity "The deck playing now (an og.FilmDeck)." {
    readonly true
  }
  field node u32 "The film node playing, as its id (a hash of its title; og.film.view names it); 0 before the first." {
    readonly true
  }
  field pending i32 "The choice taken in the window (ChoiceView.index); -1: none yet." {
    default -1
    readonly true
  }
  field window u32 "0 closed, 1 open (choices shown), 2 locked (past film.lock: the cut takes pending, else the default)." {
    readonly true
  }
  field paused bool "Held by og.film.pause (or FilmDesc.paused) until og.film.play." {
    readonly true
  }
  field chapter u32 "The node of the last chapter entered (its id, as `node`); 0: none." {
    readonly true
  }
  field ended bool "The last segment played to its cut and the conversation ended." {
    readonly true
  }
  field previous Entity "The deck dissolving out (`og-film-dissolve`), playing on under the current one; 0: none." {
    readonly true
  }
  field dissolve u32 "The dissolve's length, ticks." {
    readonly true
  }
  field dissolveLeft u32 "Ticks left of the dissolve; 0: none." {
    readonly true
  }
  field qte List.u32.8 "Per choice (ChoiceView.index, the first 8): a quick-time event's progress this window, ticks held or presses." {
    readonly true
  }
}
```

<sub>Module `film` line 26</sub>
