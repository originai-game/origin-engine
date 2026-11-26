<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `effects` guide: Effects

Which effect tool to reach for (a ready-made pack, a pixel effect, a composed effect, one emitter, a trail, a beam, a decal, a screen effect, a hit stop and a shake), how to write a 3D effect from nothing and look at it, and what an effect may cost.

The API: `og api show vfx`, `og api show og.ParticleSystem`, `og api show og.Decal`. Every line of the three effect files is in `effect-files`; ready-made effects are in `effect-packs`, pixel effects in `pixel-vfx`.

<a id="guide-effects-1"></a>
### `effects §1` What to use for what

| The effect | Use | Why |
|---|---|---|
| a common effect (a hit, an explosion, a heal, a level-up, fire, a portal) | an effect pack: `og vfx add <pack>`, then og.load.vfx and og.vfx.spawn | ready-made and measured; `effect-packs` lists each with its size, anchor and cost |
| a 2D game's effect | a pixel effect (`type vfx2d`, `pixel-vfx`), or a `flipbook` element in a composed effect | baked into a sheet at cook: hard pixels, no per-frame simulation |
| a 3D effect with parts in time (sparks, then a ring, a light, a scorch, a shake) | a composed effect (`type vfx`, §2) | one timeline, one spawn, events back to the game |
| a lasting source (a torch, smoke from a chimney, rain round the camera) | one emitter (`type particles`) on an og.ParticleSystem | no timeline needed; it runs while the entity lives |
| a streak behind a moving thing (a sword, a dash, a projectile) | og.Trail (its `effect` an effect material for a scrolling slash), or a `trail` element | the positions it passed through, faded by age |
| a line between two points (a laser, a tether, lightning) | og.Beam, or a `beam` element; og.render.ribbon through your own points | scrolls and jitters on the tick |
| a shaped glow that lasts (a shield dome, a portal's swirl, a slash arc held on a weapon) | og.EffectMesh with an effect material (og.load.effect), or a `mesh` element | a mesh the game owns, its scrolling, eroding and fresnel look run on `life` |
| a mark on the ground or a wall (a scorch, a footprint, a bullet hole) | og.Decal, or a `decal` element | projected on whatever is there, lasting |
| a marker on the ground the game moves (a selection ring, an attack telegraph, a placement ghost) | og.Decal on its own entity | follows the ground under it; colour, opacity and scale are fields a system writes |
| a route drawn over the ground (a move order, a path preview) | og.render.lines each tick through the route's points, lifted a few cm | the points change with the route |
| a sprite's hit flash, a dissolve | og.tween.flash, an effect material's `erosion` on a mesh | changes the thing hit, not a new object |
| the whole screen (a white flash, a blur, a vignette) | og.PostEffects | after the scene; one frame or a tween |
| weight on an impact | og.time.hitStop and og.camera.shake (a `shake` element) | the frame holds, the view kicks |
| an effect in a cutscene, or where a prop breaks | a timeline's `vfx` track (`animation §13`); og.Destructible.breakVfx and hitVfx | the timeline or og.destruction spawns the og.Vfx and it ends itself |
| ghosts of a moving sprite | og.Afterimage | its own past poses |

Several entries name the same idea twice, a component and an element: the
component is for something the game owns and moves (a sword's trail for the
whole fight), the element for a part of one effect (a dash's trail for its
0.3 s).

- Sprite sheets come four ways: a pixel effect's baked sheet, og.Flipbook
  on a sprite, an emitter's `sheet` (a frame per particle) and an emitter's
  `array` (a texture-array layer per particle, sprite mode). Use the
  emitter's when there are many of them, og.Flipbook when there is one.
- Dissolves come three ways: an effect material's `erosion` (effects), a
  material shader (any mesh, `looks`) and a UI material (`ui-materials`).
  Use the effect material inside an effect.
- Distortion is an emitter's `distortion` (heat haze, shock rings on
  billboards) or a material shader; a full-screen ripple is og.PostEffects.
  An effect material's `flowMap` warps its own texture (a swirling
  portal), not what is behind it.
- Lightning is a beam's `jitter` (og.Beam or a `beam` element: one bolt
  between two points, re-jittered every tick), a pixel effect's `bolt`
  layer (`pixel-vfx`, baked) or og.render.ribbon through points you
  compute (forks). Use the beam in 3D, `bolt` in 2D.
- A 3D effect plays under an orthographic camera (a 2D game with a 3D
  stage, a side view) as it does in perspective, its metres the camera's
  metres (Camera.height across the view). What lies flat on the ground (a
  `mesh ring` laid down, a `decal`, `align facing 0 1 0` ripples and
  scorches) is edge on to a level side view and disappears: tilt the
  camera down, or stand the part up. A level view sees the ground edge on
  too, so `collision` off the drawn depth misses it and sparks fall
  through: give them `collision colliders` or `die`. A pixel game's
  effects are `pixel-vfx`.
