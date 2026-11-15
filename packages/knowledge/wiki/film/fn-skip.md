<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="fn-skip"></a>

### `og.film.skip`

Skip ahead in the segment playing: to its window's open (film.open) when that is ahead, else to its cut, which takes the pending choice, else the default. For a segment the player has seen; what was seen is the game's (guide film §5). Systems may call it.

```ts
og.film.skip(world: World, film: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `film` | `Entity` | no | The film. |

**Returns** `Result<void>`: Refused for an entity that is no film, or one that has ended.

Binding symbol `og_film_skip` · publish builds: kept

```ogt
fn skip {
  doc "Skip ahead in the segment playing: to its window's open (film.open) when that is ahead, else to its cut, which takes the pending choice, else the default. For a segment the player has seen; what was seen is the game's (guide film §5). Systems may call it."
  param world World "The world."
  param film Entity "The film."
  returns Result.Void "Refused for an entity that is no film, or one that has ended."
}
```

<sub>Module `film` line 242</sub>
