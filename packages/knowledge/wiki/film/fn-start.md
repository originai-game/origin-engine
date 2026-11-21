<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`film`](../film.md) · [index](../index.md)

<a id="fn-start"></a>

### `og.film.start`

Start a film: its decks, and its conversation held at the first film node, whose segment plays from the next tick. Outside a system (setup or a command).

```ts
og.film.start(world: World, desc?: FilmDesc): Result<Entity>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `FilmDesc` | yes | The dialogue, and optionally the node, the decks and whether it starts paused. |

**Returns** `Result<Entity>`: The film's entity (og.Film), or why not: a dialogue og.film.load has not prepared, an unknown node, or inside a system.

Binding symbol `og_film_start` · publish builds: kept

```ogt
fn start {
  doc "Start a film: its decks, and its conversation held at the first film node, whose segment plays from the next tick. Outside a system (setup or a command)."
  param world World "The world."
  param desc FilmDesc "The dialogue, and optionally the node, the decks and whether it starts paused."
  returns Result.Entity "The film's entity (og.Film), or why not: a dialogue og.film.load has not prepared, an unknown node, or inside a system."
}
```

<sub>Module `film` line 207</sub>
