# The depth prepass on every frame: measured, kept on demand

docs/design.md §2.6 said "Forward+ (clustered) with a depth prepass";
og-render runs its depth-normal prepass only when an effect reads it (AO,
outlines, depth of field, motion blur, god rays, soft particles, water, 3D
fluids). The prepass draws every opaque instance into its own depth and
G-buffer targets (`pre-depth`, `gbuffer`, `lindepth`, `outline-id`); the
lit pass keeps its own depth test, so forcing it adds a geometry pass and
changes no pixel of the lit frame.

How: og-gfx (release, Metal, Apple M4 Pro, 1280x720) with the prepass forced
on by a local switch (not committed), five interleaved rounds of 200 frames
per scene and mode, load average 20 to 39 (`prepass-rounds.txt`). `frame` is
og-gfx's frame time including the GPU wait.

| scene | off best ms | on best ms | off median | on median |
|---|---|---|---|---|
| spheres-20k | 9.98 | 16.56 (+66 %) | 10.89 | 18.52 |
| showcase | 5.23 | 4.81 | 6.18 | 6.63 |
| city-street | 1.60 | 1.92 (+20 %) | 4.72 | 3.85 |
| city-aerial | 3.00 | 3.47 (+15 %) | 9.49 | 9.68 |

The medians are load; the best runs show the cost: a second pass over 20,000
spheres' geometry costs two thirds of the field's frame, and the city's
40,000 instances pay 0.3 to 0.5 ms. Nothing in the frame reads the prepass's
depth when no effect asks for it (the lit pass is forward with its own
depth test, and Forward+ assigns lights to clusters on the CPU from light
volumes, not from scene depth), so it stays on demand: design §2.6 now says
so, and og-render's README closes the owed line with these numbers.
