<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="fn-explode"></a>

### `og.destruction.explode`

An explosion: damage to every destructible in reach (as og.destruction.damage), then a push outwards to their loose chunks. Applied by og.destruction this tick, as damage. Push other bodies with og.physics.applyImpulse.

Also known as: blast, explosion, shatter, demolish, blow apart

```ts
og.destruction.explode(world: World, desc?: ExplosionDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ExplosionDesc` | yes | Its centre, reach, damage and push. |

**Returns** `Result<void>`: Refused for a point that is not finite.

Binding symbol `og_destruction_explode` · publish builds: kept

```ogt
fn explode {
  doc "An explosion: damage to every destructible in reach (as og.destruction.damage), then a push outwards to their loose chunks. Applied by og.destruction this tick, as damage. Push other bodies with og.physics.applyImpulse."
  aliases "blast, explosion, shatter, demolish, blow apart"
  param world World "The world."
  param desc ExplosionDesc "Its centre, reach, damage and push."
  returns Result.Void "Refused for a point that is not finite."
}
```

<sub>`api.ogt` line 27359</sub>
