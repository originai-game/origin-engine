<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-IconButton"></a>
### `IconButton`

A square press target with only an icon (close, settings, mute): always at least the 44 px touch target, always named for screen readers. The pack dresses it as a Button (`Button.plate.<variant>`), every state included; `IconButton.frame` adds what is its own (the square, the radius).

`import { IconButton } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/icon-button.tsx`).

| Prop | Type | What |
|---|---|---|
| `label` | `string` | What a screen reader calls it: required, since it has no words. |
| `icon` | `string` | The atlas icon it shows (close, settings, play). |
| `variant?` | `ButtonVariant \| undefined` | The look, as Button's: `ghost` (the default), `neutral`, `primary`, `danger`, `outline` … |
| `size?` | `"sm" \| "md" \| "lg" \| undefined` | `sm` (44 px target, 16 px icon), `md` (44 px, 20 px icon, the default) or `lg` (56 px, 24 px icon). |
| `pressed?` | `boolean \| undefined` | A toggle that is on (mute, pin). |
| `busy?` | `boolean \| undefined` | Its action is running: the pack's spinner replaces the icon. |
| `disabled?` | `boolean \| undefined` |  |
| `description?` | `string \| undefined` | What pressing it does, read after its name. |
| `class?` | `string \| undefined` | Classes after its own. |
| `onPress?` | `((e: UiEvent) => void) \| undefined` |  |

Pack slots: `Button.plate.…`, `Button.spinner`, `IconButton.frame`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `ghost`, `neutral`, `primary`, `pressed`, `disabled`.
