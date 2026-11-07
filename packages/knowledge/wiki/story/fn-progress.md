<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-progress"></a>

### `og.story.progress`

Count toward an objective of an active quest's current stage (a wolf killed, a herb picked); the stage advances when its objectives are done.

```ts
og.story.progress(world: World, quest: string, objective: string, amount?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `quest` | `string` | no | The quest's id. |
| `objective` | `string` | no | The objective's id. |
| `amount` | `number` | yes | How many (default 1). |

**Returns** `boolean`: False when the quest is not active or its stage has no such objective.

Binding symbol `og_story_progress` · publish builds: kept

```ogt
fn progress {
  doc "Count toward an objective of an active quest's current stage (a wolf killed, a herb picked); the stage advances when its objectives are done."
  param world World "The world."
  param quest String "The quest's id."
  param objective String "The objective's id."
  param amount u32 "How many (default 1)." {
    optional true
  }
  returns bool "False when the quest is not active or its stage has no such objective."
}
```

<sub>Module `story` line 393</sub>
