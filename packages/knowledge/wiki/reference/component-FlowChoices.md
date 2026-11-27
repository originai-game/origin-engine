<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-FlowChoices"></a>
### `FlowChoices`

An interactive film's choice window, hotspots, and rich QTE prompts over its footage (flow guide §4): - Choice & non-cutting inspect hotspots align to `videoBox` (landscape or 9:16 portrait), supporting both `rect` and `circle` shapes. - The countdown bar hides automatically when `window.held` (held or looping segments). - Choice plates surface `reason`, `visited`, `check`, `defaultChoice`, and all QTE types (`press`, `hold`, `mash`, `timing` zone bar, `sequence`, `swipe`, `drag`).

`import { FlowChoices } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/flow-choices.tsx`).

| Prop | Type | What |
|---|---|---|
| `view` | `FlowView \| null` | What the flow shows now (og.flow.view's value, set each tick); null or a view without a window/hotspots hides the screen. |
| `videoBox?` | `VideoBox \| undefined` | Rendered video rect in normalized surface fractions so hotspots align on any aspect ratio (16:9, 21:9, 9:16). |
| `onChoose?` | `((e: UiEvent, index: number) => void) \| undefined` | A choice was taken, by its row or its choice hotspot: pass `index` (ChoiceView.index) to og.flow.choose. |
| `onInspect?` | `((e: UiEvent, hotspot: string) => void) \| undefined` | A non-cutting `inspect` hotspot was pressed: pass `hotspot` id to og.flow.inspect. |

Each `VideoBox`:

| Field | Type | What |
|---|---|---|
| `left` | `number` | Left edge of the rendered video in surface width fractions. |
| `top` | `number` | Top edge of the rendered video in surface height fractions. |
| `width` | `number` | Width of the rendered video in surface width fractions. |
| `height` | `number` | Height of the rendered video in surface height fractions. |

Pack slots: `Screen.frame`.

Fixtures: `default`, `locked`, `closed`.
