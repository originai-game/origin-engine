<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-consume"></a>

### `og.platform.consume`

Tell the host a consumable purchase was delivered: it leaves the entitlements. Grant it first (og.platform.grant).

```ts
og.platform.consume(world: World, purchase: string): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `purchase` | `string` | no | The purchase id an answer, an entitlement or a notice carried. |

**Returns** `Result<number>`: Refused for a malformed id (1 to 128 of letters, digits, `.`, `_`, `-` and `:`). The answer's data is `{ purchase, product, consumed: true }`; `not_found` for a purchase not owned or already consumed.

Binding symbol `og_platform_consume` · publish builds: kept

```ogt
fn consume {
  doc "Tell the host a consumable purchase was delivered: it leaves the entitlements. Grant it first (og.platform.grant)."
  param world World "The world."
  param purchase String "The purchase id an answer, an entitlement or a notice carried."
  returns Result.u32 "Refused for a malformed id (1 to 128 of letters, digits, `.`, `_`, `-` and `:`). The answer's data is `{ purchase, product, consumed: true }`; `not_found` for a purchase not owned or already consumed."
}
```

<sub>Module `platform` line 343</sub>
