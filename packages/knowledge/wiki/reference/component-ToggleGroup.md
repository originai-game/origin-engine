<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ToggleGroup"></a>
### `ToggleGroup`

A row of `Toggle`s (NSH base's `ToggleGroup`); one pressed at a time is rune-kit's segmented control, which the game keeps by pressing the one asked for and letting go of the others. One stop of Tab, at its first pressed toggle; the arrows move among its toggles without pressing them (WAI-ARIA's toolbar). The pack draws its well (`ToggleGroup.frame`).

`import { ToggleGroup } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/toggle.tsx`).

| Prop | Type | What |
|---|---|---|
| `label?` | `string \| undefined` | What screen readers call the group ("View"). |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `children?` | `any` |  |

Pack slots: `Toggle.item`, `ToggleGroup.frame`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `segmented`.
