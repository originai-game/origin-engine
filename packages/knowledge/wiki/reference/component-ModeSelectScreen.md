<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ModeSelectScreen"></a>
### `ModeSelectScreen`

A mode select: the modes as cards with their line, tag and picture, a locked mode's reason, the selected one's detail, and Play. It reports onSelect and onPlay; the game decides what is locked.

`import { ModeSelectScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/mode-select.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `modes` | `ReadonlyArray<{ id: string; line: string; locked: boolean; name: string; reason: string; tag: string; texture: Texture \| 0 }>` | The modes: { name, line, tag, locked, reason, texture }. |
| `selected` | `string` | The stable collection key selected now. |
| `keyArt` | `Texture \| 0` | The key art's texture under the screen's veil (og.load.texture; 0: the style's dark ground). |
| `detail?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the detail snippet). |
| `onBack?` | `((e: UiEvent, key?: string) => void) \| undefined` | The back event, with the row's key where it comes from a row. |
| `onPlay?` | `((e: UiEvent, key?: string) => void) \| undefined` | The play event, with the row's key where it comes from a row. |
| `onSelect?` | `(e: UiEvent, key?: string) => void` | The select event, with the row's key where it comes from a row. |

Pack slots: `Chip.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Play), `back` (Back).
