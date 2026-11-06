<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-granted"></a>

### `og.platform.granted`

Whether this world granted purchase `purchase` (og.platform.grant). Inside a system, declare og.PlatformGrants among its reads.

```ts
og.platform.granted(world: World, purchase: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `purchase` | `string` | no | The purchase id. |

**Returns** `boolean`.

Binding symbol `og_platform_granted` · publish builds: kept

```ogt
fn granted {
  doc "Whether this world granted purchase `purchase` (og.platform.grant). Inside a system, declare og.PlatformGrants among its reads."
  param world World "The world."
  param purchase String "The purchase id."
  returns bool
}
```

<sub>`api.ogt` line 15251</sub>
