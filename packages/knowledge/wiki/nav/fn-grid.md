<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-grid"></a>

### `og.nav.grid`

A grid agents and path queries plan on. Runtime edits (og.nav.setCost) are world state.

```ts
og.nav.grid(world: World, desc?: NavGridDesc): Result<Navigation>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `NavGridDesc` | yes | Its size, cell size, origin and costs. |

**Returns** `Result<Navigation>`: Refused for costs of the wrong length.

Binding symbol `og_nav_grid` · publish builds: kept

```ogt
fn grid {
  doc "A grid agents and path queries plan on. Runtime edits (og.nav.setCost) are world state."
  param world World "The world."
  param desc NavGridDesc "Its size, cell size, origin and costs."
  returns Result.Navigation "Refused for costs of the wrong length."
}
```

<sub>`api.ogt` line 14022</sub>
