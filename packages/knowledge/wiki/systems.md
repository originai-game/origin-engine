<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `systems` module

Systems the engine ships as TypeScript: game-level code, listed explicitly in GameDesc.systems like the game's own, so the schedule (and dump_world) shows them and nothing runs that the game did not ask for.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-feel`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module systems {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 8841</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.systems.cameras`](#value-cameras) | value | The camera controllers, in this order: og.orbitCamera (og.OrbitCamera), og.followCamera (og.FollowCamera) and og.follow2D (og.Follow2D). Each moves og.Camera only while an entity carries its component. List them before the game's systems that read og.Camera: `systems: [...og.systems.cameras, ...yours]`. |

## Values

Provided by the TypeScript runtime: no binding symbol.

<a id="value-cameras"></a>

### `og.systems.cameras`

The camera controllers, in this order: og.orbitCamera (og.OrbitCamera), og.followCamera (og.FollowCamera) and og.follow2D (og.Follow2D). Each moves og.Camera only while an entity carries its component. List them before the game's systems that read og.Camera: `systems: [...og.systems.cameras, ...yours]`.

```ts
const cameras: readonly System[]
```

```ogt
value cameras List.System "The camera controllers, in this order: og.orbitCamera (og.OrbitCamera), og.followCamera (og.FollowCamera) and og.follow2D (og.Follow2D). Each moves og.Camera only while an entity carries its component. List them before the game's systems that read og.Camera: `systems: [...og.systems.cameras, ...yours]`."
```

<sub>`api.ogt` line 8845</sub>