- Ground markers are world drawing, not UI: the UI draws over the frame
  and never on the ground.
  - A selection ring is a decal whose texture is a ring, on an
    entity parented to the unit (or moved to it each tick), scaled to the
    unit's footprint and tinted by its team; hide it with `visible`.
  - An attack telegraph is a decal over the area to be hit whose opacity
    (or the scale of a second, filling decal) a system grows over the
    wind-up, with `emissive` for the last moments; remove it when the
    attack lands.
  - A placement ghost is a decal (and, for height, a translucent mesh)
    moved each tick to where the pointer's ray meets the ground, tinted
    by whether the spot is free.
  - A path line is og.render.lines through the route's points, each
    lifted a few cm above the ground; og.Lines keeps a route that does not
    change.
- Some particles are their system's own and stay so: the ocean's foam
  comes from its waves (`render`), destruction's debris are rigid
  bodies (`physics`), `<og-emitter>` is the UI's (`components`) and a
  pixel effect's `particles` layer is baked at cook. Use them where they
  are; an emitter beside them is for what they do not draw.

<a id="guide-effects-2"></a>
### `effects §2` A composed effect from nothing

An `.ogt` under `assets/`. Each element starts `at` a time and lasts a
`life`; `impact` is the moment the game hears:

```text
type vfx
version 2
---
duration 0.7
impact 0.05
glow {
  at 0
  life 0.2
  offset 0 0.4 0
  size curve 0 0.6 1 1.6
  color 3 1.6 0.6
}
emitter "fx/sparks.ogt" {
  at 0
  offset 0 0.3 0
}
mesh ring {
  at 0
  life 0.4
  rotate -90 0 0
  effect "fx/shock.ogt"
  scale curve 0 0.4 1 3
}
light {
  at 0
  life 0.4
  offset 0 0.6 0
  color 1 0.6 0.25
  intensity curve 0 20 1 0
  distance 6
}
shake {
  at 0.05
  trauma 0.3
}
```

`fx/sparks.ogt`, the emitter it names:

```text
type particles
version 1
---
maxParticles 128
shape sphere 0.1
spread 180
rate 0
burst 0 60
looping false
lifetime 0.3 0.6
speed 4 8
size 0.03 0.06
gravity 0 -9.8 0
drag 1.5
collision 0.3 0.2
align velocity 0.08
color 4 2 0.6
```

`fx/shock.ogt`, the ring's effect material:

```text
type effect
version 1
---
color 3 2 1
alpha curve 0 1 1 0
fresnel 2 1 invert
blend additive
```

Then play it where the hit lands:

```ts
const hit = must(og.load.vfx(world, "fx/hit.ogt"));
must(og.vfx.spawn(world, { asset: hit, position: at }));
```

- `og cook` refuses a line a file does not take and names the ones it
  does; `effect-files` lists them all with their defaults.
- Look at it before playing it in the game: `og vfx preview fx/hit.ogt`
  writes six frames in time order, the impact among them, and prints its
  measured size, anchor, draw calls and peak GPU milliseconds (§4), and
  the engine's warnings about it. Judge its frames as `art-review §7`
  says, and capture it in the game as `captures §6` says.
- For context, `og vfx preview fx/hit.ogt --project-look --scene scenes/courtyard.ogt --position 2,0.5,-1 --camera 2,2,6,2,1,-1,50`
  checks the same six moments in the project's look and a cooked scene instead. Fog, exposure and grading stay active; scene resources apply after the look. Use `--look looks/alternate.ogt` instead of `--project-look` to compare an explicit look. The position is exact (no automatic ground lift); the camera is world-space, or without `--camera` it auto-frames at that position. Bounds remain an isolated black-stage measurement, while draws and GPU growth are measured in the selected scene. No game entry or gameplay system runs in this preview.
- Read og.VfxEvent for the frame the hit lands; the drawn parts never
  feed gameplay.
- An effect that stands on its origin (a pillar of fire) is spawned on the
  ground; one that plays about it (a ring, a shield, an orb) is spawned at
  the height of its middle. The preview's anchor says which.

<a id="guide-effects-3"></a>
### `effects §3` Choosing elements

- A burst of light reads from a `glow` (a soft sphere facing the eye), a
  `light` (what it lights around it) and bloom, not from a hundred white
  particles: three cheap things instead of one expensive one.
- Sparks are small, fast and few: `align velocity` streaks them, `drag`
  slows them, `gravity` drops them. Smoke is large, slow and alpha
  blended, with `wind` and a soft fade (`softDistance`).
- A ring or a shock wave is a `mesh ring` with an effect material, not
  particles in a circle: one draw, any size.
