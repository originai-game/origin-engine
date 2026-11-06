<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-EnvironmentInfo"></a>

### `EnvironmentInfo` (struct)

What a ready uncooked environment holds.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `map` | `EnvironmentMap` | `0` | — | — | — | The map, for Environment.map or Environment.blendTo, lit and drawn like a cooked one. |
| `width` | `number` | `0` | — | — | — | The image's width in pixels. |
| `height` | `number` | `0` | — | — | — | The image's height in pixels. |
| `size` | `number` | `0` | — | — | — | The prefiltered cube's face size. |

```ogt
struct EnvironmentInfo {
  doc "What a ready uncooked environment holds."
  field map EnvironmentMap "The map, for Environment.map or Environment.blendTo, lit and drawn like a cooked one."
  field width u32 "The image's width in pixels."
  field height u32 "The image's height in pixels."
  field size u32 "The prefiltered cube's face size."
}
```

<sub>`api.ogt` line 8403</sub>
