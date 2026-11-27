<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ControlsScreen"></a>
### `ControlsScreen`

The control rebinding screen: every action with its keyboard and gamepad binding, a category filter and a search. Pressing a cell asks the game to listen (`onRebind`); the game captures the next input, resolves any conflict as it chooses and writes the binding back. The screen never decides a binding.

`import { ControlsScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/controls.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own. |
| `bindings?` | `ReadonlyArray<ControlsBinding> \| undefined` | The bindings, in the order the game lists them. |
| `category?` | `string \| undefined` | The category shown; empty shows all. The game owns it (each `onCategory` writes it back). |
| `search?` | `string \| undefined` | The search text; the screen lists the rows whose label contains it. |
| `listening?` | `string \| undefined` | The binding waiting for an input, as `id:key` or `id:pad`: that cell reads Press a key. |
| `hidePad?` | `boolean \| undefined` | Hide the gamepad column (a keyboard-only game). |
| `onRebind?` | `((e: UiEvent, id: string, device: ControlsDevice) => void) \| undefined` | A cell was pressed: the game listens for the next input and writes the binding back. |
| `onClear?` | `((e: UiEvent, id: string, device: ControlsDevice) => void) \| undefined` | Clear a binding (the cell's clear button). |
| `onCategory?` | `((e: UiEvent, category: string) => void) \| undefined` |  |
| `onSearch?` | `((e: UiEvent, text: string) => void) \| undefined` |  |
| `onReset?` | `((e: UiEvent) => void) \| undefined` | Restore the game's default bindings. |
| `onBack?` | `((e: UiEvent) => void) \| undefined` |  |

Each `ControlsBinding`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The action's stable id (jump, interact). |
| `label` | `string` | The action as a player reads it (Jump). |
| `category?` | `string` | The group it is listed under (Movement, Combat); empty lists it under All only. |
| `key?` | `string` | The keyboard or mouse binding's words (Space, E, LMB); empty reads Unbound. |
| `pad?` | `string` | The gamepad binding's words (A, RB); empty reads Unbound. |
| `locked?` | `boolean` | The binding is fixed (pause, the menu key): shown, not offered for rebinding. |
| `conflict?` | `boolean` | Another action uses the same input: the row is marked, the game decides what that means. |

Pack slots: `Row.frame`, `Screen.frame`.

Fixtures: `default`, `keyboard`, `empty`.

Actions: `confirm` (Rebind), `back` (Back), `details` (Reset).
