<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-choose"></a>

### `og.story.choose`

Take a choice: its `set`, `emit` and `quest` steps run, an active check rolls (og.DialogueCheck), and the conversation goes to its `goto` (or `fail` on a missed roll), or ends without one.

```ts
og.story.choose(world: World, conversation: number, index: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `conversation` | `number` | no | The conversation. |
| `index` | `number` | no | ChoiceView.index. |

**Returns** `Result<void>`: Refused for a hidden or disabled choice, a conversation showing a line, or an unknown one.

Binding symbol `og_story_choose` · publish builds: kept

```ogt
fn choose {
  doc "Take a choice: its `set`, `emit` and `quest` steps run, an active check rolls (og.DialogueCheck), and the conversation goes to its `goto` (or `fail` on a missed roll), or ends without one."
  param world World "The world."
  param conversation u32 "The conversation."
  param index u32 "ChoiceView.index."
  returns Result.Void "Refused for a hidden or disabled choice, a conversation showing a line, or an unknown one."
}
```

<sub>Module `story` line 308</sub>
