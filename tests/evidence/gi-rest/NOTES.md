# The rest of area 1: colour bleed, Radiance Cascades, visibility (2026-11-05)

Branch `gi-rest` from main 6f27e2201, measured on macrunner (M4 Pro,
Metal). All runs held `og_gpu_acquire`. The timings held every slot.

## Method

A scratch og-tests test (not kept) rendered gi-room-probes and read back
the lit image's indirect diffuse divided by the albedo, per pixel. It
compared that with a CPU path tracer over the same geometry: sky black,
four bounces, the sun and the scene's materials. It also built an "ideal
DDGI": the same probe placement and trilinear interpolation, but with
exact L1 harmonics and binary visibility. That ideal shows what the
structure can reach once its estimators have converged.

## 1. Colour bleed in gi-room

**Root cause: the cascades were centred on the camera.** gi-room's camera
stands 8 m in front of the back wall. Cascade 0 (16 × 8 × 16 probes,
0.75 m apart) spans 12 m along z, so half of it lay behind the camera.
The back wall, and the floor beside the red wall, fell into cascade 0's
fade band or into cascade 1, whose probes are 1.5 m apart. At 1.5 m the
probes next to the red wall average its light with the room's centre, so
the tint on the floor and the white walls beside it is diluted.

**Fix (`gi_probes.rs` `plan`).** Each cascade is now centred `AHEAD` =
0.3125 of its extent ahead of the eye, along the camera's view direction.
That is five of sixteen cells looking along an axis. The camera stays
three to four cells inside the face behind it, clear of the two-cell
fade. The cascade reaches nine cells ahead at full weight. The grid is
still on the world's cells of its spacing, so turning or moving through a
cell's width scrolls one plane of probes, and the rest keep their state.
0.25 left the back wall in the fade. 0.375 put the eye in the fade.

| Back wall within 1 m of the red wall | Indirect red / reference |
|---|---|
| Before (camera-centred) | 0.50–0.65 |
| After (ahead of the camera) | 0.70–0.85 |

The new unit test `the_finest_cascade_covers_what_the_camera_looks_at`
fails on the old code: with the old centre, the far points weigh below
0.9 in cascade 0.

**What remains is convergence, not structure.** Summed indirect over the
image, GPU against the path-traced reference (r / g / b):

| Frames | GPU / reference |
|---|---|
| 32 (the reference frame) | 0.81 / 0.89 / 0.94 |
| 128 | 0.87 |
| 400 | 0.90 |
| Ideal DDGI (converged) | 0.96 / 1.00 / 1.02 |

The gap left is the multiscale mean estimator's catch-up floor (1/256 in
`gi_blend.wgsl`). Raising it to 1/32 moved the 128-frame ratio from 0.88
to 0.90, too small a gain for a faster but noisier estimator, so it was
not kept. Two other
experiments were not kept either: reading the bounce from extra
short-mean harmonics (a small gain for 79 more vec4s a probe), and
stepping inside probes out through the nearest back face at once.

gi-room-probes is open to a bright sky, so the sky's light dominates the
white walls. The red bleed shows as a pink cast on the back wall's left
half and in the shade by the cube. That matches the reference's
strength. It is weaker than a closed room's bleed, because the scene is
lit differently, not because the GI is missing it.

**gi-shelter at 8 frames** shows dark blotches along the back wall's base,
which are gone by frame 32. The same blotches appear on main 6f27e2201,
so the cascade change did not cause them. They are probes that start
classed as inside the wall, then relocate within a few updates.

### Screen GI (not changed; for whoever owns it next)

- The screen-space GI (`gi-room`, `gi-room-taa`) delivers about 0.40 of
  the reference's indirect light. It sees only on-screen geometry, and
  many pixels' rays leave the screen.
- `method: both` sums to 1.12–1.20 of the reference. The screen bounce
  appears to add to the probes' bounce for the same light, rather than
  replace it where it hits. The fix is to let screen hits replace the
  probe term over the share of rays that hit, instead of adding to it.

## 2. Radiance Cascades against the probes

**Not ported.** RC does not win clearly anywhere that matters here.

### Quality: a small emitter (CPU, one bounce)

Scene: a 0.2 m emissive cube in a dark 4 × 3 × 4 m room with an occluding
slab. The error is the mean |relative error| of irradiance on the room's
surfaces against a brute-force reference.

