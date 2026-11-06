<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`math`](../math.md) · [index](../index.md)

<a id="type-Color"></a>

### `Color` (struct)

A linear RGB colour, as three.js Color with colour management on. Components are usually between 0 and 1; above 1 is HDR.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `r` | `number` | `0` | — | — | — | The red component. |
| `g` | `number` | `0` | — | — | — | The green component. |
| `b` | `number` | `0` | — | — | — | The blue component. |

```ogt
struct Color {
  doc "A linear RGB colour, as three.js Color with colour management on. Components are usually between 0 and 1; above 1 is HDR."
  field r f32 "The red component."
  field g f32 "The green component."
  field b f32 "The blue component."
}
```

<sub>Module `math` line 35</sub>
