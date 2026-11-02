# Auto-rig fixtures

`make.sh` draws them with ImageMagick 7: a 100 x 200 side-view stick
figure facing right, one canvas-sized PNG per part of the humanoid-side
template in `stick/` (`NN-<part>.png`, back to front), and the same parts
as the labelled layers of `stick.psd` (ImageMagick's PSD writer, RLE).
`tests/autorig.rs` derives its expected joints and rectangles from the
shapes in `make.sh`.

Made here for the engine; CC0 1.0 (public domain dedication).
