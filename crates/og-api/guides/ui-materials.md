# UI materials

Per-element shader finishes for a game's UI: the foil, holo, burn and rarity presets of the `materials` kit, how to put one on a styled part or an element, how to tune it, and how to write a new one.

`og ui add materials` installs the presets into `assets/ui-kits/materials/`
as ordinary `type ui-material` assets (an `.ogt` with `param` lines beside
its WGSL), the game's own to edit. The built-in `dissolve`, `shine` and
`hologram` materials need no install (`material shine -0.4 0.2 0.8 0.6` in
a dressing).

## 1. The presets

| Preset | What it does | Parameters, in order |
|---|---|---|
| `foil.ogt` | one broad soft sheen band sweeping across with the clock or the tilt, over a fine metal grain with a few glints | `amount` 0–1, `grain` CSS px (1–3), `sweep` vec2 (x sweeps/s, y an offset −1–1 the game drives), `tint` linear RGB |
| `holo.ogt` | fine iridescent stripes whose hue walks across the element and with the clock, weak enough that the art keeps its colour | `amount` 0–1 (0.2–0.45), `scale` CSS px a stripe (4–12), `speed` cycles/s, `saturation` 0–1 |
| `burn.ogt` | the element burns away from noise inside its own shape: char, ember, a white-hot rim, a glow into the hole | `progress` 0–1, `edge` CSS px (3–8), `scale` CSS px a noise cell, `glow` linear RGB |
| `rarity.ogt` | a tier-coloured glow in from the edge, a bright rim, a breath | `color` linear RGB, `size` CSS px, `pulse` breaths/s, `strength` 0–2 |

## 2. On a part, in the style

A dressing names the preset by its asset path (relative to the style file
first, then from `assets/`) and, optionally, numbers that replace its
defaults in `param` order, four per parameter vector:

```text
dress "Slot.frame[rarity=legendary]" {
  material "ui-kits/materials/rarity.ogt" 1.0 0.5 0.05 0  12 0 0 0  0.6 0 0 0  1.2
}
```

## 3. On an element, in code

Load it with og.load.uiMaterial and give the element a custom layer
material through og.ui.setDraw; the element's children are drawn
offscreen and composited through it.

```ts
const foil = og.load.uiMaterial(world, "ui-kits/materials/foil.ogt");
og.ui.column("Card", desc, () => {
  if (foil.ok) og.ui.setDraw({ layer: { material: "custom", custom: foil.value, params: [0.7] } });
});
```

- Animate a parameter by passing a new value each tick (a burn's
  `progress` from 0 to 1 over the exit, on the style's exit curve) or from
  a timeline (og.ui.defineTimeline).
- A foil reads as metal when its band follows the player: set `sweep` to
  `0 <offset>` and drive the offset from −1 to 1 with the pointer over the
  card or the device's tilt. Keep the clock sweep for cards nobody touches.

## 4. Tuning and review

`og dev` reloads a preset when its WGSL or `param` defaults change, and
`ui:material:<path>.<param>=<value>` on the command channel sets a default
live without a cook; write the value kept into the `param` line. Judge a
material on a plain ground by the card it finishes (screens §5), not over
a busy backdrop. `og ui sheet materials` shows each preset on a card in
two styles.

## 5. A new material

Copy the closest preset and change its `fn material(m: MaterialInput, p:
Params) -> vec4<f32>`: `layer(uv)` reads the element's pixels (linear,
straight alpha), `layer_texel()` is one CSS px in uv, `m.pixel` and
`m.size` are CSS px, `m.time` is the UI clock (deterministic), and
`value_noise(p)` and `hash12(p)` are lattice noise. At most four `param`
lines; the cook validates the WGSL for Metal, Vulkan and DX12. A new
preset adds its card to the kit's sheet.
