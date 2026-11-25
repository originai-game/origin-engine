# UI materials

Shader resource presets for UI finishes: foil, holo, burn and rarity.
The assets remain cookable and loadable, but the TSX UI currently exposes
no custom-material attachment prop or pack slot field. Loading a material
does not apply it to a component. Do not use the former document CSS
`-og-material` or `<og-overlay>` syntax: neither is a TSX API.

### Backdrop glass is optional

Backdrop filtering is separate from custom material presets. A TSX
element or pack variant uses supported classes such as
`backdrop-blur-regular`, with a theme ground and paired ink. For example,
an existing pack's `Panel.frame` slot can supply its fill, blur and ink;
the component only names the slot:

```tsx
import { Text, View, packClass } from "@origingame/engine/ui";

<View class={`p-4 ${packClass("Panel", "frame")}`}>
  <Text>Readable content on the pack's panel ground</Text>
</View>
```

There is no `:root` glass opt-in, `-og-glass-backdrop` media query or
stylesheet cascade in the TSX authoring model. Keep the ground readable
without blur, and review it over bright and dark scenes on desktop and
phone. A blur hidden behind an opaque fill still costs work. Backdrop
classes do not enable the custom foreground finishes below.

`og ui add materials` installs the presets into
`assets/materials/materials/` as ordinary `type ui-material` assets (an
`.ogt` with `param` lines beside its WGSL), the game's own to edit, named
`materials/materials/<preset>.ogt`. An installed copy is not replaced
silently. These are resource assets, not executable components or styles.

## 1. The presets

| Preset | What it does | Parameters, in order |
|---|---|---|
| `foil.ogt` | one broad soft sheen band sweeping across with the clock or the tilt, over a fine metal grain with a few glints | `amount` 0–1, `grain` CSS px (1–3), `sweep` vec2 (x sweeps/s, y an offset −1–1 the game drives), `tint` linear RGB |
| `holo.ogt` | fine iridescent stripes whose hue walks across the element and with the clock, weak enough that the art keeps its colour | `amount` 0–1 (0.2–0.45), `scale` CSS px a stripe (4–12), `speed` cycles/s, `saturation` 0–1 |
| `burn.ogt` | the element burns away from noise inside its own shape: char, ember, a white-hot rim, a glow into the hole | `progress` 0–1, `edge` CSS px (3–8), `scale` CSS px a noise cell, `glow` linear RGB |
| `rarity.ogt` | a tier-coloured glow in from the edge, a bright rim, a breath | `color` linear RGB, `size` CSS px, `pulse` breaths/s, `strength` 0–2 |

## 2. Loading a preset

Use the installed asset path with the material loader:

```ts
const material = og.load.uiMaterial(world, "materials/materials/rarity.ogt");
if (!material.ok) throw new Error(material.reason);
```

The result is a material handle, not a class, texture or TSX child.
`HostProps` and the typed style record have no material attachment field;
pack `variants` expand class strings and cannot attach one either.

## 3. Finishes available to TSX

Use pack artwork (`nine-*`), gradients, supported filters/backdrop classes
and class-based motion for component finishes. Use `Particles` or the
library's `Flourish` for emitter effects. Those mechanisms do not claim to
reproduce a custom shader's foil or burn: per-subtree material compositing
needs an explicit supported attachment API before a game can use these
presets that way. Do not substitute arbitrary inline CSS or cast an
unsupported `material` prop past TypeScript.

## 4. Tuning and review

Edit defaults in the asset's `param` lines and shader code in its WGSL,
then cook to validate the resource. `og dev` reloads changed material
resources. A loaded material's defaults can also be tuned through
`ui:material:<path>.<param>=<numbers>` on the command channel; the dump
lists the loaded declarations and current values. Save chosen defaults in
the asset's `param` lines. This tunes the resource, not a TSX attachment:
there is no store/custom-property binding to per-element parameter vectors.
Loading/cooking alone proves no visual finish; review actual compositing
once an attachment path is available. Judge a finish on a plain ground by
the card it decorates (screens §5), then check it over the game's scene.

## 5. A new material

Copy the closest preset and change its `fn material(m: MaterialInput, p:
Params) -> vec4<f32>`: `layer(uv)` reads the element's pixels (linear,
straight alpha), `layer_texel()` is one CSS px in uv, `m.pixel` and
`m.size` are CSS px, `m.time` is the UI clock (deterministic), and
`value_noise(p)` and `hash12(p)` are lattice noise. At most four `param`
lines; the cook validates the WGSL for Metal, Vulkan and DX12. Keep
shader-resource validation distinct from TSX integration and visual
acceptance; a new preset still needs a review card when it can be drawn.
