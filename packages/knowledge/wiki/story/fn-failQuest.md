<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-failQuest"></a>

### `og.story.failQuest`

Fail an active quest now: its undone objectives are marked failed.

```ts
og.story.failQuest(world: World, quest: string): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `quest` | `string` | no | The quest's id. |

**Returns** `Result<boolean>`: False when it is not active. Refused for a quest not loaded.

Binding symbol `og_story_fail_quest` · publish builds: kept

```ogt
fn failQuest {
  doc "Fail an active quest now: its undone objectives are marked failed."
  param world World "The world."
  param quest String "The quest's id."
  returns Result.bool "False when it is not active. Refused for a quest not loaded."
}
```

<sub>`api.ogt` line 14855</sub>
