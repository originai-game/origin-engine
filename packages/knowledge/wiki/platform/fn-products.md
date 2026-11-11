<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-products"></a>

### `og.platform.products`

Describe the products the game sells: titles and prices as the player's store shows them. The catalogue and the prices are the platform's.

```ts
og.platform.products(world: World, desc?: ProductsDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ProductsDesc` | yes | Which products. |

**Returns** `Result<number>`: Refused for a malformed id or more than 100. The answer's data is `{ products: [{ id, kind, title, description, price: { amount, currency, display } }] }`: kind `consumable`, `durable` or `subscription`, amount in the currency's units, display as the player's store writes it.

Binding symbol `og_platform_products` · publish builds: kept

```ogt
fn products {
  doc "Describe the products the game sells: titles and prices as the player's store shows them. The catalogue and the prices are the platform's."
  param world World "The world."
  param desc ProductsDesc "Which products." {
    optional true
  }
  returns Result.u32 "Refused for a malformed id or more than 100. The answer's data is `{ products: [{ id, kind, title, description, price: { amount, currency, display } }] }`: kind `consumable`, `durable` or `subscription`, amount in the currency's units, display as the player's store writes it."
}
```

<sub>Module `platform` line 320</sub>
