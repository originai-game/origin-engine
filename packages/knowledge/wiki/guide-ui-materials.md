<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `ui-materials` guide: UI materials

Per-element shader finishes for a game's UI: the foil, holo, burn and rarity presets of the `materials` kit, how to put one on a styled part or an element, how to tune it, and how to write a new one.

`og ui add materials` installs the presets into `assets/ui-kits/materials/`
as ordinary `type ui-material` assets (an `.ogt` with `param` lines beside
its WGSL), the game's own to edit. The built-in `dissolve`, `shine` and
`hologram` materials need no install (`material shine -0.4 0.2 0.8 0.6` in
a dressing).

<a id="guide-ui-materials-1"></a>
### `ui-materials §1` The presets

| Preset | What it does | Parameters, in order |
|---|---|---|
| `foil.ogt` | one broad soft sheen band sweeping across with the clock or the tilt, over a fine metal grain with a few glints | `amount` 0–1, `grain` CSS px (1–3), `sweep` vec2 (x sweeps/s, y an offset −1–1 the game drives), `tint` linear RGB |
| `holo.ogt` | fine iridescent stripes whose hue walks across the element and with the clock, weak enough that the art keeps its colour | `amount` 0–1 (0.2–0.45), `scale` CSS px a stripe (4–12), `speed` cycles/s, `saturation` 0–1 |
| `burn.ogt` | the element burns away from noise inside its own shape: char, ember, a white-hot rim, a glow into the hole | `progress` 0–1, `edge` CSS px (3–8), `scale` CSS px a noise cell, `glow` linear RGB |
| `rarity.ogt` | a tier-coloured glow in from the edge, a bright rim, a breath | `color` linear RGB, `size` CSS px, `pulse` breaths/s, `strength` 0–2 |

<a id="guide-ui-materials-2"></a>
### `ui-materials §2` On a part, in the style

A dressing names the preset by its asset path (relative to the style file
first, then from `assets/`) and, optionally, numbers that replace its
defaults in `param` order, four per parameter vector:

```text
Slot::frame[rarity=legendary] {
  -og-material: "ui-kits/materials/rarity.ogt" 1.0 0.5 0.05 0  12 0 0 0  0.6 0 0 0  1.2
}
```

<a id="guide-ui-materials-3"></a>
### `ui-materials §3` On an element, in a document

An element takes the same preset with `-og-material` in its document's
`<style>`: the asset path in quotes, then numbers over its defaults in
`param` order, four per parameter vector. Any number may be a custom
property, so a store drives it through `style:`:

```svelte
<div id="Card" style:--burn={$burn}>…</div>
<style>
#Card { -og-material: "ui-kits/materials/burn.ogt" var(--burn) 0 0 0 6; }
</style>
```

The element and its children are drawn offscreen and composited through it;
`og.load.document` loads the materials its declarations name.
`<og-overlay effect="material …">` composites the whole frame under it
through one the same way.

- Animate a parameter from a store each tick (a burn's `progress` from 0
  to 1 over the exit) or with a CSS `@keyframes` of the custom property.
- A foil reads as metal when its band follows the player: set `sweep` to
  `0 var(--tilt)` and drive the tilt from −1 to 1 with the pointer over the
  card or the device's tilt. Keep the clock sweep for cards nobody touches.

<a id="guide-ui-materials-4"></a>
### `ui-materials §4` Tuning and review

`og dev` reloads a preset when its WGSL or `param` defaults change, and
`ui:material:<path>.<param>=<value>` on the command channel sets a default
live without a cook; write the value kept into the `param` line. Judge a
material on a plain ground by the card it finishes (screens §5), not over
a busy backdrop. `og ui sheet materials` shows each preset on a card in
the core default's light and dark schemes.

<a id="guide-ui-materials-5"></a>
### `ui-materials §5` A new material

Copy the closest preset and change its `fn material(m: MaterialInput, p:
Params) -> vec4<f32>`: `layer(uv)` reads the element's pixels (linear,
straight alpha), `layer_texel()` is one CSS px in uv, `m.pixel` and
`m.size` are CSS px, `m.time` is the UI clock (deterministic), and
`value_noise(p)` and `hash12(p)` are lattice noise. At most four `param`
lines; the cook validates the WGSL for Metal, Vulkan and DX12. A new
preset adds its card to the kit's sheet.
