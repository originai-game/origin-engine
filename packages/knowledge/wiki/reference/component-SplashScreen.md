<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-SplashScreen"></a>
### `SplashScreen`

The boot splash: the studio marks one after another (a texture, or the name set as type), boot progress and what is loading, and Skip. The game moves through the marks.

`import { SplashScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/splash.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `marks` | `ReadonlyArray<{ id: string; name: string; texture: Texture \| 0 }>` | The marks in order: { name, texture } (texture: an og.load.texture handle; 0 sets the name as a wordmark). |
| `mark` | `string` | The stable collection key selected now. |
| `progress` | `number` | Boot progress, 0 to 1 (below 0: no bar). |
| `status` | `string` | What is loading, small beside the bar's figure (empty: nothing). |
| `custom?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the custom snippet). |
| `onSkip?` | `(e: UiEvent, key?: string) => void` | The skip event, with the row's key where it comes from a row. |

Pack slots: `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Skip).
