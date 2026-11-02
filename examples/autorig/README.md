# Auto-rigged characters

Characters rigged by the cook from layered art, with no editor step:
`assets/chars/knight.ogt` names `knight.psd` and the `humanoid-side`
template, and the cook makes the bones, slots, regions, leg IK and atlas
(`crates/og-cook/src/cook/autorig.rs`). The scene shows the setup pose,
which draws the art as it was drawn, and the same rig with its feet moved
by `og.IkTarget2D`.

`art/make.sh` remakes `knight.psd` from `art/knight.py` (one SVG group per
part, rendered by headless Chrome, stacked into layers by ImageMagick).

The art is made here for the engine and is CC0 1.0.
