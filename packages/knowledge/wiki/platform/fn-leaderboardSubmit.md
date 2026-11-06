<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-leaderboardSubmit"></a>

### `og.platform.leaderboardSubmit`

Submit a score; the board keeps the player's best.

```ts
og.platform.leaderboardSubmit(world: World, desc?: LeaderboardSubmitDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `LeaderboardSubmitDesc` | yes | The board, the score and its order. |

**Returns** `Result<number>`: Refused for a score that is not finite. The answer's data is `{ board, rank, best }`, or `{ board, rank: null, queued: true }` while offline.

Binding symbol `og_platform_leaderboard_submit` · publish builds: kept

```ogt
fn leaderboardSubmit {
  doc "Submit a score; the board keeps the player's best."
  param world World "The world."
  param desc LeaderboardSubmitDesc "The board, the score and its order."
  returns Result.u32 "Refused for a score that is not finite. The answer's data is `{ board, rank, best }`, or `{ board, rank: null, queued: true }` while offline."
}
```

<sub>`api.ogt` line 15025</sub>
