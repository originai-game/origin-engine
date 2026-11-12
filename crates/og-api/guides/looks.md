# Rendering and looks

Rendering an Origin Engine game. The look as a preset the game extends, what belongs in the look and what in the scene, lighting and post order, 2D versus 3D, and what a frame costs.

The API: `og api show render`, `og api show atmosphere`,
`og api show terrain`, `og api show video`, `og api show camera`,
`og api show shapes`. A look's fields are the reflected resources
(RenderSettings, PostEffects, Stylise, Environment, Fog, Sky and the Art
blocks); `og api show <Resource>` gives each field with its range.

## 1. The look

A game's look is one file, `assets/looks/<id>.ogt`, which project.ogt's
`look` names. It `extends` a built-in preset by slug (the presets are the
engine's `templates/looks/<slug>.ogt`; the creation step picked one) and
overrides it field by field:

```text
type look
version 1
---
extends "faceted-low-poly"
mood "warm"
ui "default"
PostEffects {
  vignetteIntensity 0.3
}
```

- Change the look by adding fields to the game's file, never by setting
  the same resources from code at start: the look is cooked, checked
  against the reflected ranges, and read by the art brief (art-review §2).
  Code changes the frame only for moments (a hit flash, a slow-motion
  grade) and restores it.
- `mood` lays the mood's grading, light and palette over the preset
  before the game's own fields; it never changes the art contract's
  sizes, lines or shapes. Try the mood before hand-grading.
- `ui` names the UI style the screens wear (screens §3).
- `extends` may name another look of the game (`"looks/base.ogt"`) for a
  level or a scene's variant; chains are a few deep and cycles are
  refused. Editing a base re-cooks every look on it.
- A preset whose catalogue entry is not `full` support still lacks a
  primitive; the catalogue's `missing` names it. Do not promise the
  user that look's whole effect.

## 2. What goes where

- The look: grading, tone mapping, exposure, fog, sky, the stylise pass
  (pixelation, posterize, dither, outlines) and the art contract.
- The scene: lights, their shadows, reflection probes, light volumes,
  baked lightmaps, particles and decals.
- Never imitate a look's effect with geometry or textures (a painted-on
  outline, a pre-dithered texture): the stylise pass does it once,
  consistently, at every resolution.

## 3. Light

- One shadowed sun or moon, plus local shadowed lights only where the
  player looks; every shadowed local light renders its own shadow maps.
- Static levels bake: `og bake lightmaps` traces the scenes' static
  models and light volumes on the CPU and writes the lightmaps beside the
  scene. Rebake after moving static geometry; a stale lightmap shows as
  light in the wrong place, not as an error.
- `og bake probes` renders reflection probe captures ahead of time; an
  unbaked probe is drawn on first use, which costs that frame.
- Exposure and tone mapping are the look's; adjust them before raising
  light intensities, or every light ends up retuned.
- A dark mood still needs the player's silhouette and the next goal
  readable: keep a rim or fill on the player and the interactive objects.

## 4. 2D and 3D

- A 2D game uses sprites, tilemaps and the 2D lights on an orthographic
  camera; its publish build leaves the 3D world out (terrain, planar
  water, volumetric fog and the 3D fluids) and refuses scenes holding
  them, by file and line. `RenderCapabilities.world3d` says which world
  is present.
- Pixel art: set the pixel height in the look's Stylise and pixel-snap
  the camera; never scale sprites by fractional factors. The art
  contract's sprite sizes come from the look (art-review §2).
- 2.5D (isometric, side views of 3D models) is a 3D game with a fixed
  camera; it pays for 3D.

## 5. Cost

- Measure on the target device: the probe's timings split the GPU by
  pass. Shadow maps, transparency and full-screen post passes are the
  usual top three.
- Instancing is automatic for the same model and material; a level built
  from a few hundred distinct materials loses it. Share materials across
  props.
- Transparency is sorted per object and costs fill; many overlapping
  large transparent planes (fog cards, foliage) are the classic cliff.
- Particles: hundreds of emitters each emitting a few particles cost more
  than a few emitters emitting many.
