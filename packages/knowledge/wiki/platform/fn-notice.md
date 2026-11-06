<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-notice"></a>

### `og.platform.notice`

A notice og.PlatformNotice announced. The engine keeps the last 256.

```ts
og.platform.notice(world: World, notice: number): PlatformNoticeData | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `notice` | `number` | no | og.PlatformNotice.notice. |

**Returns** `PlatformNoticeData | null`: Null for a number never given, one older than the last 256, or inside a system that writes shared state in a running lockstep session.

Binding symbol `og_platform_notice` · publish builds: kept

```ogt
fn notice {
  doc "A notice og.PlatformNotice announced. The engine keeps the last 256."
  param world World "The world."
  param notice u32 "og.PlatformNotice.notice."
  returns Option.PlatformNoticeData "Null for a number never given, one older than the last 256, or inside a system that writes shared state in a running lockstep session."
}
```

<sub>`api.ogt` line 15338</sub>
