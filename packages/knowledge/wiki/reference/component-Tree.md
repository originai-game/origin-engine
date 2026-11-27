<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Tree"></a>
### `Tree`

A HUD plate of rows, each indented by its depth with a faint guide down from its parent; the chosen row is washed in the accent from its leading edge. A leaf has no twisty and keeps its room, so its name lines up with its siblings' that open. A name longer than its row wraps inside it, at any word.

`import { Tree } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/tree.tsx`).

| Prop | Type | What |
|---|---|---|
| `rows?` | `ReadonlyArray<{ id: string; label: string; depth: number; open: boolean \| null; selected: boolean }>` | The visible rows in order, a keyed collection of { id, label, depth (0 at the top), open (true, false, or null for a leaf), selected }; the game flattens the tree as it opens and closes. |
| `label` | `string` | The tree's accessible name. |
| `onChoose?` | `(e: UiEvent, key?: string) => void` | The choose event, with the row's key where it comes from a row. |
| `onToggle?` | `(e: UiEvent, key?: string) => void` | The toggle event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`.

Fixtures: `default`, `overflow`.

Actions: `toggle` (Open or close), `choose` (Select).
