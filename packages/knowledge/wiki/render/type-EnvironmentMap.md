<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-EnvironmentMap"></a>

### `EnvironmentMap` (handle)

A prefiltered environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular HDR/EXR or sRGB PNG/JPG/WebP, a KTX2 cube map, or six face images). LDR sources keep authored background colours apart from linear lighting; hemisphere upper fills the ground, and og.load.environmentSun reports the extracted sun for a DirectionalLight.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle EnvironmentMap "A prefiltered environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular HDR/EXR or sRGB PNG/JPG/WebP, a KTX2 cube map, or six face images). LDR sources keep authored background colours apart from linear lighting; hemisphere upper fills the ground, and og.load.environmentSun reports the extracted sun for a DirectionalLight."
```

<sub>Module `render` line 987</sub>
