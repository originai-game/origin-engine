<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-TitleScreen"></a>
### `TitleScreen`

The title screen over the key art, staged like a console attract screen: it fades up from black while the art pushes in slowly, the logo (or the name as a wide-tracked wordmark) rises into the middle, then Press any button (Tap to start on a touch screen) arrives at the foot and pulses, the confirm binding's glyph beside it. The legal line and the version sit in the lower corners. Any press reports onStart. It resolves in the dark scheme in both schemes.

`import { TitleScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/title.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `version` | `string` | The build's version. |
| `legal` | `string` | The legal line at the foot (copyright, trademarks). |
| `keyArt` | `Texture \| 0` | The key art's texture under the title (og.load.texture; 0: the style's dark ground). The screen sits over a picture, so it resolves in the dark scheme in both: its veils darken the art instead of washing it pale. |
| `logo` | `Texture \| 0` | The game's logo texture (og.load.texture; 0: the name set as a wordmark). |
| `name` | `string` | The game's name, set as a wordmark where there is no logo (empty: the logo slot's placeholder). |
| `background?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the background snippet). |
| `onStart?` | `((e: UiEvent, key?: string) => void) \| undefined` | The start event, with the row's key where it comes from a row. |

Pack slots: `Screen.frame`, `Screen.note`, `Screen.prompt`, `Title.wordmark`.

Fixtures: `default`.

Actions: `confirm` (Start).