- Rendering never blocks the engine thread on the web; a heavy frame
  drops frames, never ticks. A game whose simulation slows when the GPU
  is loaded reads something presentational (determinism §1).
- `og render --commands ... --contact-sheet COLS` gives a sheet of
  frames for a quick look across a run.

## 6. Stylise: order and cost

The Stylise primitives compose: a look sets Stylise together with
PostEffects and RenderSettings, so a game's art direction is data, not
engine code. A primitive at its zero value records no pass and builds
no pipeline.

- Order in the frame: depth bands colour the lit scene with the fog,
  before tone mapping. After tone mapping and PostEffects' grading come
  paint, hatching, halftone and ink, then posterize and the palette
  quantise through the dither, all at the logical resolution. The pixel
  grid is scaled up last.
- The UI and screen-space sprites are drawn over the scaled frame at the
  output's resolution and are never stylised.
- These turn on the depth-normal prepass that PostEffects.outline uses:
  a `surface` or `object` pattern space, hatchingDepth, inkDepth and
  paintDetail above 0. One prepass serves all of them, so the second
  costs little more than the first.
- `object` space also draws the opaque instances a second time in an
  object pass. Use it for moving or turning objects that must carry
  their pattern; use `surface` for static sets.

## 7. The pixel grid

Stylise.pixelHeight sets the scale and the width follows the output's
aspect. When the rounded width times the scale would pass the output's
width, the last column gives way, not the scale. For example, 240 rows
at 1280x720 are 426.67 columns, rounded to 427, and 427 x 3 = 1281 does
not fit. The grid is then 426 x 240 at 3x (1278 x 720), with a 1-pixel
border each side.

| Output | Heights that fill it |
|---|---|
| 720p | 180 (4x), 240 (3x), 360 (2x) |
| 1080p | 270 (4x), 360 (3x) |

- pixelUpscale 0 takes the largest scale that fits: 720 / 240 = 3,
  1080 / 240 = 4, rounded down.
- A smaller scale than the largest leaves the border it implies:
  240 lines at 2x in 1280x720 draw 854 x 480 inside a 213 x 120 border.
  Camera.background fills the border.
- pixelSnap keeps a moving orthographic camera scrolling by whole
  pixels, and keeps the edges of pixel sprites in a 3D set.

## 8. Where patterns lie

StyliseSpace places the paint canvas, the hatching, the halftone dots
and the dither cells.

- `screen` is a print over the frame. The pattern stays put and the
  scene slides under it, so a dither crawls as the camera moves.
- `surface` sticks the pattern to the scene, for example a tonal art
  map's strokes on the objects. Hatching and halftone keep about their
  pixel spacing by fading between whole octaves of world spacing. A
  dither's cells are at least ditherScale pixels on screen each way,
  each surface axis by whole octaves, so a receding floor's cells are
  long along its depth. This is the Obra Dinn look. The dither's sky
  cells lie on a cube round the camera, so turning carries them; the
  other patterns keep the screen pattern on the sky.
- `object` carries the pattern on each mesh instance, however it moves,
  turns, scales or is skinned. Terrain, scattered copies and vertices
  moved by a material shader keep the world pattern.
- The paint canvas's spacing: paintCanvasScale applies on screen (and on
  the sky), paintCanvasWorldScale on surfaces. 128 texels per metre
  repeats a 256-texel canvas every two metres.

## 9. Halftone, hatching and ink

- Halftone `cmyk` puts black at halftoneAngle, cyan 30 degrees before
  it, magenta 30 after and yellow 45 before. At the default 45 degrees
  that is 15, 75, 0 and 45.
- halftoneRegion `all` reproduces the tone on white paper, and the dots
  merge in the darks as print does. `shade` and `highlight` print on the
  darkest or lightest third of the tones. Their dots are Ben-Day dots: a
  tint printed over the region's own colour, never touching, so a flat
  shade stays one colour with separate dots on it (Hi-Fi Rush,
  Spider-Verse). They fade to flat ink on surfaces turned away from the
  camera, because dots squeezed into a few rows read as a checker. A
  floor seen from a game camera's height is the usual case.
