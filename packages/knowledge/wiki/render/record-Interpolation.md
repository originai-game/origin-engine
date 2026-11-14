<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-Interpolation"></a>

### `og.Interpolation` (component)

How the renderer draws this entity between ticks while RenderSettings.interpolate is on; an entity without it is blended. Presentation only: the renderer reads it, the simulation never does.

See `render §9`.

Token `og.Interpolation`, registered under the schema name `og.Interpolation`; its value type is `Interpolation`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `off` | `boolean` | `false` | — | — | — | Draw this entity, and everything under it through og.Parent, at the last tick's transform on every frame, never blended: a marker the game already moves every frame, or an object that jumps on purpose (a clock hand that ticks). |

```ogt
component Interpolation {
  doc "How the renderer draws this entity between ticks while RenderSettings.interpolate is on; an entity without it is blended. Presentation only: the renderer reads it, the simulation never does."
  guide "render §9"
  field off bool "Draw this entity, and everything under it through og.Parent, at the last tick's transform on every frame, never blended: a marker the game already moves every frame, or an object that jumps on purpose (a clock hand that ticks)."
}
```

<sub>Module `render` line 825</sub>
