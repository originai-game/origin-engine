<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `loading` module

The loading screen: who draws it (project.ogt `publish { loading engine | game | page }`), the stages the game reports while it loads, and when it is done. Every host answers the same calls; the web page also tells whatever page frames or hosts it (window events and, when framed, postMessage), so any site can draw its own loader.

See `loading §13`.

- **TypeScript**: functions are the namespace `og.loading`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `og.loading.stage`, `og.loading.done`, `og.loading.idle`; 4 more are advanced (`og api show loading --all`).
- **Index**: [every module and guide](index.md).

```ogt
module loading {
  doc "The loading screen: who draws it (project.ogt `publish { loading engine | game | page }`), the stages the game reports while it loads, and when it is done. Every host answers the same calls; the web page also tells whatever page frames or hosts it (window events and, when framed, postMessage), so any site can draw its own loader."
  # declarations follow, each on its own page
```

<sub>Module `loading` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`LoadingScreen`](loading/type-LoadingScreen.md) | enum |  | Who draws the loading screen (project.ogt `publish.loading`). |
| [`LoadingStageDesc`](loading/type-LoadingStageDesc.md) | desc struct |  | A stage of the game's own loading (og.loading.stage). |
| [`LoadingStatus`](loading/type-LoadingStatus.md) | struct |  | Where loading is. |
| [`og.loading.stage`](loading/fn-stage.md) | function | `(world: World, desc?: LoadingStageDesc): Result<void>` | Report a stage of the game's loading: the web page sends it to the page as `og:stage` (and to the framing page), and the engine's splash shows its label and fraction. |
| [`og.loading.done`](loading/fn-done.md) | function | `(world: World): Result<void>` | The game's loading is done: a `page` screen uncovers the canvas, the page hears `og:loaded`, and with `game` the play gate (if any) shows. |
| [`og.loading.idle`](loading/fn-idle.md) | function | `(world: World): boolean` | Whether the renderer has nothing left that would make a frame draw less than the scene: no pipelines compiling, no textures waiting to upload, no font faces fetched for the locale, no streamed model … |
| [`og.loading.status`](loading/fn-status.md) | function | `(world: World): LoadingStatus` | The screen, the last stage reported and whether loading is done. |