- hatchingDepth finds its edges within one stroke spacing, with the
  same thresholds that inkDepth uses. Strokes then gather on the lines
  that inkDepth inks.
- hatchingKeepColor's wash is as light as one-ink hatching's paper. It
  keeps more of a strong colour's chroma than of a weak tint's, so props
  keep their colour while a greyish floor goes to paper. Conformed art
  keeps its authored colour, with half the chroma the lift would take.
  On a coloured hatchingPaper (a cream sketchbook, a grey card, a
  blueprint) the wash lies on the paper's colour at its own lightness.
- Ink without inkDepth draws on luminance edges over every surface,
  sprite and texture alike, and needs no prepass. Add inkDepth for
  contours round flat colours that luminance alone misses.

## 10. Paint and watercolour

- paintRadius is an anisotropic Kuwahara filter with eight sectors over
  a smoothed structure tensor (Kyprianidis 2009 and 2010). Grass and
  noise become strokes; edges and thin lines keep their shape.
- paintDetail keeps faces, hands and lettering drawn while grass and
  broad areas paint. It runs the prepass only where meshes are drawn.
- In a profile 2d project under the art contract, the cook applies the
  paint and watercolour fields once, in each texture's own texels.
- The watercolour wash follows Bousseau et al. 2006 and Montesdeoca et
  al. 2018 and is drawn over the painted frame.
- Edge darkening: a boundary is a step between two colour regions wider
  than a line (the colour averaged over six pixels either side differs),
  or a silhouette against something farther behind. One wash covers a
  seam, a crease or a neighbour of the same colour, so a texture's seams
  and a tiled floor get no rim. paintEdgeDarkening pools broadly over the
  source's structure instead.
- Granulation: the density is about one, from the paper's relief.
  That is paintCanvas's height about its mean in units of its spread,
  so any canvas reads as paper. Without a canvas it comes from a noise a
  few pixels across. A grain two to three pixels across breaks it. Each
  colour C becomes C(1 - (1 - C)(d - 1)) (Bousseau). Dark and saturated
  washes granulate and the wash keeps its mean value, where
  paintCanvasStrength's grain only darkens.
- Turbulence uses a noise about a hundred canvas texels across, plus one
  of half that, so a wash is lighter here and darker there.
- Dilution is the water in the wash. A lit 3D scene has darks no
  watercolour holds: deep shade and saturated foliage read as oil paint
  under the strokes. watercolorDilution thins the pigment over
  watercolorPaper (a layer that much less dense, by Beer and Lambert), so
  the darks lift towards the paper, hues keep and the paper stays the
  paper. 0.3 to 0.45 suits a daylight scene; above 0.6 only the
  silhouettes remain. Raise watercolorEdgeDarkening with it when shapes
  lose their edge.
- A rim pools only where there is pigment: the side of a boundary on
  the paper or on a pale wash (the sky behind a branch) stays clean.

## 11. Depth bands

Depth bands make layered-silhouette and stage looks. Use them for side
and three-quarter views, where the depth runs across the frame. Seen
from above, the play area lies at one depth and falls in one band.

## 12. MaterialStyle

- A terrain takes toon (the built-in three bands, matte) and shadowTint.
- toon passes over a material with a MeshPhysicalMaterial lobe
  (clearcoat, sheen, transmission, iridescence, anisotropy, ior), which
  keeps its physical shading. Set the material's own toon to override.
- shadowDarkness takes light from the toon ramp's dark band, from the
  side turned away and from cast shadows. It also removes the
  Environment's map and its hemisphere light there. At 1 the ramp's
  middle band keeps half its light, and every shadow is a solid black
  mass: the ink shadows of noir comics.
- Outlines are hull outlines: voxel and flat-shaded models stay clean.
  At a grazing angle a surface is lined only where it ends, so a
  1,600 m ground under a chase camera keeps its far edge and gets no
  band along the horizon.
