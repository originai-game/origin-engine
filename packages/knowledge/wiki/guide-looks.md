<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `looks` guide: Rendering and looks

Rendering an Origin Engine game. The look as a preset the game extends, what belongs in the look and what in the scene, lighting and post order, 2D versus 3D, and what a frame costs.

The API: `og api show render`, `og api show atmosphere`,
`og api show terrain`, `og api show video`, `og api show camera`,
`og api show shapes`. A look's fields are the reflected resources
(RenderSettings, PostEffects, Stylise, Environment, Fog, Sky and the Art
blocks); `og api show <Resource>` gives each field with its range.

<a id="guide-looks-1"></a>
### `looks §1` The look

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
ui "fantasy"
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

<a id="guide-looks-2"></a>
### `looks §2` What goes where

- The look: grading, tone mapping, exposure, fog, sky, the stylise pass
  (pixelation, posterize, dither, outlines) and the art contract.
- The scene: lights, their shadows, reflection probes, light volumes,
  baked lightmaps, particles and decals.
- Never imitate a look's effect with geometry or textures (a painted-on
  outline, a pre-dithered texture): the stylise pass does it once,
  consistently, at every resolution.

<a id="guide-looks-3"></a>
### `looks §3` Light

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

<a id="guide-looks-4"></a>
### `looks §4` 2D and 3D

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

<a id="guide-looks-5"></a>
### `looks §5` Cost

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
