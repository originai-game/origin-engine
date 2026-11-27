<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`loading`](../loading.md) · [index](../index.md)

<a id="type-LoadingScreen"></a>

### `LoadingScreen` (enum)

Who draws the loading screen (project.ts `publish.loading`).

A string: one of the variants below. The zero value is `"engine"`.

| Variant | Doc |
|---|---|
| `"engine"` | The engine's splash: the brand mark with the boot's progress and the stage labels, until the first frame. |
| `"game"` | The game: the splash covers only the engine's own boot, then the game's first frames are its loading screen, the clock running, until og.loading.done. |
| `"page"` | The page hosting the game: no splash, the canvas hidden with the clock running until og.loading.done, and the page draws its loader from the window events. |

```ogt
enum LoadingScreen {
  doc "Who draws the loading screen (project.ts `publish.loading`)."
  variant engine "The engine's splash: the brand mark with the boot's progress and the stage labels, until the first frame."
  variant game "The game: the splash covers only the engine's own boot, then the game's first frames are its loading screen, the clock running, until og.loading.done."
  variant page "The page hosting the game: no splash, the canvas hidden with the clock running until og.loading.done, and the page draws its loader from the window events."
}
```

<sub>Module `loading` line 6</sub>
