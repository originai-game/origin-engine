<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-CalibrationScreen"></a>
### `CalibrationScreen`

Calibration in three steps: brightness (gamma), the audio offset measured from taps, and the controller's dead zone, each with its slider and test. The game owns the values and writes them back from the events.

`import { CalibrationScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/calibration.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `step` | `string` | brightness, offset or controller. |
| `brightness` | `number` | The gamma's adjustment, 0 to 1 (two-way). |
| `offset` | `number` | The audio offset in ms, -200 to 200 (two-way). |
| `taps` | `number` | Taps measured so far. |
| `deadZone` | `number` | The sticks' dead zone, 0 to 0.5 (two-way). |
| `onBrightness?` | `(e: UiEvent, value: number) => void` | The brightness slider stepped (onBrightness), the value clamped to 0 to 1. |
| `onOffset?` | `(e: UiEvent, value: number) => void` | The offset slider stepped (onOffset), the value clamped to -200 to 200 ms. |
| `onDeadZone?` | `(e: UiEvent, value: number) => void` | The dead-zone slider stepped (onDeadZone), the value clamped to 0 to 0.5. |
| `test?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the test snippet). |
| `onBack?` | `(e: UiEvent, key?: string) => void` | The back event, with the row's key where it comes from a row. |
| `onDone?` | `(e: UiEvent, key?: string) => void` | The done event, with the row's key where it comes from a row. |
| `onMeasure?` | `(e: UiEvent, key?: string) => void` | The measure event, with the row's key where it comes from a row. |
| `onNext?` | `(e: UiEvent, key?: string) => void` | The next event, with the row's key where it comes from a row. |
| `onTap?` | `(e: UiEvent, key?: string) => void` | The tap event, with the row's key where it comes from a row. |

Pack slots: `Panel.frame`, `Screen.frame`.

Fixtures: `default`.

Actions: `confirm` (Next), `back` (Back).
