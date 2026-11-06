<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-mark"></a>

### `og.story.mark`

Point a quest at an entity or a point (the map, compass and tracker read QuestView.marker), over its objectives' own markers.

```ts
og.story.mark(world: World, id: string, desc?: MarkerDesc): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `id` | `string` | no | The quest's id. |
| `desc` | `MarkerDesc` | yes | The entity or point; both zero clears it. |

**Returns** `boolean`: False for a quest not started.

Binding symbol `og_story_mark` · publish builds: kept

```ogt
fn mark {
  doc "Point a quest at an entity or a point (the map, compass and tracker read QuestView.marker), over its objectives' own markers."
  param world World "The world."
  param id String "The quest's id."
  param desc MarkerDesc "The entity or point; both zero clears it."
  returns bool "False for a quest not started."
}
```

<sub>Module `story` line 433</sub>
