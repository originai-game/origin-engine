<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="type-Game"></a>

### `Game` (interface)

A declared game, the bundle's default export.

```ogt
interface Game {
  doc "A declared game, the bundle's default export."
  # members follow, each quoted below
```

<sub>Module `game` line 53</sub>

<a id="member-Game-tick"></a>

#### `Game.tick` (property)

GameDesc.tick, or 60 when it is unset (the engine then runs at project.ogt's `tick` when the host passes the project, as og run and og build do).

```ts
readonly tick: number
```

Binding symbol `og_game_get_tick` · publish builds: kept

```ogt
prop tick u32 "GameDesc.tick, or 60 when it is unset (the engine then runs at project.ogt's `tick` when the host passes the project, as og run and og build do)."
```

<sub>Module `game` line 55</sub>

<a id="member-Game-systems"></a>

#### `Game.systems` (property)

The systems in declaration order.

```ts
readonly systems: readonly System[]
```

Binding symbol `og_game_get_systems` · publish builds: kept

```ogt
prop systems List.System "The systems in declaration order."
```

<sub>Module `game` line 56</sub>
