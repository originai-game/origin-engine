<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `game` module

The game entry: what the bundle's default export declares.

- **TypeScript**: functions are members of the root object `og` (`og.<fn>`); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module game {
  doc "The game entry: what the bundle's default export declares."
  place root
  # declarations follow, each on its own page
```

<sub>Module `game` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`FactsFn`](game/type-FactsFn.md) | callback |  | The game's own facts for the probe's state(): a small JSON object, computed from the world without changing it. |
| [`StartFn`](game/type-StartFn.md) | callback |  | The game's setup: called once before the first tick, after GameDesc.scene has spawned. |
| [`UiSetupFn`](game/type-UiSetupFn.md) | callback |  | Register named presentation stores. |
| [`GameDesc`](game/type-GameDesc.md) | desc struct |  | A game: its tick, scenes, systems and commands. |
| [`ExploreDesc`](game/type-ExploreDesc.md) | desc struct |  | `og explore`'s view of a game (GameDesc.explore): the state its archive of cells abstracts and the inputs it perturbs from them. |
| [`ExploreIntent`](game/type-ExploreIntent.md) | desc struct |  | An intent `og explore` sends (ExploreDesc.intents). |
| [`Game`](game/type-Game.md) | interface |  | A declared game, the bundle's default export. |
| [`og.game`](game/fn-game.md) | function | `(desc?: GameDesc): Game` | Declare the game. The bundle's default export is its result. |
