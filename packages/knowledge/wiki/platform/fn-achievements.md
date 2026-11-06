<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-achievements"></a>

### `og.platform.achievements`

The player's achievements in this game.

```ts
og.platform.achievements(world: World): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `number`: The answer's data is `{ achievements: [{ id, percent, unlocked, time }] }`, time the unlock's UTC milliseconds or null.

Binding symbol `og_platform_achievements` · publish builds: kept

```ogt
fn achievements {
  doc "The player's achievements in this game."
  param world World "The world."
  returns u32 "The answer's data is `{ achievements: [{ id, percent, unlocked, time }] }`, time the unlock's UTC milliseconds or null."
}
```

<sub>`api.ogt` line 15287</sub>
