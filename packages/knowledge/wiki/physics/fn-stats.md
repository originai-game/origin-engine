<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-stats"></a>

### `og.physics.stats`

What the last physics step cost: each phase's time and whether it ran on the lanes pool. All zero before the first step.

```ts
og.physics.stats(world: World): Result<PhysicsStats>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `Result<PhysicsStats>`: Refused for an engine without physics.

Binding symbol `og_physics_stats` · publish builds: kept

```ogt
fn stats {
  doc "What the last physics step cost: each phase's time and whether it ran on the lanes pool. All zero before the first step."
  param world World "The world."
  returns Result.PhysicsStats "Refused for an engine without physics."
}
```

<sub>Module `physics` line 2145</sub>
