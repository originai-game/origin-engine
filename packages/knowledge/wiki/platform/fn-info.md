<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-info"></a>

### `og.platform.info`

What answers platform calls here, and what it answers.

```ts
og.platform.info(world: World): PlatformInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `PlatformInfo`.

Binding symbol `og_platform_info` · publish builds: kept

```ogt
fn info {
  doc "What answers platform calls here, and what it answers."
  param world World "The world."
  returns PlatformInfo
}
```

<sub>`api.ogt` line 15351</sub>
