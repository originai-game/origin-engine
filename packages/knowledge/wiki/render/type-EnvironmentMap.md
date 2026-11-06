<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-EnvironmentMap"></a>

### `EnvironmentMap` (handle)

A prefiltered HDR environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular `.hdr` or `.exr`, a KTX2 cube map, or six face images).

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle EnvironmentMap "A prefiltered HDR environment map for Environment.map, from og.load.environment (an `.ogt` of `type environment` naming an equirectangular `.hdr` or `.exr`, a KTX2 cube map, or six face images)."
```

<sub>`api.ogt` line 4492</sub>
