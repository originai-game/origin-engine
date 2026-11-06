<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-grant"></a>

### `og.platform.grant`

Record that this world granted purchase `purchase` (in og.PlatformGrants): true the first time, false for one already granted, so a purchase's answer, notice or entitlement seen again (a replay, a relaunch, a save restored from before og.platform.consume) grants once. It records; it is not proof of payment (the host's answer is). Inside a system, declare og.PlatformGrants among its writes.

```ts
og.platform.grant(world: World, purchase: string): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `purchase` | `string` | no | The purchase id. |

**Returns** `Result<boolean>`: Whether it was new. Refused for a malformed id, or inside a system that did not declare og.PlatformGrants.

Binding symbol `og_platform_grant` · publish builds: kept

```ogt
fn grant {
  doc "Record that this world granted purchase `purchase` (in og.PlatformGrants): true the first time, false for one already granted, so a purchase's answer, notice or entitlement seen again (a replay, a relaunch, a save restored from before og.platform.consume) grants once. It records; it is not proof of payment (the host's answer is). Inside a system, declare og.PlatformGrants among its writes."
  param world World "The world."
  param purchase String "The purchase id."
  returns Result.bool "Whether it was new. Refused for a malformed id, or inside a system that did not declare og.PlatformGrants."
}
```

<sub>`api.ogt` line 15229</sub>
