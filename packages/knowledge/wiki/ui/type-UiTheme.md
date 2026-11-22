<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiTheme"></a>

### `UiTheme` (struct)

What the UI draws with, read-only: the style pack's theme (the default pack with the game's over it) in the scheme and contrast in effect, in the design tokens' own names, and the player's settings in effect.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheme` | `Scheme` | `"dark"` | — | — | — | The palette in use. |
| `colors` | `Readonly<Record<string, string>>` | `{}` | — | — | — | Each colour of the theme by its token name (`--color-fg` is fg), as `#rrggbb` or `#rrggbbaa`. |
| `radius` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The corner radii by name (button, card, panel, …), in CSS pixels. |
| `fontSize` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The font sizes by name (body, caption, display-lg, …), in CSS pixels. |
| `lineHeight` | `Readonly<Record<string, number>>` | `{}` | — | — | — | Each font size's line height, a ratio of it. |
| `tracking` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The letter spacing by name, in em. |
| `fontWeight` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The font weights by name (regular, medium, semibold, bold, body). |
| `blur` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The backdrop blurs by name (thin, regular, thick, lens), in CSS pixels. |
| `durations` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The motion durations by name (instant, fast, base, slow, spring, bounce, …: `--motion-duration-*`), in milliseconds. |
| `style` | `string` | `""` | — | — | — | The configured style's name; empty for the default style. |
| `contrast` | `Contrast` | `"standard"` | — | — | — | The variant in effect: high when the game asked for it or the page prefers more contrast. |
| `reducedMotion` | `boolean` | `false` | — | — | — | Transitions only fade: the game asked for it or the page prefers reduced motion. A game's own animations read it to keep their state changes and drop their movement. |
| `textScale` | `number` | `0` | — | — | — | The player's text scale in effect (UiDesc.textScale, 1 to 1.5). |
| `unit` | `number` | `0` | — | — | — | The UI unit in effect, in CSS px (UiDesc.unitScale; auto: 1 to 2 by the surface): what the tree lays out in. A game drawing its own screen sprites beside the UI multiplies by it. |
| `colorVision` | `ColorVision` | `"none"` | — | — | — | The player's colour vision in effect (UiDesc.colorVision). |
| `colorVisionFilter` | `ColorVisionFilter` | `"correct"` | — | — | — | Whether the frame is corrected or simulated for it. |
| `colorVisionStrength` | `number` | `0` | — | — | — | Its severity, 0 to 1. |

```ogt
struct UiTheme {
  doc "What the UI draws with, read-only: the style pack's theme (the default pack with the game's over it) in the scheme and contrast in effect, in the design tokens' own names, and the player's settings in effect."
  field scheme Scheme "The palette in use."
  field colors Map.String "Each colour of the theme by its token name (`--color-fg` is fg), as `#rrggbb` or `#rrggbbaa`."
  field radius Map.f32 "The corner radii by name (button, card, panel, …), in CSS pixels."
  field fontSize Map.f32 "The font sizes by name (body, caption, display-lg, …), in CSS pixels."
  field lineHeight Map.f32 "Each font size's line height, a ratio of it."
  field tracking Map.f32 "The letter spacing by name, in em."
  field fontWeight Map.f32 "The font weights by name (regular, medium, semibold, bold, body)."
  field blur Map.f32 "The backdrop blurs by name (thin, regular, thick, lens), in CSS pixels."
  field durations Map.f32 "The motion durations by name (instant, fast, base, slow, spring, bounce, …: `--motion-duration-*`), in milliseconds."
  field style String "The configured style's name; empty for the default style."
  field contrast Contrast "The variant in effect: high when the game asked for it or the page prefers more contrast."
  field reducedMotion bool "Transitions only fade: the game asked for it or the page prefers reduced motion. A game's own animations read it to keep their state changes and drop their movement."
  field textScale f32 "The player's text scale in effect (UiDesc.textScale, 1 to 1.5)."
  field unit f32 "The UI unit in effect, in CSS px (UiDesc.unitScale; auto: 1 to 2 by the surface): what the tree lays out in. A game drawing its own screen sprites beside the UI multiplies by it."
  field colorVision ColorVision "The player's colour vision in effect (UiDesc.colorVision)."
  field colorVisionFilter ColorVisionFilter "Whether the frame is corrected or simulated for it."
  field colorVisionStrength f32 "Its severity, 0 to 1."
}
```

<sub>Module `ui` line 145</sub>
