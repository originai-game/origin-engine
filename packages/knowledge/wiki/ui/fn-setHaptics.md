<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setHaptics"></a>

### `og.ui.setHaptics`

The player's haptic strength (a settings screen's vibration slider or switch): 0 turns the UI's haptics off, 1 (the default) plays the style's patterns as set; between scales them. The web shell rumbles the gamepad and vibrates a phone; the native host rumbles its pad.

```ts
og.ui.setHaptics(intensity: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `intensity` | `number` | no | 0 to 1 (clamped). |

**Returns** `void`.

Binding symbol `og_ui_set_haptics` · publish builds: kept

```ogt
fn setHaptics {
  doc "The player's haptic strength (a settings screen's vibration slider or switch): 0 turns the UI's haptics off, 1 (the default) plays the style's patterns as set; between scales them. The web shell rumbles the gamepad and vibrates a phone; the native host rumbles its pad."
  param intensity f32 "0 to 1 (clamped)."
  returns Void
}
```

<sub>Module `ui` line 1751</sub>
