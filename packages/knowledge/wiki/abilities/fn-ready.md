<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`abilities`](../abilities.md) · [index](../index.md)

<a id="fn-ready"></a>

### `og.abilities.ready`

Whether an ability can be used now: a charge, no lock, and its pool's value at least its cost.

```ts
og.abilities.ready(world: World, ability: Entity): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `ability` | `Entity` | no | The og.Ability. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn ready {
  doc "Whether an ability can be used now: a charge, no lock, and its pool's value at least its cost."
  host false
  param world World "The world."
  param ability Entity "The og.Ability."
  returns bool
}
```

<sub>Module `abilities` line 126</sub>
