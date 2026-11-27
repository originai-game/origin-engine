<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-CreatorToolsScreen"></a>
### `CreatorToolsScreen`

The creator's tools on the HUD's smoked glass: the four tools a segmented switch lit in the accent, the panels plates with tracked capital titles. On a phone the panel tabs are one strip that scrolls sideways, each a finger's target; a focus move scrolls its tab into view.

`import { CreatorToolsScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/creator-tools.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `tool` | `string` | select, move, rotate or scale. |
| `playing?` | `boolean` | The world is playing. |
| `selected` | `string` | The selected object's name (empty: none). |
| `panel` | `string` | On a phone, the open panel: hierarchy, inspector or catalogue. |
| `dirty?` | `boolean` | Unsaved changes. |
| `cataloguePanelCatalogue?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the cataloguePanelCatalogue snippet). |
| `scrollInspector?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the scrollInspector snippet). |
| `treePanelHierarchy?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the treePanelHierarchy snippet). |
| `view?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the view snippet). |
| `onPlay?` | `(e: UiEvent, key?: string) => void` | The play event, with the row's key where it comes from a row. |
| `onRedo?` | `(e: UiEvent, key?: string) => void` | The redo event, with the row's key where it comes from a row. |
| `onSave?` | `(e: UiEvent, key?: string) => void` | The save event, with the row's key where it comes from a row. |
| `onUndo?` | `(e: UiEvent, key?: string) => void` | The undo event, with the row's key where it comes from a row. |
| `onTool?` | `(e: UiEvent, key?: string) => void` | A tool pressed, with its key (select, move, rotate or scale): the game sets `tool`. |
| `onPanel?` | `(e: UiEvent, key?: string) => void` | A panel tab pressed on a phone, with its key (hierarchy, inspector or catalogue): the game sets `panel`. |

Pack slots: `Panel.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `back` (Exit).
