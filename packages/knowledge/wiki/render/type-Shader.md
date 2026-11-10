<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-Shader"></a>

### `Shader` (handle)

A material shader from og.load.shader (an `.ogt` of `type shader`): WGSL hooks of the lit shader, or with `mode full` its own vertex and fragment entries, and the parameters they read.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Shader "A material shader from og.load.shader (an `.ogt` of `type shader`): WGSL hooks of the lit shader, or with `mode full` its own vertex and fragment entries, and the parameters they read."
```

<sub>Module `render` line 2680</sub>
