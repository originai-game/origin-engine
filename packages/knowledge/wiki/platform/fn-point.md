<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-point"></a>

### `og.platform.point`

Ask for Origin Point for something the player did.

```ts
og.platform.point(world: World, desc?: PointDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `PointDesc` | yes | Why and how many. |

**Returns** `Result<number>`: Refused for a reason outside 1 to 64 of letters, digits, `.`, `_` and `-`. The answer's data is `{ granted, balance }`; under a Player today always `{ granted: 0, balance: null }` (the platform grants nothing until a release carries an approved point policy), locally the amount asked.

Binding symbol `og_platform_point` · publish builds: kept

```ogt
fn point {
  doc "Ask for Origin Point for something the player did."
  param world World "The world."
  param desc PointDesc "Why and how many."
  returns Result.u32 "Refused for a reason outside 1 to 64 of letters, digits, `.`, `_` and `-`. The answer's data is `{ granted, balance }`; under a Player today always `{ granted: 0, balance: null }` (the platform grants nothing until a release carries an approved point policy), locally the amount asked."
}
```

<sub>`api.ogt` line 15071</sub>
