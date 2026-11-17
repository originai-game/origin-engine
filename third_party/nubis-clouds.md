# Volumetric cloudscapes

`crates/og-render/shaders/clouds_noise.wgsl`, `clouds_shape.wgsl`,
`sky.wgsl` `clouds_march` and `fs_clouds_resolve` follow published descriptions; no code is copied.

## Shape and march

Andrew Schneider, *The Real-time Volumetric Cloudscapes of Horizon Zero
Dawn*, SIGGRAPH 2015 (Advances in Real-Time Rendering), and *Nubis:
Authoring Real-Time Volumetric Cloudscapes with the Decima Engine*, SIGGRAPH
2017.

- 2015: https://advances.realtimerendering.com/s2015/The%20Real-time%20Volumetric%20Cloudscapes%20of%20Horizon%20-%20Zero%20Dawn%20-%20ARTR.pdf
- 2017: https://advances.realtimerendering.com/s2017/Nubis%20-%20Authoring%20Realtime%20Volumetric%20Cloudscapes%20with%20the%20Decima%20Engine%20-%20Final%20.pdf

What Origin takes from them:

- the base shape, a low-frequency Perlin-Worley noise carved by Worley fBms
  at three frequencies, and the high-frequency Worley detail that erodes it
  (wisps at the base, billows above);
- the weather map that says where clouds stand and how much, the height
  gradient that stands them on the layer's base, and the coverage remap;
- the march in long steps through clear air on the cheap sample, going back
  and crossing a cloud in short steps on the full one, and the cone of
  widening samples towards the light;
- the temporal reprojection (2015): each frame marches one pixel of each
  two by two of a reduced-resolution target and reprojects the rest from
  the frame before (`fs_clouds_resolve`, `clouds.rs` `CloudsTemporal`).

What differs: the detail moves the surface about a mean erosion, at two
and a half times the Worley fBms' own spread, rather than only eroding it;
the reprojection clamps the history to the nine marched pixels around each
pixel, as TAA does; the three carving fBms are baked into the shape's channel (one
read a sample, not four), both noises share one 128³ volume, the weather map
is procedural (a 2D value fBm in the volume's alpha) with the cloud type
read from the cover, the channels are equalised so `Clouds.coverage` is the
share of the sky covered, and the papers' "powder" term is not used.

## Lighting

Sébastien Hillaire, *Physically Based Sky, Atmosphere and Cloud Rendering in
Frostbite*, SIGGRAPH 2016 (Physically Based Shading in Theory and Practice):
the energy-conserving integration of a step's light, and the octaves of
multiple scattering after Wrenninge, Kulla and Lundqvist, *Oz: The Great and
Volumetric*, SIGGRAPH 2013 Talks.

- https://media.contentapi.ea.com/content/dam/eacom/frostbite/files/s2016-pbs-frostbite-sky-clouds-new.pdf

The papers are descriptions of techniques, not licensed code; the WGSL here
is Origin's own.
