# Animation sample compression and sparse morph targets (P4, cook-anim)

Apple M4 Pro, macOS, 2026-10-20, dev profile.

`#anim` blob sizes of every committed animated model (fx-showcase's
`assets/`; `templates/game`, `templates/archetypes/third-person`,
`crates/og-tests/assets` and `crates/og-tests/reference-scene` carry
byte-identical copies: walker, trickster and strider have one MD5 each).

- **Before**: `og-cook` at 5dc3aa0 (cooker r3), f32 samples, dense morph
  deltas.
- **f32**: this branch's default: constant channels stored once (lossless),
  sparse morph targets (lossless), f32 samples otherwise.
- **compressed**: `animation { compress true }` at the default tolerance
  1e-4 (`anim-compress.json`, written by
  `crates/og-cook/tests/anim_compress.rs`).

| Model | Before | f32 | compressed | `samples` (+ `packed`) before → f32 → compressed | morph data before → after |
|---|---:|---:|---:|---:|---:|
| models/walker.glb | 9,002 | 9,051 | 6,203 | 4,892 → 4,892 → 2,028 | 395 → 156 |
| models/trickster.glb | 121,879 | 106,967 | 99,479 | 25,200 → 9,504 → 2,012 | — |
| models/strider.glb | 21,285 | 20,501 | 10,293 | 15,940 → 14,660 → 4,448 | — |
| models/stout.glb | 4,136 | 4,200 | 4,200 | — | — |
| models/puppet2d.glb | 8,427 | 8,603 | 5,899 | 4,836 → 4,836 → 2,132 | — |
| anims/humanoid-clips.glb | 14,736 | 12,576 | 7,648 | 11,636 → 9,076 → 4,150 | — |
| **total** | **179,465** | **161,898** | **133,722** | **62,504 → 42,968 → 14,770** | |

The f32 blobs of models with no constant channel grow by the channel rows'
three new fields (12 B a channel). The trickster's blob is 93,149 B of skin
stream (joints and unorm16 weights per vertex, meshopt), untouched.

Errors of the compressed cook against the f32 one (every frame, every
channel; model space at every frame and halfway between):

| Model | Largest lane error | Largest node error | Skeleton extent |
|---|---:|---:|---:|
| walker | 9.28e-5 | 0.137 mm | 1.70 m |
| trickster | 9.97e-5 | 0.141 mm | 0.80 m |
| strider | 7.62e-5 | 0.149 mm | 1.92 m |
| puppet2d | 2.16e-5 | 0.074 mm | 1.55 m |
| humanoid-clips | 8.94e-5 | 0.180 mm | 1.69 m |

## The frame

`anim-compress-f32.png` and `anim-compress-quantized.png`: the cooked
trickster, scaled to two metres and filling the frame, skinned on the GPU
and posed by its `die` clip at 0.5 s (mid-fall: leaning back, the left
arm raised to the face, the hat's tip swung back), lit by og-tests'
gltf-reference lights, on Metal. The two are the same picture:
`anim-compress.diff.png` (differences x4, red past 32 levels) is black but
for single pixels along the silhouette, the dice and the hat's rim
(RMSE 0.0028, 73 of 921,600 pixels past 32 levels). og-tests' default frame tolerance
is RMSE 0.01 and 0.5 % of the pixels.

Because compression is opt-in, every committed reference frame (og-tests'
gfx scenes, the templates' playtest frames) is cooked from f32 samples
exactly as before, and the two lossless encodings rebuild the same floats
bit for bit by construction: a constant channel is one whose frames all
have the same bits, copied back to every frame; a vertex a morph target
does not list had a delta of all-zero bits, and is filled with 0.0.
