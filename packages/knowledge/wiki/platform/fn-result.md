<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-result"></a>

### `og.platform.result`

The answer to a call, once it arrived.

```ts
og.platform.result(world: World, request: number): PlatformResult | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `request` | `number` | no | The call. |

**Returns** `PlatformResult | null`: Null while it is in flight, for an unknown request, or inside a system that writes shared state in a running lockstep session.

Binding symbol `og_platform_result` · publish builds: kept

```ogt
fn result {
  doc "The answer to a call, once it arrived."
  param world World "The world."
  param request u32 "The call."
  returns Option.PlatformResult "Null while it is in flight, for an unknown request, or inside a system that writes shared state in a running lockstep session."
}
```

<sub>`api.ogt` line 15330</sub>
