# Physical sky solar disc

`crates/og-render/shaders/sky.wgsl` follows the separate, additive HDR
sun-disc composition of Sébastien Hillaire, *A Scalable and Production Ready
Sky and Atmosphere Rendering Technique*, EGSR 2020 (§5.3).

- Paper: https://sebh.github.io/publications/egsr2020.pdf
- Reference implementation: https://github.com/sebh/UnrealEngineSkyAtmosphere
- Revision: `183ead5bdacc701b3b626347a680a2f3cd3d4fbd`
- Relevant code: `Resources/RenderSkyRayMarching.hlsl` (sky plus disc),
  `Resources/RenderSkyCommon.hlsl` (`GetSunLuminance`).

The reference's demo uses arbitrary solar luminance 1,000,000. Origin keeps
its existing scene-linear `SkyLook.sun_intensity` units instead, attenuates
each channel with its atmospheric transmittance, and retains its linear
limb-darkening profile. No display shoulder is applied to this radiance;
the tone mapper owns display mapping. The final sky pass only bounds the
sum to finite RGBA16F storage. This is a composition adaptation, not a port
of the demo's arbitrary solar brightness or the proprietary UE code.

## Reference implementation licence

MIT License

Copyright (c) 2020 Epic Games, Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
