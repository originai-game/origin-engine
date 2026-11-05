# Moods

A look's `mood "<word>"` (docs/art.md §2.9). A preset's own `mood` line names
the mood it is drawn in. A game's look that asks for another mood has the
mood's layer laid over the preset its `extends` chain reaches, before the
game's own fields:

- `<mood>.ogt`: the layer for every style, written as the mood would read over
  a neutral look. A number moves the preset's value by its distance from the
  field's default (`exposure 0.72` takes a preset at 0.8 to 0.52, clamped to
  the field's range); any other value (a bool, a variant, a colour, a palette)
  replaces the preset's.
- `<mood>/<slug>.ogt`: one style's layer, used for that style in place of the
  generic one: the palette-bound styles swap their palette for the mood's.

A layer sets the frame's grading, light and palette only: `PostEffects`,
`RenderSettings`, `Environment`, `Fog`, `Sky`, `Stylise.palette` and
`ArtPalette.colors`. It never changes the art contract's sizes, shapes, lines
or motion. The cook compiles every file here in (`og-cook`'s `MOOD_LAYERS`),
and the art brief reads the palette a layer names.