- toonAmbient 1 brings back the smooth PBR ambient and reflections,
  which wash the bands out. Keep it low for clean cel shading.
- wireframe draws edges over drawn faces, which hide the edges behind
  them, as vector displays drew with hidden lines removed. Dark
  materials make the fill for vector and neon looks; emissive edge
  colours glow through bloom.

## 13. Sprites

- Texture arrays: pack a flipbook's frames as layers and its sprites
  batch across frames, one draw per array.
- Drop shadows let pixel sprites in a 3D set cast shadows that agree
  with the sun's. Under a low sun a drop shadow is at most five times the
  sprite's height long. A 2D camera sees the ground edge on, so use them
  with a perspective camera over the xz ground. Over terrain the shadow
  samples the heights under a 4 x 2 grid of its cells.
- toneMapped false is for world UI that must read the same at noon and
  at dusk: speech bubbles, name tags, markers.
- Sprite.outline is for a line the cook cannot bake: a scaled, tinted or
  run-time sprite. The quad grows to hold it, and a sheet's neighbouring
  frame never lines it.
- Palette swaps (docs/art.md §2.4) give team colours, a hit flash in
  palette, or a day and a night ramp from one index texture. A texture
  that is itself an index texture is drawn as it is.
- Sprite shaders: the hooks see `local` (the point on the quad, 0 to 1
  from its bottom-left corner), `uv`, `color` (the tint) and `time` (the
  scene clock). surface_color sees texture times tint and flash; an
  alpha of 0 cuts the pixel (a dissolve). A `mode sprite` shader may call
  `sprite_texel(uv)` to read the sprite's texture anywhere (distortion, a
  chromatic split). Dissolve, freeze, heat shimmer and palette effects
  are shaders, not engine features.
- Shader batching: sprites of one material, texture and blend are one
  draw. A material per sprite, each with its own parameters, is a draw
  per sprite.
- With a shader, the drop shadow takes the shader's alpha. The glow, the
  blob shadow and afterimages draw plain, as does a sprite drawn from a
  texture array. A faced sprite keeps the sprite order.
- A hooks shader that calls a helper only meshes have draws its sprites
  plain; dump_render's spriteShaderErrors names it.
- A HUD element made of sprites on a UI panel sits under or over the
  whole UI (UI_LAYER). Draw it only while its screen shows.

## 14. Particles and decals

- Particles simulate in compute with nothing read back, sprite-mode
  emitters included. Emitters with sub-emitters run on the CPU, with
  identical results.
- `collision` bounces particles off the drawn depth (`depth`, the default
  in 3D) or off the scene's og.Collider boxes, spheres, capsules and
  terrains (`colliders`, on or off screen, always in sprite mode). Both
  run in compute.
- An emitter with an `events` line is stepped in the tick by the
  engine's particles system, with the engine's randomness and fixed
  math. The renderer draws that simulation as it stands. Give `events` only to emitters whose
  particles gameplay reads.
- Decals: tens of thousands can be placed, because each pixel walks only
  the decals whose box can reach it.

## 15. Reflection probes

- Use a probe for interiors that the sky must not light: a chrome
  sphere in a room shows the room.
- The probe holds light as it reaches surfaces: the sky scaled by the
  environment's intensity (RenderSettings.environmentIntensity,
  og.Environment.intensity), as outside the box, and the surfaces as
  drawn. Crossing the box's face changes what is reflected, not how
  bright it is.
- `realtime` probes render six faces per update. Raise `interval` before
  adding probes.

## 16. Applying looks and quality presets

- Apply a look in GameDesc.start (or name it in GameDesc.look), or at any
  tick for a level's mood, a flashback or a photo mode.
- The art contract's resources (ArtPalette, ArtPixels, ArtTexels,
  ArtShape, ArtLine, ArtMotion, ArtCamera) need no write declaration. A
  system that declares the frame's resources can therefore apply looks
  that carry a contract; the undeclared contract stays as it was.
- A look that switches the stylisation (a Stylise pass on or off, the
  pixel grid, retro 3D, the material style) starts TAA's history over,
  as a camera cut does.
