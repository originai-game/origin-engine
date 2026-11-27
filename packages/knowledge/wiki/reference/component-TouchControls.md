<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-TouchControls"></a>
### `TouchControls`

On-screen controls for touch: a stick (or a d-pad) at the start side feeding a move action, an optional look stick and the action buttons at the end side. They feed og.input's actions and show only where touch is the last input, unless always is set.

`import { TouchControls } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/touch-controls.tsx`).

| Prop | Type | What |
|---|---|---|
| `move?` | `string` | The two-axis og-input action the stick at the start side feeds (move); empty leaves the stick out. |
| `pad?` | `boolean` | A d-pad in the stick's place: eight ways, each axis -1, 0 or 1. |
| `look?` | `string` | A second stick at the end side and the two-axis action it feeds (look); empty leaves it out. |
| `floatingStick?` | `boolean` | Centre the move stick where the finger lands inside its box. Ignored for a d-pad. |
| `lookPad?` | `boolean` | Use a drag pad for look instead of a stick: CSS px per fixed tick, x right and y up; holding still reads zero. |
| `deadZone?` | `number` | The sticks' dead zone as a part of their radius, up to 0.95; 0 is the engine's 0.15. |
| `actions?` | `ReadonlyArray<TouchAction>` | The buttons at the end side, the first nearest the corner: keyed rows of { id, action, label, icon, primary, charge, disabled }, each as ActionButton takes them. |
| `always?` | `boolean` | Shown under a mouse too (a touch laptop, a preview). Otherwise the controls show only where the primary pointer is a finger. |
| `disabled?` | `boolean` | Every control feeds nothing and shows faded (a cutscene, a menu over the HUD). |
| `leftHanded?` | `boolean` | Mirror the layout: the stick at the end side, the buttons and the look stick at the start side (a left-handed player's setting). |

Each `TouchAction`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `action` | `string` |  |
| `label` | `string` |  |
| `icon?` | `string` |  |
| `primary?` | `boolean` |  |
| `charge?` | `number` | A charge round its rim, 0 to 1; negative is none. Default -1. |
| `disabled?` | `boolean` |  |

Pack slots: `Touch.actions`, `Touch.bar`, `Touch.base`, `Touch.knob`, `Touch.pad`.

Fixtures: `floating`, `lookPad`, `leftHanded`, `default`, `pad`, `twin`, `many`, `disabled`.
