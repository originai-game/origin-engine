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
