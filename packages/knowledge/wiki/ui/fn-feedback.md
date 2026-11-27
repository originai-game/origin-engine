<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-feedback"></a>

### `og.ui.feedback`

Play a UI event's feedback (docs/ui.md §3.1): the style pack's pack-wide sound for it (pack.json `sounds`, or og.ui.configure's; confirm and back fall back to press), one of its clips at random with volume and pitch from its ranges, and the event's haptic pattern at the player's strength (setHaptics). Events with a haptic: press, select, cancel, back, tab, toggleOn, toggleOff, sliderTick, confirm, notification, warning, error, success, reward and revealUncommon to revealMythic. Choices use UI state and replay deterministically. An event the pack gives no sound and no haptic plays nothing.

```ts
og.ui.feedback(event: string, rise: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `event` | `string` | no | The event's name. |
| `rise` | `number` | no | A rising cascade's step, 0 to 1, walking the set's pitch range up (a counter's ticks); below 0 picks the pitch at random. |

**Returns** `void`.

Binding symbol `og_ui_feedback` · publish builds: kept

```ogt
fn feedback {
  doc "Play a UI event's feedback (docs/ui.md §3.1): the style pack's pack-wide sound for it (pack.json `sounds`, or og.ui.configure's; confirm and back fall back to press), one of its clips at random with volume and pitch from its ranges, and the event's haptic pattern at the player's strength (setHaptics). Events with a haptic: press, select, cancel, back, tab, toggleOn, toggleOff, sliderTick, confirm, notification, warning, error, success, reward and revealUncommon to revealMythic. Choices use UI state and replay deterministically. An event the pack gives no sound and no haptic plays nothing."
  param event String "The event's name."
  param rise f32 "A rising cascade's step, 0 to 1, walking the set's pitch range up (a counter's ticks); below 0 picks the pitch at random."
  returns Void
}
```

<sub>Module `ui` line 359</sub>
