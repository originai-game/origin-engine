<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-leaderboardTop"></a>

### `og.platform.leaderboardTop`

The best entries of a board, and the player's own.

Also known as: high scores, leaderboard, ranking, top players

```ts
og.platform.leaderboardTop(world: World, desc?: LeaderboardQueryDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `LeaderboardQueryDesc` | yes | The board and how many. |

**Returns** `number`: The answer's data is `{ board, order, entries: [{ rank, name, score, me }], me }`.

Binding symbol `og_platform_leaderboard_top` · publish builds: kept

```ogt
fn leaderboardTop {
  doc "The best entries of a board, and the player's own."
  aliases "high scores, leaderboard, ranking, top players"
  param world World "The world."
  param desc LeaderboardQueryDesc "The board and how many."
  returns u32 "The answer's data is `{ board, order, entries: [{ rank, name, score, me }], me }`."
}
```

<sub>`api.ogt` line 15054</sub>
