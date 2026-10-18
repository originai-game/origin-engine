# Effects showcase

Every og-render effect a game declares from TypeScript (the IDL's
`render` and `load` modules: `PostEffects`, `Lighting2D`, `Light2D`,
`Occluder2D`, `Decal`, `ParticleSystem`, `Trail`, `Beam`,
`og.render.ribbon`, the sprite and material effect fields, screen-space
sprites and text, `og.load.lut`, `og.load.emitter`, a skinned model posed
by `og.anim`), on five stations, with a screen-space HUD and an `og.ui`
badge over every frame.

```sh
og run templates/fx-showcase                        # the window
og test templates/fx-showcase                       # the playtest below
OG_FX_CAPTURE=1 og test templates/fx-showcase       # re-record this tier's references
python3 templates/fx-showcase/tools/make-assets.py templates/fx-showcase/assets   # regenerate textures and the LUT
```

| Station (`station:<name>`) | What it shows |
|---|---|
| `post` | ambient occlusion (GTAO), outlines, god rays from a low sun, bloom on an emissive ball, the `luts/teal-orange.cube` grading, vignette, chromatic aberration, film grain |
| `lens` | depth of field (high quality, focused on the near sphere) and per-object motion blur on a box orbiting fast |
| `fx` | a campfire: `fx/fire.ogt` (a flickering flame sheet), `fx/smoke.ogt` (alpha, noise), `fx/sparks.ogt` (bursts, velocity-stretched, bouncing off the depth buffer); decals from `textures/decals.png` (scorch, an emissive rune, an arrow, a splat); a trail behind an orbiting wisp; a jittering, scrolling beam between two pylons; a rainbow ribbon submitted every tick |
| `2d` | an orthographic camera over lit tiles: two `Light2D`s (one orbiting), crates that cast shadows (`castShadow`), an `Occluder2D` wall, a crate with a hit flash, an additive glowing orb (`glow`, `PostEffects.glow`), `fx/embers2d.ogt` drawn as 2D sprites |
| `stylised` | toon (with `textures/toon.png` as the ramp) and an outline, rim light, dissolve with a glowing edge, a hit flash pulse, screen flash and camera shake on the same beat, the skinned `walker.glb` walking with its first blendshape breathing |

`tests/frames.og.ts` shows each station, waits for its effects (TAA,
particles) to settle, takes its frame at 1280×720 and compares it with
`references/<tier>/<station>.png` under og-tests' rule; it also checks
that the HUD's opaque accent bar is exactly its authored colour on every
station (the overlay is drawn after tone mapping, grading, vignette, grain
and TAA). `references/metal` is recorded on Apple Metal, `references/vulkan-cpu`
on Mesa lavapipe (CI's tier) in the platform's E2B sandbox.

`dump_entity:stage` reads the `Stage` component (the station and how many
ticks it has been shown).
