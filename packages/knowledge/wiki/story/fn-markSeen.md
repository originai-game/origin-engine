<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-markSeen"></a>

### `og.story.markSeen`

The player looked at a quest: clear QuestView.isNew and updated.

```ts
og.story.markSeen(world: World, id: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `id` | `string` | no | The quest's id. |

**Returns** `boolean`: False for a quest not started.

Binding symbol `og_story_mark_seen` · publish builds: kept

```ogt
fn markSeen {
  doc "The player looked at a quest: clear QuestView.isNew and updated."
  param world World "The world."
  param id String "The quest's id."
  returns bool "False for a quest not started."
}
```

<sub>`api.ogt` line 14882</sub>
