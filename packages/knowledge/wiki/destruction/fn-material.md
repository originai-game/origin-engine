<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="fn-material"></a>

### `og.destruction.material`

A material's og.Destructible tuning (MATERIAL_*): toughness, strength, friction, tension, density and how hard a hit must be to hurt it, measured against the props of the destruction-yard test fixture (a brick wall, a concrete pillar, a crate, a pane). Pair it with a model whose fracture block has the same `style` (brick, concrete, wood, glass).

```ts
og.destruction.material(world: World, kind: number): Result<DestructibleMaterial>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `kind` | `number` | no | MATERIAL_*. |

**Returns** `Result<DestructibleMaterial>`: Refused for a kind that is none of MATERIAL_*.

Binding symbol `og_destruction_material` · publish builds: kept

```ogt
fn material {
  doc "A material's og.Destructible tuning (MATERIAL_*): toughness, strength, friction, tension, density and how hard a hit must be to hurt it, measured against the props of the destruction-yard test fixture (a brick wall, a concrete pillar, a crate, a pane). Pair it with a model whose fracture block has the same `style` (brick, concrete, wood, glass)."
  param world World "The world."
  param kind u32 "MATERIAL_*."
  returns Result.DestructibleMaterial "Refused for a kind that is none of MATERIAL_*."
}
```

<sub>Module `destruction` line 294</sub>
