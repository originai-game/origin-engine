# xegtao

Intel's XeGTAO (Filip Strugar, https://github.com/GameTechDev/XeGTAO,
`Source/Rendering/Shaders/XeGTAO.hlsli`). MIT, Copyright (C) 2016-2021
Intel Corporation; the licence is in `LICENSE`. XeGTAO implements
Jimenez et al. 2016, "Practical Realtime Strategies for Accurate Indirect
Occlusion".

The engine's GTAO (`crates/og-render/shaders/fxpost.wgsl` `gtao_bent`)
takes from it the bent normal of the paper's Algorithm 2 extension:

- The per-slice terms `t0` and `t1` over the two horizon angles and the
  projected normal's angle.
- The rotation from the slice frame to the view vector
  (`XeGTAO_RotFromToMatrix`).
- The weighting by the projected normal's length.

`fs_gtao_bent` writes the bent normal in world space beside the AO when
the probe GI is on. The lit shaders read the probes' light along it, so
the sky and bounce come from the least occluded direction.
