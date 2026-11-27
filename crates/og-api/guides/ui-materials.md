# UI materials

Shader resource presets for UI finishes: foil, holo, burn and rarity.
Load a material, then attach it to a TSX primitive's `material` prop.
Loading alone does not apply it. The material processes the node and its
children after their filters, before the group's mask and opacity.

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
`content/materials/materials/` as typed TS declarations, with WGSL under
`sources/materials/materials/` and discovery metadata under
`assets/materials/materials/`. Their IDs are `base/materials/materials/<preset>`.
An installed copy is not replaced silently. These are resource declarations,
not executable components or styles.

## 1. The presets

| Preset | What it does | Parameters, in order |
|---|---|---|
| `foil.ts` | one broad soft sheen band sweeping across with the clock or the tilt, over a fine metal grain with a few glints | `amount` 0–1, `grain` CSS px (1–3), `sweep` vec2 (x sweeps/s, y an offset −1–1 the game drives), `tint` linear RGB |
| `holo.ts` | fine iridescent stripes whose hue walks across the element and with the clock, weak enough that the art keeps its colour | `amount` 0–1 (0.2–0.45), `scale` CSS px a stripe (4–12), `speed` cycles/s, `saturation` 0–1 |
| `burn.ts` | the element burns away from noise inside its own shape: char, ember, a white-hot rim, a glow into the hole | `progress` 0–1, `edge` CSS px (3–8), `scale` CSS px a noise cell, `glow` linear RGB |
| `rarity.ts` | a tier-coloured glow in from the edge, a bright rim, a breath | `color` linear RGB, `size` CSS px, `pulse` breaths/s, `strength` 0–2 |

## 2. Loading a preset

Use the installed asset path with the material loader:

```ts
const material = og.load.uiMaterial(world, "base/materials/materials/rarity");
if (!material.ok) throw new Error(material.reason);
```

The result is a material handle, not a class, texture or TSX child.
Pack variants expand classes; attach the material explicitly to a primitive.

## 3. Attach and bind parameters

```tsx
import { Text, View } from "@origingame/engine/ui";

<View class="rounded-card bg-surface text-fg p-4"
  material={{ shader: material.value, params: { strength: 0.7, color: [0.6, 0.2, 0.9] } }}>
  <Text>A material over the entire subtree</Text>
</View>
```

Parameters use the shader's declared names and finite scalar/vector values;
unknown names, wrong vector widths and unloaded handles are errors. Omitted
parameters follow the loaded asset's tuned defaults. JSX expressions are
reactive: bind a progress signal in `params` rather than generating classes
or recompiling the shader. Set `material={undefined}` to remove it.

Nested materials compose from the inside out. Blur and drop shadow run
before the node's material; its mask, opacity and ancestor filters compose
afterward. Backdrop glass remains separate. Materials share the filter
layer/area budget; exceeding it reports a warning, not silent success.
`m.time` advances with the unscaled UI clock without rebuilding static UI.
The game controls animation parameters and reduced-motion alternatives;
do not bake essential information into a moving sheen alone.

## 4. Tuning and review

Edit defaults in the declaration's `params` array and shader code in its WGSL,
then cook to validate the resource. `og dev` reloads changed material
resources. A loaded material's defaults can also be tuned through
`ui:material:<path>.<param>=<numbers>` on the command channel; the dump
lists the loaded declarations and current values. Save chosen defaults in
the declaration's `params` array. Explicit per-node `params` override tuned defaults.
Loading/cooking alone proves no visual finish; review actual compositing.
Judge a finish on a plain ground by
the card it decorates (screens §5), then check it over the game's scene.

## 5. A new material

Copy the closest preset and change its `fn material(m: MaterialInput, p:
Params) -> vec4<f32>`: `layer(uv)` reads the element's pixels (linear,
straight alpha), `layer_texel()` is one CSS px in uv, `m.pixel` and
`m.size` are CSS px, `m.time` is the UI clock (deterministic), and
`value_noise(p)` and `hash12(p)` are lattice noise. At most four parameters;
the cook validates the WGSL for Metal, Vulkan and DX12. Keep
shader-resource validation distinct from TSX integration and visual
acceptance; a new preset still needs a rendered review card.

Declare its code and uniforms in `content/ui/finish.ts`:

```ts
import { file, input } from "@origingame/engine/author";

export default file["ui-material"]({
  code: input.text("sources/ui/finish.wgsl"),
  params: [{ name: "strength", kind: "float", default: 0.7 }],
});
```

The source path is relative to its content module, not this TS file.
The cook captures the WGSL and imported TS modules as dependencies before
planning. Changing either invalidates the cached payload. Load `base/ui/finish`.
