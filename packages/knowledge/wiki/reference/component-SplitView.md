<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-SplitView"></a>
### `SplitView`

Two panes on the HUD plate, the divider between them a faint groove with its grip lit in the accent.

Dropped against the .svelte: nothing.

`import { SplitView } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/split-view.tsx`).

| Prop | Type | What |
|---|---|---|
| `ratio?` | `number` | The start pane's share of the room, 0 to 1; the game keeps it from the divider's drag events. Default 0.5. |
| `column?` | `boolean` | One pane over the other; false is side by side. |
| `label` | `string` | What a screen reader calls the divider; empty is Resize. |
| `end?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the end snippet). |
| `start?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the start snippet). |
| `onResize?` | `(e: UiEvent, key?: string) => void` | The resize event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`.

Fixtures: `default`.

Actions: `resize` (Resize).
