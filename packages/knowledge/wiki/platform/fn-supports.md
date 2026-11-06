<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-supports"></a>

### `og.platform.supports`

Whether the host answers platform method `method` (`purchase.buy`, `ad.show`, `share`: og.platform.info's capabilities): what to offer the player before asking. A method it lacks is answered `unavailable`.

```ts
og.platform.supports(world: World, method: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `method` | `string` | no | The method. |

**Returns** `boolean`.

Binding symbol `og_platform_supports` · publish builds: kept

```ogt
fn supports {
  doc "Whether the host answers platform method `method` (`purchase.buy`, `ad.show`, `share`: og.platform.info's capabilities): what to offer the player before asking. A method it lacks is answered `unavailable`."
  param world World "The world."
  param method String "The method."
  returns bool
}
```

<sub>`api.ogt` line 15253</sub>
