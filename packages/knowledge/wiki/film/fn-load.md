<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="fn-load"></a>

### `og.film.load`

Prepare a loaded dialogue's film: every segment's timeline (and through it every video, which fetches nothing until it plays) and og.film's records. Call it in setup, after og.story.load, in every process that restores a film.

```ts
og.film.load(world: World, dialogue: string): Result<FilmInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `dialogue` | `string` | no | The dialogue (StoryAsset.name). |

**Returns** `Result<FilmInfo>`: Refused for a dialogue not loaded, or a segment that is not a cooked timeline with a video track.

Binding symbol `og_film_load` · publish builds: kept

```ogt
fn load {
  doc "Prepare a loaded dialogue's film: every segment's timeline (and through it every video, which fetches nothing until it plays) and og.film's records. Call it in setup, after og.story.load, in every process that restores a film."
  param world World "The world."
  param dialogue String "The dialogue (StoryAsset.name)."
  returns Result.FilmInfo "Refused for a dialogue not loaded, or a segment that is not a cooked timeline with a video track."
}
```

<sub>Module `film` line 200</sub>
