<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="type-FilmDesc"></a>

### `FilmDesc` (desc struct)

A film to start.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `dialogue` | `string` | `""` | — | — | — | The dialogue (StoryAsset.name), loaded with og.story.load and og.film.load. |
| `node` | `string` | `""` | — | — | — | The node to start at (empty: the dialogue's start). |
| `decks` | `number` | `0` | ≤ 3 | — | — | Decks to play on, 1 to 3 (0: 3). One plays; the others warm what the film may cut to. 1 warms nothing (one decoder: WeChat on iOS). |
| `paused` | `boolean` | `false` | — | — | — | Start holding the first segment's first frame, warmed, until og.film.play (a title screen before the first gesture). |

```ogt
struct FilmDesc {
  doc "A film to start."
  desc true
  field dialogue String "The dialogue (StoryAsset.name), loaded with og.story.load and og.film.load."
  field node String "The node to start at (empty: the dialogue's start)."
  field decks u32 "Decks to play on, 1 to 3 (0: 3). One plays; the others warm what the film may cut to. 1 warms nothing (one decoder: WeChat on iOS)." {
    max 3
  }
  field paused bool "Start holding the first segment's first frame, warmed, until og.film.play (a title screen before the first gesture)."
}
```

<sub>Module `film` line 7</sub>
