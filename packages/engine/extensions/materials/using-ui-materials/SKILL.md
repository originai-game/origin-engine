---
name: using-ui-materials
description: "Per-element shader effects for a game's UI: foil, holo, burn and rarity glow as ui-material presets with tunable parameters. Use when a card, slot, reward or button needs a material finish (a foil edition, a legendary glow, a card burning away) or when writing a new ui-material."
---

# UI material presets

Install the presets into the game with `og ui add materials`; they land in
`assets/ui-kits/materials/` as ordinary `type ui-material` assets (an
`.ogt` with `param` lines beside its WGSL), the game's own to edit.

| Preset | What it does | Parameters (in order) |
| --- | --- | --- |
| `foil.ogt` | thin crossing bands of metallic sheen that slide with the clock, bright where the art is bright | `amount` 0–1, `scale` CSS px between bands, `speed` cycles/s, `tint` linear RGB |
| `holo.ogt` | a rainbow in diagonal bands, warped by noise, over the art's lightness | `amount` 0–1, `scale` CSS px a cycle, `speed` cycles/s, `saturation` 0–1 |
| `burn.ogt` | the element burns away from noise: a charred rim behind a hot edge | `progress` 0–1, `edge` 0.02–0.2, `scale` CSS px a noise cell, `glow` linear RGB (above 1 blooms) |
| `rarity.ogt` | a tier-coloured glow in from the element's edge, a bright rim, a breath | `color` linear RGB, `size` CSS px, `pulse` breaths/s, `strength` 0–2 |

The built-in `dissolve`, `shine` and `hologram` materials (docs/ui.md
§5.4) need no install: `material shine -0.4 0.2 0.8 0.6` in a dressing.

## On a part, in the style

A dressing names the preset by its asset path (relative to the style file
first, then from `assets/`) and, optionally, numbers that replace its
defaults in `param` order, four per parameter vector:

```ogt
dress "Slot.frame[rarity=legendary]" {
  material "ui-kits/materials/rarity.ogt" 1.0 0.5 0.05 0  12 0 0 0  0.6 0 0 0  1.2
}
dress "ShopCard.frame[foil]" {
  material "ui-kits/materials/foil.ogt" 0.7
}
```

## On an element, in code

```ts
const foil = og.load.uiMaterial(world, "ui-kits/materials/foil.ogt");
og.ui.column("Card", desc, () => {
  if (foil.ok) og.ui.setDraw({ layer: { material: "custom", custom: foil.value, params: [0.7] } });
  // the card's children: drawn offscreen, composited through the foil
});
```

Animate a parameter by passing a new value each tick (a burn's `progress`
from 0 to 1 over its exit, eased with the style's exit curve), or from a
timeline (`og.ui.defineTimeline`).

## Tuning

`og dev` hot-reloads a preset when its WGSL or `param` defaults change,
and `ui:material:ui-kits/materials/foil.ogt.amount=0.8` sets a default
live without a cook (docs/ui.md §16.3); write the value you keep into the
`param` line. Review on a plain ground (docs/ui.md §15.4): a material is
judged by the card it finishes, not by a busy backdrop.

## Writing a new one

Copy the closest preset and change its `fn material(m: MaterialInput, p:
Params) -> vec4<f32>`: `layer(uv)` reads the element's pixels (linear,
straight alpha), `layer_texel()` is one CSS px in uv, `m.pixel` and
`m.size` are CSS px, `m.time` the UI clock (deterministic), and
`value_noise(p)` and `hash12(p)` are lattice noise. At most four `param`
lines; the cook validates the WGSL for Metal, Vulkan and DX12.

The sheet (`og ui sheet materials`) shows each preset on a card in two
styles; a new preset adds its card there.
