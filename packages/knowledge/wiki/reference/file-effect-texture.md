<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="file-effect-texture"></a>
### `file:effect-texture`

Checked `file["effect-texture"](input)` declaration from `@origingame/engine/author`.

```ts
/** Offline texture transport input; all appearance values are explicit.
Atlas dimensions, total work and nonzero light direction are domain checks. */
export type Input = ({
/** Nonnegative Beer-Lambert extinction/density/cube width. */
readonly "absorption": (number);
/** Nonnegative isotropic scattering light; does not change alpha or emission. */
readonly "ambient": (number);
/** Signed Y acceleration per unit heat. */
readonly "buoyancy": (number);
/** Linear scattering RGB, each 0..1, illuminated by ambient/direct/light. */
readonly "color": (ReadonlyArray<(number)> & { readonly length: 3 });
/** Sheet columns. Domain validation also checks columns <= frames. */
readonly "columns": (number);
/** Nonnegative exponential heat loss/second. */
readonly "cooling": (number);
/** Nonnegative exponential density loss/second. */
readonly "decay": (number);
/** Density/second; explosion uses initial density. Nonnegative. */
readonly "density": (number);
/** Source boundary/density fBm modulation, 0..1; zero is a smooth sphere. */
readonly "detail": (number);
/** Nonnegative directional light with volume self-shadow; RGB clamps to sRGB8. */
readonly "direct": (number);
/** Seconds per substep, >0..0.1; noise time is (frame+1)*steps*dt. */
readonly "dt": (number);
/** Linear emission RGB/unit heat, each 0..1, written separately to #emission. */
readonly "emission": (ReadonlyArray<(number)> & { readonly length: 3 });
/** Nonnegative radial source speed in cube widths/s. */
readonly "expansion": (number);
/** 1..256 chronological frames, in row-major sheet order. */
readonly "frames": (number);
/** Positive cycles per unit cube; three simplex octaves, lacunarity 2, gain 0.5. */
readonly "frequency": (number);
/** 8..96 volume cells per axis of the unit cube; no per-frame rescaling. */
readonly "grid": (number);
/** Heat/second; explosion uses initial heat. Nonnegative. */
readonly "heat": (number);
/** Nonnegative source duration in seconds; explosion injects only initially. */
readonly "inject": (number);
/** Nonzero direction toward the light, normalized at cook. View rays travel +Z. */
readonly "light": (ReadonlyArray<(number)> & { readonly length: 3 });
/** Density/heat transport, or the shared noise/Worley/curl kernels; no preset. */
readonly "mode": ("smoke" | "fire" | "explosion" | "noise" | "worley" | "curl") & (string);
/** 1..128 Jacobi pressure iterations per volume step. */
readonly "pressure": (number);
/** Injection radius in cube widths, >0..0.5. */
readonly "radius": (number);
/** Positive encoding range: signed noise/curl -range..range maps to 0..1;
Worley 0..range maps to 0..1. Values clamp. Unused for volume transport. */
readonly "range": (number);
/** Seed for the same fBm/curl/Worley kernels as og.procgen. */
readonly "seed": (number);
/** Square frame resolution, 8..1024. Sheet extents <=8192; <=16777216 frame pixels. */
readonly "size": (number);
/** Injection centre in the unit cube, Y up. */
readonly "source": (ReadonlyArray<(number)> & { readonly length: 3 });
/** 1..64 fixed simulation substeps between frames; frame zero follows one interval. */
readonly "steps": (number);
/** Nonnegative curl acceleration; on noise textures this is Z travel speed. */
readonly "turbulence": (number);
/** Nonnegative vorticity confinement in 1/s, scaled by cell width; zero disables it. */
readonly "vorticity": (number);
});
```
