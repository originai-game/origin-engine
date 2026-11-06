<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="type-DestructibleMaterial"></a>

### `DestructibleMaterial` (struct)

A material's og.Destructible fields (og.destruction.material): spread it into the component and add the material's fx, `{ ...og.destruction.material(world, MATERIAL_BRICK), breakFx }`, so a prop is tuned by what it is made of rather than by numbers in the scene. Its fields are og.Destructible's, for a prop of about a metre; override any one after the spread.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `toughness` | `number` | `0` | — | `newtonSecondPerM2` | — | og.Destructible.toughness. |
| `strength` | `number` | `0` | — | `pascal` | — | og.Destructible.strength. |
| `friction` | `number` | `0` | — | — | — | og.Destructible.friction. |
| `tension` | `number` | `0` | — | — | — | og.Destructible.tension. |
| `density` | `number` | `0` | — | `kgPerM3` | — | og.Destructible.density. |
| `impactThreshold` | `number` | `0` | — | `newtonSecond` | — | og.Destructible.impactThreshold. |
| `impactRadius` | `number` | `0` | — | `m` | — | og.Destructible.impactRadius. |

```ogt
struct DestructibleMaterial {
  doc "A material's og.Destructible fields (og.destruction.material): spread it into the component and add the material's fx, `{ ...og.destruction.material(world, MATERIAL_BRICK), breakFx }`, so a prop is tuned by what it is made of rather than by numbers in the scene. Its fields are og.Destructible's, for a prop of about a metre; override any one after the spread."
  field toughness f32 "og.Destructible.toughness." {
    unit newtonSecondPerM2
  }
  field strength f32 "og.Destructible.strength." {
    unit pascal
  }
  field friction f32 "og.Destructible.friction."
  field tension f32 "og.Destructible.tension."
  field density f32 "og.Destructible.density." {
    unit kgPerM3
  }
  field impactThreshold f32 "og.Destructible.impactThreshold." {
    unit newtonSecond
  }
  field impactRadius f32 "og.Destructible.impactRadius." {
    unit m
  }
}
```

<sub>Module `destruction` line 227</sub>
