<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-achievementProgress"></a>

### `og.platform.achievementProgress`

Report progress towards an achievement; 100 unlocks it.

```ts
og.platform.achievementProgress(world: World, desc?: AchievementProgressDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `AchievementProgressDesc` | yes | The id and how far. |

**Returns** `Result<number>`: Refused for a malformed id or a percent outside 0 to 100. The answer's data is `{ id, percent, unlocked, new }`.

Binding symbol `og_platform_achievement_progress` · publish builds: kept

```ogt
fn achievementProgress {
  doc "Report progress towards an achievement; 100 unlocks it."
  param world World "The world."
  param desc AchievementProgressDesc "The id and how far."
  returns Result.u32 "Refused for a malformed id or a percent outside 0 to 100. The answer's data is `{ id, percent, unlocked, new }`."
}
```

<sub>`api.ogt` line 15281</sub>
