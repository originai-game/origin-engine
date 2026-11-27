<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-FlowChaptersScreen"></a>
### `FlowChaptersScreen`

A chapter select: the story's chapters in order with their thumbnails, the selected one shown, and Back. It reports onSelect with the chapter's key.

`import { FlowChaptersScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-chapters.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `chapters` | `ReadonlyArray<{ id: string; title: string; thumb: Texture \| 0; locked: boolean }>` | The chapters, in order (og.flow.load's chapters): keyed rows of { id, title, thumb (texture: the chapter's save's poster), locked (not reached yet: no save to play from) }. |
| `selected` | `string` | The chapter selected now (its key). |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onSelect?` | `(e: UiEvent, key?: string) => void` | The select event, with the row's key where it comes from a row. |

Pack slots: `Screen.frame`.

Fixtures: `default`.

Actions: `select` (Play the chapter), `back` (Back).
