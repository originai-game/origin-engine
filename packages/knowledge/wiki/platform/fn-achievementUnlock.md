<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-achievementUnlock"></a>

### `og.platform.achievementUnlock`

Unlock an achievement. The achievements and their titles are the platform's, by id.

Also known as: unlock trophy, grant achievement, Steam SetAchievement

```ts
og.platform.achievementUnlock(world: World, id: string): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `id` | `string` | no | The achievement's id. |

**Returns** `Result<number>`: Refused for a malformed id. The answer's data is `{ id, percent: 100, unlocked: true, new }`, new false when it was unlocked before.

Binding symbol `og_platform_achievement_unlock` · publish builds: kept

```ogt
fn achievementUnlock {
  doc "Unlock an achievement. The achievements and their titles are the platform's, by id."
  aliases "unlock trophy, grant achievement, Steam SetAchievement"
  param world World "The world."
  param id String "The achievement's id."
  returns Result.u32 "Refused for a malformed id. The answer's data is `{ id, percent: 100, unlocked: true, new }`, new false when it was unlocked before."
}
```

<sub>Module `platform` line 379</sub>
