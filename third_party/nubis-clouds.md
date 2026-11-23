# Volumetric cloudscapes

`crates/og-render/shaders/clouds_noise.wgsl`, `clouds_shape.wgsl`,
`cloud_shadow_bake.wgsl`, `sky.wgsl` (`clouds_march`, `cloud_sunlit`,
`cloud_skylit`, `fs_clouds_resolve`) and `media.wgsl` `draine_phase` follow
published descriptions of techniques. No code and no data is copied: the
WGSL, the noise recipes and the baked fields are Origin's own.

## Shape

Andrew Schneider, *Nubis, Evolved: Real-Time Volumetric Clouds for Skies,
Environments, and VFX*, SIGGRAPH 2022, and *Nubis³: Methods (and madness) to
model and render immersive real-time voxel-based clouds*, SIGGRAPH 2023
(both Advances in Real-Time Rendering).

- 2022: https://advances.realtimerendering.com/s2022/SIGGRAPH2022-Advances-NubisEvolved-NoVideos.pdf
- 2023: https://advances.realtimerendering.com/s2023/Nubis%20Cubed%20(Advances%202023).pdf

What Origin takes from them:

- the **dimensional profile**: a low-frequency field that is 0 at a cloud's
  skin and rises inward (2022), a gradient from the outside of a cloud to
  its inside as a signed distance field's is (2023): here that of a round
  envelope over a 2D field of where clouds stand, so a cloud narrows as it
  rises and its sides lean in to a dome (`cloud_profile`);
- the **Perlin-Worley** shape noise (2015 and 2022): octaves of inverted
  Worley noise cut by a low-frequency gradient noise, and clouds of more
  than one size from a field of two frequencies;
- the **up-rez**: the profile eroded by tiling noise as a remap,
  `saturate((profile - noise) / (1 - noise))` (2023), wispy noise at the
  base and billowy noise above it (2022);
- the profile used again for light: less of the light scattered more than
  once reaches the eye from a cloud's skin than from its body (2022, the
  in-scatter probability that replaced the 2015 "powder" term);
- the march's steps lengthening with the distance (2022), out to where the
  air has taken the clouds: the far clouds are the same volume read more
  coarsely, with no layer standing in for them; and the two near light
  samples followed by a precomputed summed density (2023, there a voxel
  grid).

What differs: the 2D fields are procedural and baked on the GPU, not
authored; a type's shape is ten numbers per `CloudKind`, not a lookup
texture or a modelled voxel field; the noise is Origin's own recipe of
Worley's F1 and gradient noise in two volumes, not Guerrilla's curly
alligator noise; the summed density towards the sun is a 2D Beer shadow
map (below), not a voxel grid.

Guerrilla's *Nubis Voxel Clouds Pack* and its noise generator are published
without a licence. Nothing of them is used: no texture, no VDB, no recipe
read out of the generator.

The earlier talks the first cloud renderer followed still describe the
march's cheap and full samples and the temporal upsampling
(`fs_clouds_resolve`, `clouds.rs` `CloudsTemporal`): Schneider, *The
Real-time Volumetric Cloudscapes of Horizon Zero Dawn*, SIGGRAPH 2015, and
*Nubis: Authoring Real-Time Volumetric Cloudscapes with the Decima Engine*,
SIGGRAPH 2017.

- 2015: https://advances.realtimerendering.com/s2015/The%20Real-time%20Volumetric%20Cloudscapes%20of%20Horizon%20-%20Zero%20Dawn%20-%20ARTR.pdf
- 2017: https://advances.realtimerendering.com/s2017/Nubis%20-%20Authoring%20Realtime%20Volumetric%20Cloudscapes%20with%20the%20Decima%20Engine%20-%20Final%20.pdf

## Lighting

- Sébastien Hillaire, *Physically Based Sky, Atmosphere and Cloud Rendering
  in Frostbite*, SIGGRAPH 2016 (Physically Based Shading in Theory and
  Practice): the energy-conserving integration of a step's light, and the
  octaves of multiple scattering whose contribution never exceeds their
  extinction's scale.
  https://media.contentapi.ea.com/content/dam/eacom/frostbite/files/s2016-pbs-frostbite-sky-clouds-new.pdf
- Magnus Wrenninge, Christopher Kulla and Viktor Lundqvist, *Oz: The Great
  and Volumetric*, SIGGRAPH 2013 Talks: the octaves themselves.
- Sébastien Hillaire, *A Scalable and Production Ready Sky and Atmosphere
  Rendering Technique*, EGSR 2020, and *Physically Based and Scalable
  Atmospheres in Unreal Engine*, SIGGRAPH 2020 course notes: the Beer shadow
  map, a cloud's shadow along the light as a front depth, a mean extinction
  and a far depth (`cloud_shadow_bake.wgsl`, here a slab of the ray's mean
  height and spread over the cloud field's period).
- Johannes Jendersie and Eugene d'Eon, *An Approximate Mie Scattering
  Function for Fog and Cloud Rendering*, SIGGRAPH 2023 Talks: the blend of a
  Henyey-Greenstein and a Draine lobe and its fit to droplet diameter
  (equations 2 to 7), evaluated at 6 µm into `sky.wgsl`'s `CLOUD_G_HG`,
  `CLOUD_G_D`, `CLOUD_ALPHA` and `CLOUD_W_D`.
  https://research.nvidia.com/labs/rtr/approximate-mie/
- B. T. Draine, *Scattering by Interstellar Dust Grains. I. Optical and
  Ultraviolet*, The Astrophysical Journal 598, 2003: the Draine phase
  function.

## Marching

- John C. Hart, *Sphere Tracing: A Geometric Method for the Antialiased Ray
  Tracing of Implicit Surfaces*, The Visual Computer 12, 1996: a step as
  long as a bound on the field's slope leaves clear. Here the field is the
  billows' erosion less the profile, in the gaps between a cloud's billows
  (`clouds_shape.wgsl` `cloud_clear`).

## Filtering and dither

- Christian Sigg and Markus Hadwiger, *Fast Third-Order Texture Filtering*,
  GPU Gems 2, 2005, chapter 20: a cubic B-spline read as four bilinear
  reads (`clouds_shape.wgsl` `cloud_cubic`, the read of the cloud map and
  of the shadow map).
- Jorge Jimenez, *Next Generation Post Processing in Call of Duty: Advanced
  Warfare*, SIGGRAPH 2014 (Advances in Real-Time Rendering): interleaved
  gradient noise, the march's dither. Its turn per march is a 2 x 2 ordered
  dither and the golden ratio's sequence (`clouds.rs` `dither_turn`).

## Implementations read, not copied

- Takram, the clouds package of `three-geospatial` (MIT), and Sébastien Hillaire,
  `TileableVolumeNoise` (MIT): read for how a Beer shadow map and tiling
  Worley noise are laid out. No code is taken from either.

The papers and talks are descriptions of techniques, not licensed code.
