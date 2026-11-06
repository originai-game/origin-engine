<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-share"></a>

### `og.platform.share`

Share through the device's share sheet: the text, the game's own Play link and, with image, the frame drawn last.

```ts
og.platform.share(world: World, desc?: ShareDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `ShareDesc` | yes | What to share. |

**Returns** `Result<number>`: Refused for an empty text, one over 280 characters, a title over 80 or a malformed path. The answer's data is `{ shared: true, target }` (target the app the player picked, when the host knows it, else null); `cancelled` when the player closed the sheet.

Binding symbol `og_platform_share` · publish builds: kept

```ogt
fn share {
  doc "Share through the device's share sheet: the text, the game's own Play link and, with image, the frame drawn last."
  param world World "The world."
  param desc ShareDesc "What to share."
  returns Result.u32 "Refused for an empty text, one over 280 characters, a title over 80 or a malformed path. The answer's data is `{ shared: true, target }` (target the app the player picked, when the host knows it, else null); `cancelled` when the player closed the sheet."
}
```

<sub>`api.ogt` line 15331</sub>
