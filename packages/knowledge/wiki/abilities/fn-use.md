<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`abilities`](../abilities.md) · [index](../index.md)

<a id="fn-use"></a>

### `og.abilities.use`

Use an ability now: a charge, its lock and its cost, then og.AbilityUsed. A system that calls this lists og.Ability, og.Stat and og.AbilityUsed in its writes.

Also known as: cast, activate ability, trigger skill

```ts
og.abilities.use(world: World, ability: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `ability` | `Entity` | no | The og.Ability. |

**Returns** `Result<void>`: Refused, changing nothing, without a charge, while locked or when its pool holds less than its cost, naming which.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn use {
  doc "Use an ability now: a charge, its lock and its cost, then og.AbilityUsed. A system that calls this lists og.Ability, og.Stat and og.AbilityUsed in its writes."
  aliases "cast, activate ability, trigger skill"
  host false
  param world World "The world."
  param ability Entity "The og.Ability."
  returns Result.Void "Refused, changing nothing, without a charge, while locked or when its pool holds less than its cost, naming which."
}
```

<sub>Module `abilities` line 134</sub>
