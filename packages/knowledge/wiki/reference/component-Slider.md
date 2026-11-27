<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-Slider"></a>
### `Slider`

A value picked along a track (NSH base's `Slider`: a volume, a sensitivity): it owns its presses, drags, keys (the arrows by a step, Page Up and Down by a tenth, Home and End to the ends) and its range for screen readers; the pack draws its track (`Slider.track`), the filled range (`Slider.range`) and the thumb (`Slider.thumb`).

`import { Slider } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/slider.tsx`).

| Prop | Type | What |
|---|---|---|
| `value` | `number` | The value, from `min` to `max`. The game owns it: a press, a drag or a key reports the next one with `onChange`. |
| `min?` | `number \| undefined` | The range; default 0 to 100. |
| `max?` | `number \| undefined` |  |
| `step?` | `number \| undefined` | What the value snaps to, and what one arrow key moves it by; default 1. |
| `disabled?` | `boolean \| undefined` |  |
| `label?` | `string \| undefined` | What a screen reader calls it ("Music volume"). |
| `valueText?` | `string \| undefined` | The value as words for screen readers ("80%"). |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `onChange?` | `((e: UiEvent, value: number) => void) \| undefined` | The value it asks for, snapped to the step, as often as it changes during a drag. |
| `onChangeEnd?` | `((e: UiEvent, value: number) => void) \| undefined` | The change ended (the pointer lifted, a key's step done): the value it ended at, once. |

Pack slots: `Slider.range`, `Slider.thumb`, `Slider.track`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `empty`, `full`, `disabled`.
