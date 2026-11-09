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
| `styleVersion` | `number` | `0` | — | — | — | The configured style file's version: 1 for a version-1 style, whose game keeps the controls and dialogs it shipped with (docs/ui.md §3); 2 for a version-2 style and the default style. |
| `contrast` | `Contrast` | `"standard"` | — | — | — | The variant in effect: high when the game asked for it or the page prefers more contrast. |
| `reducedMotion` | `boolean` | `false` | — | — | — | Transitions only fade and sheens stop: the game asked for it or the page prefers reduced motion. A game's own animations read it to keep their state changes and drop their movement. |
| `textScale` | `number` | `0` | — | — | — | The player's text scale in effect (UiDesc.textScale, 1 to 1.5): typeRamp and sizes already carry it; tokens' typography is the style's as authored, so a game's own text drawn from it multiplies by this. |
| `unit` | `number` | `0` | — | — | — | The og-ui unit in effect, in CSS px (UiDesc.unitScale; auto: 1 to 2 by the surface): what every length the UI reads and gives is in, og.ui.bounds and og.ui.viewport too. Only what leaves the UI for the page is CSS px: the probe's dump_ui and og.render's screen sprites; a game drawing its own screen sprites beside the UI multiplies by it. |
| `colorVision` | `ColorVision` | `"none"` | — | — | — | The player's colour vision in effect (UiDesc.colorVision). |
| `colorVisionFilter` | `ColorVisionFilter` | `"correct"` | — | — | — | Whether the frame is corrected or simulated for it. |
| `colorVisionStrength` | `number` | `0` | — | — | — | Its severity, 0 to 1. |
| `reducedTransparency` | `boolean` | `false` | — | — | — | Nothing is see-through: the scrim and the translucent fills of `palette` are already the opaque colours they make over panel (the scrim over backdrop) and glass is an opaque surface; the game asked for it or the page prefers reduced transparency. A game's own translucent overlays read it to draw opaque too. |
| `tokens` | `unknown` | `null` | — | — | — | The style's tokens (docs/ui.md §3.1), what a game's own drawing reads to match its UI: `typography` (each role, display title heading body label caption numeric button, at each breakpoint: size, lineHeight, trackingEm, weight, font, features), the `space`, `size`, `radius` and `stroke` scales by step, `elevation` (each level's shadows: dx, dy, blur, spread, color), `shape` (each element class's corner and radius: the shape language), `springs` (ui, enter, exit, emphasis, playful: durationMs, bounce), `staggerMs`, `overshoot`, the `rarity`, `team` and `series` colours, `cues` (each colour token's cue besides its hue, by name: `{ icon: <index in the style's icons> }`, `{ pattern: <name> }` or `{ pips: <n> }`; docs/ui.md §3.2), and `feedback` (each UI event's set: how many sounds, the volume and pitch ranges, the haptic's [startMs, ms, low, high] pulses). |

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
  field styleVersion u32 "The configured style file's version: 1 for a version-1 style, whose game keeps the controls and dialogs it shipped with (docs/ui.md §3); 2 for a version-2 style and the default style."
  field contrast Contrast "The variant in effect: high when the game asked for it or the page prefers more contrast."
  field reducedMotion bool "Transitions only fade and sheens stop: the game asked for it or the page prefers reduced motion. A game's own animations read it to keep their state changes and drop their movement."
  field textScale f32 "The player's text scale in effect (UiDesc.textScale, 1 to 1.5): typeRamp and sizes already carry it; tokens' typography is the style's as authored, so a game's own text drawn from it multiplies by this."
  field unit f32 "The og-ui unit in effect, in CSS px (UiDesc.unitScale; auto: 1 to 2 by the surface): what every length the UI reads and gives is in, og.ui.bounds and og.ui.viewport too. Only what leaves the UI for the page is CSS px: the probe's dump_ui and og.render's screen sprites; a game drawing its own screen sprites beside the UI multiplies by it."
  field colorVision ColorVision "The player's colour vision in effect (UiDesc.colorVision)."
  field colorVisionFilter ColorVisionFilter "Whether the frame is corrected or simulated for it."
  field colorVisionStrength f32 "Its severity, 0 to 1."
  field reducedTransparency bool "Nothing is see-through: the scrim and the translucent fills of `palette` are already the opaque colours they make over panel (the scrim over backdrop) and glass is an opaque surface; the game asked for it or the page prefers reduced transparency. A game's own translucent overlays read it to draw opaque too."
  field tokens Json "The style's tokens (docs/ui.md §3.1), what a game's own drawing reads to match its UI: `typography` (each role, display title heading body label caption numeric button, at each breakpoint: size, lineHeight, trackingEm, weight, font, features), the `space`, `size`, `radius` and `stroke` scales by step, `elevation` (each level's shadows: dx, dy, blur, spread, color), `shape` (each element class's corner and radius: the shape language), `springs` (ui, enter, exit, emphasis, playful: durationMs, bounce), `staggerMs`, `overshoot`, the `rarity`, `team` and `series` colours, `cues` (each colour token's cue besides its hue, by name: `{ icon: <index in the style's icons> }`, `{ pattern: <name> }` or `{ pips: <n> }`; docs/ui.md §3.2), and `feedback` (each UI event's set: how many sounds, the volume and pitch ranges, the haptic's [startMs, ms, low, high] pulses)."
}
```

<sub>Module `ui` line 394</sub>
