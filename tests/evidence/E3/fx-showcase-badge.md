# fx-showcase references after the text-wrap fix (00653bf)

00653bf ("a fit text no longer wraps over the half pixel its snapped width
lost") puts the bottom-right `og.ui` badge of `templates/fx-showcase` on one
line ("og.ui over the frame") where it wrapped onto two ("og.ui over the" /
"frame"). Every station then failed its references on metal, webgpu and
webgl2 (and, untested here, vulkan-cpu):

| Station | outliers | RMSE (whole frame) | changed pixels | RMSE outside x 1100–1271, y 640–699 |
|---|---|---|---|---|
| post | 0.428 % | 0.02858 | 4299 | 0 |
| lens | 0.418 % | 0.03875 | 4305 | 0 |
| fx | 0.125 % | 0.02155 | 4108 | 0 |
| 2d | 0.347 % | 0.02187 | 4132 | 0 |
| stylised | 0.418 % | 0.03820 | 4307 | 0 |

Metal (`og test templates/fx-showcase`, then a numpy compare of
`og-out/test/frames/<station>.png` with `references/metal/`). Every changed
pixel lies in x 1109–1263, y 648–693, the badge; outside it the largest
channel difference is 0. webgpu and webgl2 (`bun tests/web/reference.ts
[--webgl]`, the frames in `dist/test-reference/out/<tier>/`) give the same
table to the fifth decimal (webgl2 post 0.02859) and the same box.

`fx-showcase-badge-diff.png`: per station (rows post, lens, fx, 2d,
stylised) the metal badge region x 1100–1271, y 640–699 at 2×: the old
reference, the new frame, and the mask of changed pixels.

Re-recorded: `references/metal` (`OG_FX_CAPTURE=1 og test
templates/fx-showcase`), `references/webgpu` and `references/webgl2`
(`bun tests/web/reference.ts --record [--webgl]`, Chrome for Testing 150).
Afterwards `og test templates/fx-showcase` passes at RMSE 0 and `make
web-reference` passes both tiers. Owed: `references/vulkan-cpu` still holds
the two-line badge; it is recorded on lavapipe in the platform's E2B sandbox
(as the archetypes' were, bcbf941), which this machine does not run.
