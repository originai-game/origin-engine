<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-purchase"></a>

### `og.platform.purchase`

Ask the player to buy a product: the host shows its own purchase sheet and takes the money. The answer is the host's word on it; deliver a consumable with og.platform.grant and then og.platform.consume.

See `saves §8`.

```ts
og.platform.purchase(world: World, desc?: PurchaseDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `PurchaseDesc` | yes | The product and how many. |

**Returns** `Result<number>`: Refused for a malformed product id. Data `{ purchase, product, quantity, state }`: state `completed` or `pending` (a parent's approval or a payment; a `purchase` notice settles it). Errors: `cancelled` (the player backed out), `not_found`, `conflict` (a durable already owned), `unavailable`.

Binding symbol `og_platform_purchase` · publish builds: kept

```ogt
fn purchase {
  doc "Ask the player to buy a product: the host shows its own purchase sheet and takes the money. The answer is the host's word on it; deliver a consumable with og.platform.grant and then og.platform.consume."
  guide "saves §8"
  param world World "The world."
  param desc PurchaseDesc "The product and how many."
  returns Result.u32 "Refused for a malformed product id. Data `{ purchase, product, quantity, state }`: state `completed` or `pending` (a parent's approval or a payment; a `purchase` notice settles it). Errors: `cancelled` (the player backed out), `not_found`, `conflict` (a durable already owned), `unavailable`."
}
```

<sub>Module `platform` line 328</sub>
