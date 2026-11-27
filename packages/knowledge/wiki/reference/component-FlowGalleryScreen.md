<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-FlowGalleryScreen"></a>
### `FlowGalleryScreen`

An endings gallery: the endings reached and locked, with their thumbnails, the selected one shown, and Back. It reports onSelect with the ending's key.

`import { FlowGalleryScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-gallery.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `endings` | `ReadonlyArray<{ id: string; title: string; thumb: Texture \| 0; locked: boolean }>` | The endings, in order (og.flow.load's endings): keyed rows of { id, title, thumb (texture: the ending segment's poster), locked (not reached in any playthrough: its title stays hidden) }. |
| `selected` | `string` | The ending selected now (its key). |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onSelect?` | `(e: UiEvent, key?: string) => void` | The select event, with the row's key where it comes from a row. |

Pack slots: `Screen.frame`.

Fixtures: `default`.

Actions: `select` (Watch the ending again), `back` (Back).
