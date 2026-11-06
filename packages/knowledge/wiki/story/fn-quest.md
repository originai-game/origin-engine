<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-quest"></a>

### `og.story.quest`

A started quest as the log shows it.

```ts
og.story.quest(world: World, id: string): QuestView | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `id` | `string` | no | The quest's id. |

**Returns** `QuestView | null`: Null for a quest not started.

Binding symbol `og_story_quest` · publish builds: kept

```ogt
fn quest {
  doc "A started quest as the log shows it."
  param world World "The world."
  param id String "The quest's id."
  returns Option.QuestView "Null for a quest not started."
}
```

<sub>Module `story` line 405</sub>
