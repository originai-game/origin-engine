# Tiled motion blur

`crates/og-render/shaders/motion_blur.wgsl` implements TileMax and
NeighborMax from McGuire, Hennessy, Bukowski and Osman, *A Reconstruction
Filter for Plausible Motion Blur*, I3D 2012, equations 1–2:
https://casual-effects.com/research/McGuire2012Blur/McGuire12Blur.pdf

The separable tile reduction ports the mechanism of Wojtek Pe's
`addons/per_object_motion_blur/tilemax_2step.glsl`, MIT, at revision
`8fdc341062fb9592ea92402e4f3e1951d374204a`:
https://github.com/wojtekpil/godot_per_object_motion_blur
The licence is retained in `LICENSE`. This port uses separate horizontal
and vertical targets instead of transposing, float pixel velocities instead
of rg8 UV velocities, and Origin's explicit shutter interval and cap.
No Godot sentinel, frame-rate convention or default is copied.

The reconstruction retains Origin's authored tap count and centred shutter
integral. NeighborMax selects a sampling direction; each source still has
to cover the receiving pixel with its own swept segment and pass the depth
test. A fast tile never directly assigns its velocity to an unrelated
stationary marker. Like McGuire's algorithm, a dominant-direction gather
cannot represent every crossing direction within one tile. This is a
screen-space reconstruction, not visibility-aware temporal ray tracing.

Jimenez, *Next Generation Post Processing in Call of Duty: Advanced Warfare*,
SIGGRAPH 2014, is the additional published scatter-as-gather reference:
https://www.iryoku.com/next-generation-post-processing-in-call-of-duty-advanced-warfare/
No Unity Companion License implementation is incorporated.
