<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-pathOf"></a>

### `og.nav.pathOf`

The path an agent follows now (its corners from where it planned), for debugging and playtests; `dump_nav:<entity>` writes the same.

```ts
og.nav.pathOf(world: World, entity: Entity): NavPath | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.NavAgent. |

**Returns** `NavPath | null`: Null for an entity that has not planned.

Binding symbol `og_nav_path_of` · publish builds: kept

```ogt
fn pathOf {
  doc "The path an agent follows now (its corners from where it planned), for debugging and playtests; `dump_nav:<entity>` writes the same."
  param world World "The world."
  param entity Entity "An entity with an og.NavAgent."
  returns Option.NavPath "Null for an entity that has not planned."
}
```

<sub>`api.ogt` line 14190</sub>
