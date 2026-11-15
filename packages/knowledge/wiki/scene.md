<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `scene` module

Scene changes with their assets fetched first: og.scene.preload asks for every streaming group a scene declares, og.scene.progress says how much of them has arrived, and og.scene.change replaces the current scene with it as the next tick starts. A scene is a GameDesc.scenes name (`level2`) or an asset id (`scenes/level2.ogt`).

What is resident is presentation: until a group arrives its models draw proxies and its sounds silence, so a change happens at the same tick on every host and in a replay. A loading screen (og.loading) covers it until og.scene.progress says ready.

See `loading §12`.

- **TypeScript**: functions are the namespace `og.scene`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `og.scene.preload`, `og.scene.progress`, `og.scene.change`; 5 more are advanced (`og api show scene --all`).
- **Index**: [every module and guide](index.md).

```ogt
module scene {
  doc "Scene changes with their assets fetched first: og.scene.preload asks for every streaming group a scene declares, og.scene.progress says how much of them has arrived, and og.scene.change replaces the current scene with it as the next tick starts. A scene is a GameDesc.scenes name (`level2`) or an asset id (`scenes/level2.ogt`)."
  doc "What is resident is presentation: until a group arrives its models draw proxies and its sounds silence, so a change happens at the same tick on every host and in a replay. A loading screen (og.loading) covers it until og.scene.progress says ready."
  # declarations follow, each on its own page
```

<sub>Module `scene` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`og.SceneState`](scene/record-SceneState.md) | resource |  | The scene og.scene.change put in place and the one it waits to put in place: world state, saved, restored and hashed. |
| [`og.SceneChanged`](scene/record-SceneChanged.md) | event |  | A scene change was put in place: sent as the tick starts, before any system runs, with the previous scene's entities despawned and the new one's spawned. |
| [`ScenePreload`](scene/type-ScenePreload.md) | struct |  | How far a scene's streaming groups have arrived (og.scene.progress). |
| [`og.scene.preload`](scene/fn-preload.md) | function | `(world: World, scene: string): Result<Asset>` | Load a scene and ask for every streaming group it declares (og.stream.request) until og.scene.release, so its packs are fetched while the current scene plays. |
| [`og.scene.progress`](scene/fn-progress.md) | function | `(world: World, scene: string): Result<ScenePreload>` | How far the scene's streaming groups have arrived: answered at once from what the host has received, for a loading screen's bar. |
| [`og.scene.release`](scene/fn-release.md) | function | `(world: World, scene: string): Result<void>` | Stop asking for the streaming groups og.scene.preload asked for (each unloads unless its trigger or og.stream.request still wants it). |
| [`og.scene.change`](scene/fn-change.md) | function | `(world: World, scene: string): Result<Asset>` | Replace the current scene with this one as the next tick starts: the current scene's root and every entity under it are despawned (the boot scene's own entities, with their descendants, for the first… |
| [`og.scene.current`](scene/fn-current.md) | function | `(world: World): string \| null` | The asset id of the scene in place: the last og.scene.change's, else GameDesc.scene's; null when the world started without a scene. |
