<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RadialMenu"></a>
### `RadialMenu`

A radial menu: slices clockwise from the top, the one the pointer or stick aims at lit, and its detail in the centre. It reports onAim and onChoose; the game decides what a slice does.

`import { RadialMenu } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/radial-menu.tsx`).

| Prop | Type | What |
|---|---|---|
| `items?` | `ReadonlyArray<RadialMenuItem>` | The slices, clockwise from the top. |
| `span?` | `number` | One slice's share of the turn (1 / n). |
| `chosen?` | `string` | The slice the pointer or stick points at (its id); empty is none. |
| `detail?` | `string` | The centre's line for the chosen slice; empty shows its label. |
| `onAim?` | `(e: UiEvent) => void` | Aim: the pointer moved. |
| `onHover?` | `(e: UiEvent, id: string) => void` | Optional slice hover callback when pointer enters a slice's button. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose a slice. |

Each `RadialMenuItem`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `label` | `string` | Its label (Heal). |
| `icon?` | `string` | An icon of the atlas. |
| `at` | `number` | Its centre as a turn fraction (slice i of n: i / n). |
| `cost?` | `string` | Its cost as the game words it (300); empty is none. |
| `unaffordable?` | `boolean` | It cannot be paid for. |
| `disabled?` | `boolean` |  |

Fixtures: `default`.
