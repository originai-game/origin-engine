<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiTheme"></a>

### `UiTheme` (struct)

The style values the UI draws with, read-only.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheme` | `Scheme` | `"dark"` | — | — | — | The palette in use. |
| `palette` | `Readonly<Record<string, string>>` | `{}` | — | — | — | Each ThemeColor, then each colour role the style declares of its own (docs/ui.md §3.1), as `#rrggbb` or `#rrggbbaa` (sRGB as authored). |
| `space` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The spacing scale by name (xxs, xs, sm, md, lg, xl, xxl, xl3), in CSS pixels. |
| `radius` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The corner radii by name (small, control, bar, panel), in CSS pixels. |
| `typeRamp` | `Readonly<Record<string, TypeStep>>` | `{}` | — | — | — | Each TypeStepName's step. |
| `sizes` | `ControlSizes` | `{ controlSm: 0, controlMd: 0, controlLg: 0, gutter: 0, header: 0, row: 0, section: 0 }` | — | — | — | The control sizes of the density in effect. |
| `durationsMs` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The motion durations by name (instant, fast, base, slow, spring, bounce), in milliseconds. |
| `style` | `string` | `""` | — | — | — | The configured style's name; empty for the default style. |
| `contrast` | `Contrast` | `"standard"` | — | — | — | The variant in effect: high when the game asked for it or the page prefers more contrast. |
| `reducedMotion` | `boolean` | `false` | — | — | — | Transitions only fade and sheens stop: the game asked for it or the page prefers reduced motion. A game's own animations read it to keep their state changes and drop their movement. |
| `textScale` | `number` | `0` | — | — | — | The player's text scale in effect (UiDesc.textScale, 1 to 1.5): typeRamp and sizes already carry it; tokens' typography is the style's as authored, so a game's own text drawn from it multiplies by this. |
| `colorVision` | `ColorVision` | `"none"` | — | — | — | The player's colour vision in effect (UiDesc.colorVision). |
| `colorVisionFilter` | `ColorVisionFilter` | `"correct"` | — | — | — | Whether the frame is corrected or simulated for it. |
| `colorVisionStrength` | `number` | `0` | — | — | — | Its severity, 0 to 1. |
| `glassBackdrop` | `GlassBackdrop` | `"none"` | — | — | — | The backdrop blur policy in effect, after the configured override and style; none when neither opts in. |

```ogt
struct UiTheme {
  doc "The style values the UI draws with, read-only."
  field scheme Scheme "The palette in use."
  field palette Map.String "Each ThemeColor, then each colour role the style declares of its own (docs/ui.md §3.1), as `#rrggbb` or `#rrggbbaa` (sRGB as authored)."
  field space Map.f32 "The spacing scale by name (xxs, xs, sm, md, lg, xl, xxl, xl3), in CSS pixels."
  field radius Map.f32 "The corner radii by name (small, control, bar, panel), in CSS pixels."
  field typeRamp Map.TypeStep "Each TypeStepName's step."
  field sizes ControlSizes "The control sizes of the density in effect."
  field durationsMs Map.f32 "The motion durations by name (instant, fast, base, slow, spring, bounce), in milliseconds."
  field style String "The configured style's name; empty for the default style."
  field contrast Contrast "The variant in effect: high when the game asked for it or the page prefers more contrast."
  field reducedMotion bool "Transitions only fade and sheens stop: the game asked for it or the page prefers reduced motion. A game's own animations read it to keep their state changes and drop their movement."
  field textScale f32 "The player's text scale in effect (UiDesc.textScale, 1 to 1.5): typeRamp and sizes already carry it; tokens' typography is the style's as authored, so a game's own text drawn from it multiplies by this."
  field colorVision ColorVision "The player's colour vision in effect (UiDesc.colorVision)."
  field colorVisionFilter ColorVisionFilter "Whether the frame is corrected or simulated for it."
  field colorVisionStrength f32 "Its severity, 0 to 1."
  field glassBackdrop GlassBackdrop "The backdrop blur policy in effect, after the configured override and style; none when neither opts in."
}
```

<sub>Module `ui` line 285</sub>
