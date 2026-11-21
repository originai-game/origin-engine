<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="type-FilmView"></a>

### `FilmView` (struct)

What a film shows now.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `film` | `Entity` | `0` | — | — | — | The film. |
| `node` | `string` | `""` | — | — | — | The film node playing. |
| `segment` | `string` | `""` | — | — | — | Its segment's asset id. |
| `texture` | `Texture` | `0` | — | — | — | The current deck's picture (its og.VideoPlayer.texture): put it on a full-screen sprite or image. |
| `time` | `number` | `0` | — | `s` | — | Seconds into the segment. |
| `cut` | `number` | `0` | — | `s` | — | When it cuts to the next: its film.cut marker, else its duration. |
| `choices` | `readonly ChoiceView[]` | `[]` | — | — | — | The node's choices while its window is open or locked (og.story's). |
| `window` | `WindowView \| null` | `null` | — | — | — | The window, while it is open or locked. |
| `hotspots` | `readonly FilmHotspot[]` | `[]` | — | — | — | The hotspots that take a choice now: while the window is open, those of its enabled choices. |
| `qtes` | `readonly QteView[]` | `[]` | — | — | — | The quick-time events of its enabled choices while the window is open or locked. |
| `previous` | `Texture` | `0` | — | — | — | The picture dissolving out (`og-film-dissolve`); 0: none. Draw it under `texture`, which shows at `mix`. |
| `mix` | `number` | `0` | — | `ratio` | — | How far the dissolve is: `texture`'s opacity over `previous`, 1 without one. |
| `paused` | `boolean` | `false` | — | — | — | Held by og.film.pause. |
| `chapter` | `string` | `""` | — | — | — | The last chapter entered. |
| `ending` | `string` | `""` | — | — | — | The ending reached, once ended. |
| `ended` | `boolean` | `false` | — | — | — | It played to its end. |

```ogt
struct FilmView {
  doc "What a film shows now."
  field film Entity "The film."
  field node String "The film node playing."
  field segment String "Its segment's asset id."
  field texture Texture "The current deck's picture (its og.VideoPlayer.texture): put it on a full-screen sprite or image."
  field time f32 "Seconds into the segment." {
    unit s
  }
  field cut f32 "When it cuts to the next: its film.cut marker, else its duration." {
    unit s
  }
  field choices List.ChoiceView "The node's choices while its window is open or locked (og.story's)."
  field window Option.WindowView "The window, while it is open or locked."
  field hotspots List.FilmHotspot "The hotspots that take a choice now: while the window is open, those of its enabled choices."
  field qtes List.QteView "The quick-time events of its enabled choices while the window is open or locked."
  field previous Texture "The picture dissolving out (`og-film-dissolve`); 0: none. Draw it under `texture`, which shows at `mix`."
  field mix f32 "How far the dissolve is: `texture`'s opacity over `previous`, 1 without one." {
    unit ratio
  }
  field paused bool "Held by og.film.pause."
  field chapter String "The last chapter entered."
  field ending String "The ending reached, once ended."
  field ended bool "It played to its end."
}
```

<sub>Module `film` line 118</sub>
