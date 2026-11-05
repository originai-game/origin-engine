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

# Second head: `both`, the screen GI's reach, the lit pass's cost, relocated probes

Measured on macrunner (M4 Pro, Metal), from main 574f1a091. The method
is the first head's: the gi-room geometry path-traced on the CPU (sky
black, four bounces) against the GPU's indirect light, summed over the
image (r / g / b). "Exact probes" is the same CPU model's DDGI with
probes path-traced to convergence.

## 1. `method: both` counted the bounce twice

| | Before | After |
|---|---|---|
| Probes alone | 0.85 / 0.93 / 0.97 | 0.85 / 0.93 / 0.97 |
| Screen GI and probes (`both`) | 1.11 / 1.18 / 1.21 | 0.87 / 0.94 / 0.97 |

**Cause.** The lit pass multiplied the probes' light by the screen rays'
ambient occlusion: the rays that hit took an even share of the probes'
light from every direction. The walls the rays hit are brighter than the
sky they miss, so too little was taken and the screen's light of those
walls came on top. With exact probes the CPU model gives 1.19 / 1.22 /
1.22 for that pairing: the error is in the pairing, not the probes.

**Fix (ssgi.wgsl, gi_sample.wgsl).** The screen GI's pass gathers the
probes once and takes their light along the hits' directions: with E/π
along n c0 + c1·n, the radiance is L1, so the hits' share over π is
h c0 + 1.5 c1·s (h the hits' weight, s their weighted mean direction).
The bounce carries the difference from what the lit pass removes.
L1 is too smooth to say how much light came from far hits:

| Screen ray reach (exact probes, L1 share) | Sum / reference |
|---|---|
| 4 m | 1.09 / 1.13 / 1.13 |
| 2.5 m | 1.04 / 1.08 / 1.07 |
| 1.5 m | 1.00 / 1.03 / 1.03 |
| 1.0 m | 0.98 / 1.02 / 1.02 |
| 0.5 m | 0.97 / 1.01 / 1.02 |
| Probes alone | 0.96 / 1.02 / 1.02 |

So with the probes on, the screen rays reach one spacing of the finest
cascade: the near field the probes cannot resolve. Rays whose light the
screen cannot give (no history, a hit that was off screen) count as
misses, so the probes give that direction.

`screen_and_probe_gi_count_the_bounce_once`: the light both add over
no GI against the probes' alone, 1.345 on the old code, 1.035 now.

## 2. The screen GI alone gathered 0.40

| Screen rays can see (CPU) | Share of the reference |
|---|---|
| Every on-screen hit | 0.80 / 0.74 / 0.72 |
| Within the old 4 m (faded from 2.4 m) | 0.49 / 0.46 / 0.45 |

| GPU, gi-room | Share of the reference |
|---|---|
| Old: 4 m, 12 world-space steps | 0.39 / 0.41 / 0.42 |
| Old march, 16 m, 12 steps | 0.60 / 0.61 / 0.64 |
| Old march, 16 m, 32 steps | 0.67 / 0.67 / 0.67 |
| New march, 16 m, 12 steps (the new default) | 0.65 / 0.65 / 0.66 |
| New march, 16 m, 32 steps | 0.71 / 0.71 / 0.72 |

**Cause.** Not the bounces, the thickness or the normalisation: the
rays marched a fixed 4 m on world-space steps, so they never reached
the far wall of a 6 m room, and lengthening them spent the steps off
screen. **Fix.** Each ray is cut where it leaves the view frustum (the
projected ray is linear before the divide, so each side plane cuts it
at one t) and its steps are spread over what it crosses on screen,
perspective-correct and finer near the pixel. `maxDistance` defaults
to 16 m (it was 4): with the cut it costs 0.4 ms more at 1080p, not a
step more. What remains (0.65 against 0.72 to 0.80) is the step count
and the off-screen share no screen method sees.

`screen_gi_reaches_across_the_room`: the screen GI's light against the
probes', 0.521 on the old code, 0.796 now.

## 3. The lit pass's probe sampling

Metal, 1080p, frames 20 to 60, every GPU slot held. "Before" is main
574f1a091 measured in the same session.

| Pass (scene) | Before | After |
|---|---|---|
| main, no GI (gi-room-probes) | 1.46–2.04 ms | 1.41–2.08 ms |
| main (gi-room-probes) | 7.98 ms | 3.70 ms |
| gi-probes (gi-room-probes) | 1.73 ms | 1.64 ms |
| frame (gi-room-probes) | 9.80 ms | 5.43 ms |
| ao, the screen GI (gi-room) | 10.20 ms | 11.51 ms |
| frame (gi-room) | 13.27 ms | 14.56 ms |
| ao (`both`) | 2.26 ms | 5.74 ms |
| main (`both`) | 8.39 ms | 4.38 ms |
| gi-probes (`both`) | 9.28 ms | 1.72 ms |
| frame (`both`) | 20.86 ms | 12.70 ms |
| gi-probes (gi-shelter) | 4.61 ms | 4.49 ms |
| main (gi-shelter) | 7.74 ms | 4.02 ms |

Under `both` the old frame's pass timers read 2.26 ms for the ao pass
(10.20 ms alone) and 9.28 ms for gi-probes (1.73 ms alone): how the two
were split is not clear, so compare the frame totals. The screen GI
alone costs 1.3 ms more (gi-room's ao pass): 0.9 ms for the new march
(the frustum cut, the perspective-correct steps) and 0.4 ms for the
16 m cap, on the same 12 steps. Under `both` its rays reach one probe
spacing and its pass gathers the probes once.

Where the main pass's time went, on the way (gi-room-probes):

| Change | main |
|---|---|
| Before | 7.98 ms |
| No reflection-occlusion gather (experiment) | 4.81 ms |
| No visibility test (experiment) | 7.10 ms |
| One gather shared by the diffuse light and the reflections' occlusion | 5.17 ms |
| And the ring buffer's slot and key worked out once, not per corner | 3.70 ms |

**Changes (gi_sample.wgsl, gi_probes.wgsl, frame.wgsl).** A gather
returns the 8 probes' weighted harmonics (12 floats), linear in the
direction; the lit pass evaluates them along the normal and along the
reflected ray. The reflections' occlusion now uses the visibility-tested
probes, where it had its own unweighted gather. The corners' slot and
key came from a vector modulo and division per corner; they are the base
corner's plus one, wrapping. The trace's bounce reads the same gather.

## 4. Dark spots on gi-shelter's back wall (the art runner's note)

gi-shelter's 1 m probes lie on its back wall's and roof's planes. The
first trace finds some just inside the slabs (back faces on more than a
quarter of their rays), and the relocation moves them out at once; at
frame 32 every one of them was active and 0.13 of a spacing off the
surfaces (read back from the probe buffer). But their radiance estimator
and distance moments kept what they saw inside, black, and the
estimator's catch-up is slow, so each left a dark spot on the wall for
dozens of frames: with the classification off the spots went, with the
relocation off they stayed, with the spacing at 0.97 m (no probe on a
plane) they went.

**Fix (gi_blend.wgsl).** A probe whose last update found it inside
starts its light and distances again from this update's rays, as a fresh
probe does.

`probes_moved_out_of_a_wall_light_it_at_once`: the deepest spot on the
back wall's middle at frame 8 against the ring 24 px around it, 16.43
levels on the old code, 2.71 now (a settled frame: 2.6).
