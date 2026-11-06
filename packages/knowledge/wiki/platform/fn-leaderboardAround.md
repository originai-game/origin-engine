<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-leaderboardAround"></a>

### `og.platform.leaderboardAround`

The entries around the player's rank.

```ts
og.platform.leaderboardAround(world: World, desc?: LeaderboardQueryDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `LeaderboardQueryDesc` | yes | The board and how many around the player. |

**Returns** `number`: The answer's data is the leaderboardTop shape with the entries around the player's rank.

Binding symbol `og_platform_leaderboard_around` · publish builds: kept

```ogt
fn leaderboardAround {
  doc "The entries around the player's rank."
  param world World "The world."
  param desc LeaderboardQueryDesc "The board and how many around the player."
  returns u32 "The answer's data is the leaderboardTop shape with the entries around the player's rank."
}
```

<sub>Module `platform` line 155</sub>
