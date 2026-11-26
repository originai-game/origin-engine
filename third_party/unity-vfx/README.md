# Unity Labs VFX sequences

Thomas Iché / Unity Labs Paris released these Houdini-simulated sequences
under CC0 1.0, including commercial use and redistribution:
https://unity.com/blog/engine-platform/free-vfx-image-sequences-flipbooks
(retrieved 2026-11-26).

Original downloads:

- https://unity3d.com/files/labs/downloads/vfx/assets01/Explosion02/Explosion02-flipbooks.zip
- https://unity3d.com/files/labs/downloads/vfx/assets01/DiscSmoke01/DiscSmoke01-flipbooks.zip

The cinematic-realism downloadable pack uses PNG RGBA conversions.
Explosion02_5x5 uses the linear HDR EXR, not the highlight-clipped TGA.
Clamp negative RGB to zero, divide RGB by 1 + max(R, G, B), then encode
sRGB; alpha is unchanged. This hue-preserving range compression keeps
fire detail that a direct 8-bit conversion clips. The image is resampled
from 1024 square to 1000 square so each of its 25 cells occupies an
integral 200 pixels. DiscSmoke01_16x4 uses the TGA and is
resampled from 2048x1024 to 1024x512, preserving its 16x4 layout. Each cell
gets an elliptical alpha feather, normalized by its half-extent minus
2 pixels: alpha times 1 - smoothstep(0.65, 1, radius). The transparent
2-pixel gutter and soft rim remove the source's rectangular boundary when
the plume is retimed as ground dust; its RGB is unchanged. No EXR or
unused source frames are shipped. Source archive hashes and conversion
details are recorded in the pack's assets.json.

The native effect compositions are original MIT work, Origin Engine.
