<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-startQuest"></a>

### `og.story.startQuest`

Start a loaded quest at its first stage.

Also known as: begin quest, accept quest, quest start

```ts
og.story.startQuest(world: World, id: string): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `id` | `string` | no | The quest's id. |

**Returns** `Result<boolean>`: False when it had started before. Refused for a quest not loaded.

Binding symbol `og_story_start_quest` · publish builds: kept

```ogt
fn startQuest {
  doc "Start a loaded quest at its first stage."
  aliases "begin quest, accept quest, quest start"
  param world World "The world."
  param id String "The quest's id."
  returns Result.bool "False when it had started before. Refused for a quest not loaded."
}
```

<sub>Module `story` line 386</sub>