- Large soft sprites cost by the pixels they cover, not by their count:
  20 smoke puffs filling the screen cost more than 2,000 sparks.
- Additive blending is order independent and never darkens; alpha
  blending is for smoke, dust and solid chips (`render mesh` that is
  `lit`).
- One effect, several variants: spawn the same file with `hue` (a fire
  turned blue for ice, about 3.1), `tint` (darker, brighter) and
  `emission` (fewer particles), not a copy of it per colour. What the
  variants add or remove is a `param`: `param crit 0` in the file, `when
  crit` on the extra burst, `amount power` on the light and the shake,
  and og.vfx.spawn's `params: [1, 1.5]` picks them per instance. The hue
  turns the whole drawn colour of particles, meshes and glows, textures
  included; a trail's, beam's or decal's colour turns, but a coloured
  texture on one keeps its colours, so give those greyscale textures.

<a id="guide-effects-4"></a>
### `effects §4` Budget

What an effect may cost, measured by `og vfx preview` as its peak GPU
milliseconds over an empty frame. The engine reports the numbers; these
are the thresholds to hold them to. They are the reference host's (an
NVIDIA L4, release, 1280×720) until the ratio of the target devices to it
is measured:

| What | Budget |
|---|---|
| a one-shot effect (a hit, an explosion), at its peak | 0.5 ms |
| a looping effect (a torch, an aura, weather), all the time | 0.2 ms |
| every effect on screen at once | 1.5 to 2 ms |

- `og vfx preview --together a.ogt b.ogt ...` plays several at once on a
  grid and reports the frame's and each pass's milliseconds on the GPU,
  and the CPU's for recording them (`cpu`): the on-screen total.
- For scale: the twenty starter effects playing at once on the reference
  host add 0.32 ms to the frame at their peak and 0.18 ms typically.
  Almost all of it is the main pass drawing their sprites (overdraw); the
  particle step is 0.02 ms.
- Large sprites' cost does not fall with their number: if an effect is
  over budget because it covers the screen, shrink its sprites or use a
  few large parts instead (a `glow`, a `mesh`). Fewer particles helps
  only an effect of many small ones: og.Vfx.emission thins one instance
  (a distant fire at 0.5) without another file.
- The low graphics preset thins every emitter to half its particles
  (RenderSettings.particleEmission 0.5); an emitter that would keep fewer
  than 8 alive keeps more, so flashes and rings stay whole. Judge an
  effect at low as well as medium: one made of a few large sprites costs
  the same at both.
- Each preset caps the composed effects drawn at once (RenderSettings.maxVfx:
  32, 64, 128) and the particle slots among them (maxVfxParticles: 32768,
  65536, 131072). Past either, the oldest one-shot effects are left out
  first, then the oldest looping ones; their events, shakes and sounds
  still come. The probe's dump_render counts them (`vfx.dropped`): a
  dropped effect means a burst of them outlived the budget, so shorten
  their life or lower the slots their emitters hold (`maxParticles`).
- An emitter with `subEmitter` lines runs on the CPU; one with `events`
  is stepped in the tick. Keep both to the effects that need them.
- The frame as a whole is `testing §6`.

<a id="guide-effects-5"></a>
### `effects §5` Cook source textures and generated declarations

`type effect-texture`, `version 1` simulates and renders a source volume during cook, never in a game tick. `og api show file:effect-texture` lists every required parameter, unit and range. `smoke` and `fire` transport the authored density and heat while the source is active; neither chooses colours or emission for the author. `explosion` injects once and gives it the authored radial velocity. The solver uses semi-Lagrangian advection, a pressure projection, buoyancy, seeded curl forcing and vorticity confinement. Seeded detail modulates the source boundary; a directional Beer-Lambert sweep supplies self-shadowing from the authored light direction and ambient/direct strengths. It then integrates scattering/thermal emission through the cube along +Z. It is a coarse inviscid volume, not combustion or a compressible blast. The implementation's source and MIT licence are recorded under `third_party/stable-fluids`.

The asset id is its RGBA scattering sheet. `<id>#emission` is the corresponding thermal emission sheet, `<id>#array` and `<id>#emission#array` their chronological texture arrays. RGB is sRGB, alpha is straight coverage, empty cells and volume boundaries transparent. Sheet columns and rows are explicit/ceil(frames/columns); use that exact layout in `sheet` and give `frames` so unused cells never play. For example `texture "puff.ogt"` plus `sheet 2 2 3 life 1 blend`, or `array "puff.ogt#array" 3 life 1` in sprite mode. Use the emission output on an additive emitter with the same placement/timing and an authored intensity; it is not silently mixed into lit smoke. Frame interpolation is available on sheets; texture arrays retain the normal array playback semantics. No normal or motion map is needed by this camera-facing integration. Leave room in the unit cube for the whole plume: material escaping the open boundary is lost, not automatically fitted into the frame.

