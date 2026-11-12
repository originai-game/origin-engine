<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-ReflectionProbeMode"></a>

### `ReflectionProbeMode` (enum)

When a reflection probe's cube is rendered.

A string: one of the variants below. The zero value is `"baked"`.

| Variant | Doc |
|---|---|
| `"baked"` | Once: read from the project's cook cache when `og bake probes` wrote it there, otherwise rendered on its first frame and kept. Static rooms. |
| `"realtime"` | Again every `interval` ticks, so moving things show in it (a cost per update of six faces). |

```ogt
enum ReflectionProbeMode {
  doc "When a reflection probe's cube is rendered."
  variant baked "Once: read from the project's cook cache when `og bake probes` wrote it there, otherwise rendered on its first frame and kept. Static rooms."
  variant realtime "Again every `interval` ticks, so moving things show in it (a cost per update of six faces)."
}
```

<sub>Module `render` line 2838</sub>
