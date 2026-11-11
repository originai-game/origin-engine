<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-adReady"></a>

### `og.platform.adReady`

Ask whether an ad can be shown now, to offer a rewarded one only when there is one.

```ts
og.platform.adReady(world: World, desc?: AdDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `AdDesc` | yes | The kind and placement. |

**Returns** `Result<number>`: Refused for a malformed placement. The answer's data is `{ kind, placement, ready }`.

Binding symbol `og_platform_ad_ready` · publish builds: kept

```ogt
fn adReady {
  doc "Ask whether an ad can be shown now, to offer a rewarded one only when there is one."
  param world World "The world."
  param desc AdDesc "The kind and placement."
  returns Result.u32 "Refused for a malformed placement. The answer's data is `{ kind, placement, ready }`."
}
```

<sub>Module `platform` line 363</sub>
