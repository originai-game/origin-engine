<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-entitlements"></a>

### `og.platform.entitlements`

What the player owns in this game, as the host knows it: durables, active subscriptions and consumables not yet consumed. Ask when the game starts and after a restore, never trust a save for it.

```ts
og.platform.entitlements(world: World): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `number`: The answer's data is `{ entitlements: [{ purchase, product, kind, quantity, expires }] }`, expires UTC milliseconds for a subscription, null otherwise.

Binding symbol `og_platform_entitlements` · publish builds: kept

```ogt
fn entitlements {
  doc "What the player owns in this game, as the host knows it: durables, active subscriptions and consumables not yet consumed. Ask when the game starts and after a restore, never trust a save for it."
  param world World "The world."
  returns u32 "The answer's data is `{ entitlements: [{ purchase, product, kind, quantity, expires }] }`, expires UTC milliseconds for a subscription, null otherwise."
}
```

<sub>`api.ogt` line 15216</sub>