- Call og.render.applyQuality when the player picks a preset on the
  settings page, and apply a saved preset at start.

The quality rows:

| Field | low | medium | high |
|---|---|---|---|
| rectShadows | false | true | true |
| shadowedLights | 2 | 8 | 16 |
| shadowCascades | 2 | 3 | 4 |
| shadowMapScale | 0.5 | 1 | 2 |
| volumetricFog | false | true | true |
| softParticles | false | true | true |
| screenSpaceReflections | false | true | true |
| ambientOcclusion | false | true | true |
| aoResolution | half | postEffects | full |
| ssrStepScale | 0.5 | 1 | 2 |
| volumetricStepScale | 0.5 | 1 | 2 |
| msaaSamples | 1 | 4 | 4 |
| upscaler | taau | taau | fsr3 |

## 17. Display settings and automatic quality

- Keep the display settings with the player's settings, not the world.
  Save og.render.display(world).settings with og.platform.saveSettings,
  and give it back to og.render.setDisplay at the next start.
- A native run (`og run`, `og test`, reference captures) draws every
  frame at render scale 1.
- The browser decides some settings, and a page reads them from
  DisplayInfo. Vsync is always on, because a browser presents on the
  display's refresh. Fullscreen follows the page: Escape leaves it and
  turns the setting off. A power preference applies at the next load
  (restartNeeded).
- Frame cap `auto` is 60 on a phone or tablet, which halves the GPU's
  work on a 120 Hz display for a game ticking at 60 Hz or less.
- Automatic quality measures the GPU's time where the device times it
  (RenderCapabilities.gpuTimestamps), otherwise frames that miss the
  display's interval. It tries a step up after a calm while and gives
  up after three failures.
- Under TAA a render scale below 1 is upscaled temporally, which
  recovers most of the detail over a few frames.
- Cloth subdivision: devices without compute and vertex storage keep
  the coarse mesh with CPU normals. A change of the effective level
  resets surface motion history.

## 18. Render capabilities

RenderCapabilities.stylise names these groups:

| Resource | Groups |
|---|---|
| Stylise | pixel, posterize, palette, dither, halftone, hatching, paint, watercolor, ink, inkDepth, depthBands, retro3d, surfaceSpace, objectSpace, paintDetail |
| MaterialStyle | toon, shadowTint, shadowDarkness, rim, outline, outlineDistance, flatShading, surface, toonAmbient, outlinePlace, wireframe |

- The stylise groups map to fields: `watercolor` is the watercolor*
  fields; `surfaceSpace` and `objectSpace` are the `surface` and `object`
  values of paintCanvasSpace, hatchingSpace and halftoneSpace; `retro3d`
  is vertexSnap, affineTexturing and depthBits. A look that uses a
  missing primitive still applies.
- maxMsaaSamples is also 1 where antialias resolved to fxaa because the
  HDR format cannot be multisampled.
- spriteAnisotropy: text is not filtered anisotropically, because its
  distance field would lose its edge. A software rasterizer reports 1,
  because its anisotropic filter softens even an image drawn 1:1.
  Above 1, a style's nine-slice middle squeezed into a short button
  keeps its outline as crisp as the corners. At 1 the squeezed piece
  reads a smaller mip level; og.ui bounds a middle's squeeze at twice
  its art, so its outline is one mip level softer than the corners.
- gpuCrowds needs indirect draws with a first instance (WebGPU's
  `indirect-first-instance`, Vulkan, Metal, D3D12).
- visibilityBuffer: Chrome on the Pixel 10's PowerVR D-Series refuses
  the resolve pipelines with VK_ERROR_UNKNOWN. The fallback draws the
  same surfaces at a higher vertex cost and the log says so once.
- visualSimulation is true on every WebGPU, Metal, Vulkan and DX12
  device.
- world3d is false in the 2D profile's publish and lanes modules. Their
  publish build refuses a 2D project whose scenes hold og.Terrain,
  og.TerrainScatter, og.Water, og.WaterSurface or og.Fluid3D (§4).
