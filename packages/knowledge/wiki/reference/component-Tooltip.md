<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Tooltip"></a>
### `Tooltip`

A hint over its trigger (NSH base's `Tooltip`): it shows while the pointer is over the trigger or the keyboard's focus is in it (WAI-ARIA's tooltip), dismisses on Escape, and its text describes the trigger for screen readers. The pack draws the bubble (`Tooltip.frame`); it stands centred over the trigger's box wherever it moves (`anchor`).

`import { Tooltip } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/tooltip.tsx`).

| Prop | Type | What |
|---|---|---|
| `text` | `string` | What it says. |
| `placement?` | `"top" \| "bottom" \| undefined` | Where it stands: over its trigger (`top`, the default) or under it (`bottom`). |
| `open?` | `boolean \| undefined` | Shown whatever the pointer does (a tutorial's hint); otherwise it shows while the trigger is hovered or keyboard-focused. |
| `class?` | `string \| undefined` | Classes of the document that uses it, on its bubble. |
| `children?` | `any` | The trigger. |

Pack slots: `Tooltip.frame`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`.
