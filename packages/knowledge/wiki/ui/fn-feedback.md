<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-feedback"></a>

### `og.ui.feedback`

Play a UI event's feedback from the style (docs/ui.md §3.1): one sound of its set, at random and never the one it played last, at a volume and pitch from the set's ranges, and its haptic pattern at the player's strength (setHaptics). Widgets play their own events (press, focus, select, confirm, back, open, close, tab, toggleOn, toggleOff, sliderTick); game code plays the rest: scrollTick, typewriter, counterTick, notification, warning, error, success, reward and the reveals revealCommon to revealMythic. The choices come from the UI state, so a replay plays the same. An unknown event or one the style leaves silent plays nothing.

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
  doc "Play a UI event's feedback from the style (docs/ui.md §3.1): one sound of its set, at random and never the one it played last, at a volume and pitch from the set's ranges, and its haptic pattern at the player's strength (setHaptics). Widgets play their own events (press, focus, select, confirm, back, open, close, tab, toggleOn, toggleOff, sliderTick); game code plays the rest: scrollTick, typewriter, counterTick, notification, warning, error, success, reward and the reveals revealCommon to revealMythic. The choices come from the UI state, so a replay plays the same. An unknown event or one the style leaves silent plays nothing."
  param event String "The event's name."
  param rise f32 "A rising cascade's step, 0 to 1, walking the set's pitch range up (a counter's ticks); below 0 picks the pitch at random."
  returns Void
}
```

<sub>Module `ui` line 741</sub>