`noise`, `worley` and `curl` use the same Rust kernels as `og.procgen`, not a second noise library. Their output is linear data: noise/curl map the authored -range..range to 0..1; Worley maps first-site distance 0..range to 0..1. Values outside that range clamp. Curl encodes its three vector lanes in RGB around 0.5; increase `range` when high-frequency derivatives would otherwise saturate the encoding. Use them as `noise` for erosion/dissolve or `flowMap` for UV distortion. Coordinates span the unit square; `frequency` gives cycles per cube, `turbulence` its Z travel speed. These textures are not promised tileable. Fluid-only parameters are still explicit but unused in these modes; `range` is unused for volumes. Generation and texture encoding use the cook's BLAKE3 action/content cache; source parameters, engine identity and texture options invalidate it. Nothing writes images into assets/.

Each of `particles`, `effect` and `vfx` can instead contain one exclusive `generate { script "...ts"; seed ...; params {...}; input "name" "path" }` block (write each field on its own line). The existing restricted executor returns `EffectSource {text}`: body declarations without a type/version/header. The regular parser, linked asset checks and field baking process that body at the file's declared version. Recursive generation and mixing authored lines with generation are refused. Static imports, input bytes and parameters are tracked exactly as in `generating`; no filesystem, process or network is exposed. One module can generate a family by each file's explicit parameters, without another executor or a second effect schema.

This recipe cooks a small asymmetric three-frame volume, composes the three generated effect types and loads both sheet/array paths. Its dimensions keep the guide check short; choose resolution, steps and pressure iterations for the final asset, inspect every frame and check it in the project's own look (§2).

```ts recipe ticks=2
import { og } from "@origingame/engine";

export default og.game({ start(world) {
  const sheet = og.load.texture(world, "puff.ogt");
  const array = og.load.textureArray(world, "puff.ogt#array");
  const emission = og.load.texture(world, "puff.ogt#emission");
  const material = og.load.effect(world, "surface.ogt");
  const effect = og.load.vfx(world, "composed.ogt");
  og.assert.always(sheet.ok && array.ok && emission.ok && material.ok && effect.ok, "generated source outputs and linked effect load", {});
  if (!effect.ok) throw new Error(effect.reason);
  const info = og.vfx.info(world, effect.value);
  og.assert.always(info.ok && info.value.duration === 0.75 && info.value.elements.length === 1, "generated composition keeps its authored timeline", {});
  const instance = og.vfx.spawn(world, { asset: effect.value, position: {x: 2, y: 1, z: -3} });
  og.assert.always(instance.ok, "generated emitter spawns normally", {});
} });
```

```ogt recipe file=assets/puff.ogt
type effect-texture
version 1
---
mode explosion
size 16
frames 3
columns 2
grid 12
steps 1
dt 0.04
pressure 6
detail 0.8
vorticity 2
light -1 2 -1
ambient 0.2
direct 1
seed 17
frequency 3
range 1
turbulence 0.03
source 0.43 0.38 0.5
radius 0.22
inject 0
density 12
heat 1
buoyancy 0.1
expansion 0.4
decay 0.5
cooling 2
absorption 8
color 0.4 0.45 0.5
emission 1 0.2 0.01
```

```ogt recipe file=assets/surface.ogt
type effect
version 1
---
generate {
  script "effect.ts"
  params {
    kind surface
  }
}
```

```ogt recipe file=assets/emitter.ogt
type particles
version 1
---
generate {
  script "effect.ts"
  params {
    kind emitter
  }
}
```

```ogt recipe file=assets/composed.ogt
type vfx
version 2
---
generate {
  script "effect.ts"
  params {
    kind composition
  }
}
```

```ts recipe file=assets/effect.ts
import type { EffectSource, GeneratorContext } from "@origingame/engine/generate";

export default function effect(ctx: GeneratorContext<{kind: string}>): EffectSource {
  if (ctx.params.kind === "surface") return { text: 'main\nsoftDepth 0.15\nblend alpha\n' };
  if (ctx.params.kind === "emitter") return { text: [
    'maxParticles 1', 'rate 0', 'burst 0 1', 'looping false',
    'lifetime 0.75', 'speed 0', 'size 1.5', 'texture "puff.ogt"',
    'sheet 2 2 3 life 1 blend', 'effect "surface.ogt"',
    'colorOverLife {\nkey 0 1 1 1 0\nkey 0.15 1 1 1 1\nkey 0.8 1 1 1 0.7\nkey 1 1 1 1 0\n}',
  ].join('\n') };
  return { text: 'duration 0.75\nemitter "emitter.ogt" {\nat 0\n}\n' };
}
```