| Method | Error |
|---|---|
| 3D world-space RC (c0 0.25 m, 4 × 4 octahedral directions with 16 sub-rays each, intervals ×4, 4 cascades) | 0.61 |
| DDGI, 0.75 m probes | 0.72 |
| DDGI, 0.25 m probes | 0.60 |

RC's coarse first cascade, 16 directions, still misses a 0.2 m emitter
near the source. Away from it RC is no better than DDGI at the same
spacing. Its advantage in flatland (a linear cost in resolution for
penumbra-correct light) does not carry over to 3D world space without
far more directions in c0.

### Cost

- In a BVH a short interval still costs 57% of a full ray
  (`tests/evidence/gi-area/NOTES.md`), so RC's interval split saves
  little. World-space RC with c0 at 0.25 m traces about 6× the rays of
  the probes.
- Split Radiance Cascades (Freeman, Sannikov and Pitt-Francis, arXiv
  2607.20384), the newest 3D variant, reports 8.6–11.5 ms on an RTX 3080
  Laptop with OptiX hardware rays and a sparse hashmap built with atomics.
  It still leaks below its probe spacing. WebGPU has neither hardware
  rays nor cheap 64-bit atomics, so the engine would pay several times
  that.
- Screen-space RC (radiance-cascades-urp) reaches no further than the
  screen GI, which already misses off-screen light (above).

### Stability

Probe GI flickers by 0.00004–0.00007 mean |ΔE| per frame (linear), at
rest. RC was not built on the GPU, so its behaviour under TAA, TAAU or
FSR was not measured; the CPU results above already ruled it out.

### Implementations surveyed

| Implementation | Licence | Kind |
|---|---|---|
| three-rc | none (not usable) | 2D/2.5D |
| jumeder/dx12-radiance-cascades | MIT | coarse 3D world |
| radiance-cascades-urp | MIT | screen space |

### What would change the verdict

- Hardware ray tracing on WebGPU, or a Lumen-style global SDF, where an
  interval's cost does grow with its length.
- A 2D lighting path: og-render's 2D lights are the natural home for
  flatland RC, which is a separate feature area.

## Metal timings at 1080p

Frames 20–60, every GPU slot held.

| Configuration | Pass times | Frame total |
|---|---|---|
| No GI | — | 1.55–2.2 ms |
| Probes (gi-room-probes: 0.75 m, 2 cascades, 64 rays) | gi-probes 1.73 ms, main 7.98 ms | 9.8 ms |
| Probes, 256 rays | gi-probes 5.39 ms | 13.45 ms |
| Probes, 1 cascade | gi-probes 1.09 ms, main 6.25 ms | — |
| Screen GI (gi-room) | ao 10.2 ms | 13.2 ms |
| Both | gi-probes 9.19 ms | 20.85 ms |
| gi-shelter | gi-probes 4.59 ms | — |

The lit pass's probe sampling costs about 6 ms at 1080p: 8 probes a
cascade, each with a moments fetch from the storage buffer. That is the
largest share of the probe GI's cost, and the place to optimise next
(for example, a half-resolution irradiance pass upsampled by depth, or
the probes' irradiance baked into a 3D texture once a texture slot is
free).

## 3. DDGI visibility and leaks

**Complete; no change needed.** Scene: a closed box room on a sunlit
plane, sky black, probes 1 m apart, walls of three thicknesses.

| Walls 0.2, 0.1 and 0.05 m thick | Inside mean E/π | Inside max E/π |
|---|---|---|
| Range over the three | 0.0005–0.0009 | 0.011–0.021 |

The sun outside gives 0.76. The leak is under 0.1% on average and under
3% at the worst pixel, even for walls a twentieth of the probe spacing.
The port already has DDGI's full visibility: Chebyshev moments, the
normal and view bias, back-face relocation and inside-probe deactivation.

**RTXGI's licence.** RTXGI-DDGI is under the NVIDIA RTX SDK licence, not
MIT (checked on GitHub, 2026-11-05), so it can be cited but not ported.
The engine's port stays Wicked Engine's (MIT, `third_party/wicked-ddgi`).
It has the same visibility parts RTXGI documents (moments, biases,
relocation, classification), and the measurement above shows they work.

## Tests

- `the_finest_cascade_covers_what_the_camera_looks_at` (og-render unit):
  fails on the old camera-centred plan, passes on the new one.
- `gi_room_probes_matches_its_reference` and
  `gi_shelter_matches_its_reference`: re-recorded on Metal.
- `probe_gi_bleeds_each_walls_colour` and
  `probe_gi_occludes_the_sky_under_a_roof`: pass.
