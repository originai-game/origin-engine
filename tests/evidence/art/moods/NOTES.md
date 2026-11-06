# A4: the look's mood, first sample on Metal

Branch `look-catalogue`, drawn on macrunner (Metal) with this branch's dev
`og` and addon, by `tools/art-samples/capture.ts --only
faceted-low-poly,monochrome-pixel,muted-cinematic-3d --mood
own,bright,dark,warm,cold,soft,high-contrast` at 1280x720.
`sample-sheet.jpg` has one row per style and one column per mood. `og style
check` passed on all 21 frames.

Read:

- faceted-low-poly (drawn soft): each mood is distinct and still flat-shaded
  low poly. Dark closes in with the vignette, warm and cold shift the white
  balance, high-contrast deepens the shadows.
- monochrome-pixel (drawn soft, its palette swapped per mood): stays four
  tones and one ink. High-contrast shows a visible Bayer dot grid across the
  flat sky, because its two light tones are far apart. This needs the art
  runner's eye; the fix would be its palette (`@palettes/handheld4-high-contrast`).
- muted-cinematic-3d (drawn dark): warm, cold and high-contrast read well.
  Bright and soft move it only a little: the layer moves the preset's own
  values, and this preset starts deep in the dark.

The full 37 x 7 sweep and its judgement are the art runner's.

## Polish (look-mood-polish), the art runner's five notes

`polish-before.jpg` is the art runner's own 37 × 7 render, cut to the five
styles it flagged; `polish-after.jpg` is the same five, rendered on Metal
with `tools/art-samples/capture.ts --mood own,bright,dark,warm,cold,soft,high-contrast`.

- minimal-geometric: the ground's hue flips were not the mood layer's
  doing. An exposure-only layer flipped it too, because per-channel
  posterize rounds each display channel at its own threshold. The sample
  ground is a weak grey-green (≈140,165,140): at exposure 1 all three
  channels snap to 153 (grey); at 0.7, R and B fall to 102 while G holds,
  so it turns green. `Stylise.posterizeMode tone` (value and linear tint
  stepped apart) holds the hue; the og-tests pixel test
  `tone_posterize_keeps_a_colours_hue_as_its_light_moves` shows the same
  field grey at three lights and green at two per channel, with one tint
  at all five in tone mode. Its own frame moves: the ground now reads
  its grey-green in every mood. Its dark and high-contrast drop the
  vignette, whose falloff stepped into a huge ellipse and rings; bright
  drops the saturation and vibrance that pushed it a tint step mintier.
- bold-comic-illustration, dark: the rainbow band was the vignette and
  the shadow balance through per-channel posterize, one ring per channel.
  Its dark layer leaves both out. It keeps the default posterize, since
  tone mode would have changed its own sky (lavender to pale ice) and hills.
- heavy-ink-shadows, bright and soft: the pastel hills were the same
  channel flip on drained red-brown greys; tone mode holds them at one
  muted hue in every mood. No layer of its own.
- monochrome-pixel, high-contrast: `@palettes/handheld4-high-contrast`'s
  light greens drawn closer (9cc24a, f4fbe0 to c6de7c, eaf4c4): the
  flat sky's dither alternates near neighbours, no dot grid.
- muted-cinematic-3d: its own bright (exposure to 1.2, a paler #a4a4aa
  fog, a little warmth and colour) and soft (contrast -0.32, exposure
  0.9, ambient 0.9, a paler #8c8c92 fog) now read clearly apart from its own.
