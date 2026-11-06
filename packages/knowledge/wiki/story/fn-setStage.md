<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-setStage"></a>

### `og.story.setStage`

Move an active quest to a stage, its counts cleared.

```ts
og.story.setStage(world: World, quest: string, stage: string): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `quest` | `string` | no | The quest's id. |
| `stage` | `string` | no | The stage's id. |

**Returns** `Result<boolean>`: False when it is not active or already there. Refused for an unknown quest or stage.

Binding symbol `og_story_set_stage` · publish builds: kept

```ogt
fn setStage {
  doc "Move an active quest to a stage, its counts cleared."
  param world World "The world."
  param quest String "The quest's id."
  param stage String "The stage's id."
  returns Result.bool "False when it is not active or already there. Refused for an unknown quest or stage."
}
```

<sub>`api.ogt` line 14840</sub>
