<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-quests"></a>

### `og.story.quests`

Every started quest, in the order they started (a quest log).

```ts
og.story.quests(world: World): readonly QuestView[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly QuestView[]`.

Binding symbol `og_story_quests` · publish builds: kept

```ogt
fn quests {
  doc "Every started quest, in the order they started (a quest log)."
  param world World "The world."
  returns List.QuestView
}
```

<sub>`api.ogt` line 14876</sub>
