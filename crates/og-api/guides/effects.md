# Effects

Which effect tool to reach for (a ready-made pack, a pixel effect, a composed effect, one emitter, a trail, a beam, a decal, a screen effect, a hit stop and a shake), how to write a 3D effect from nothing and look at it, and what an effect may cost.

The API: `og api show vfx`, `og api show og.ParticleSystem`, `og api show og.Decal`. Every line of the three effect files is in `effect-files`; ready-made effects are in `effect-packs`, pixel effects in `pixel-vfx`.

## 1. What to use for what

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

## 2. A composed effect from nothing

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
- Read og.VfxEvent for the frame the hit lands; the drawn parts never
  feed gameplay.
- An effect that stands on its origin (a pillar of fire) is spawned on the
  ground; one that plays about it (a ring, a shield, an orb) is spawned at
  the height of its middle. The preview's anchor says which.

## 3. Choosing elements

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

## 4. Budget

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
