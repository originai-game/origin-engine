<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-history"></a>

### `og.story.history`

A conversation's lines so far, oldest first, with the choice made on each (a VN backlog).

```ts
og.story.history(world: World, conversation: number): readonly StoryLine[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `conversation` | `number` | no | The conversation. |

**Returns** `readonly StoryLine[]`: Empty for an unknown conversation.

Binding symbol `og_story_history` · publish builds: kept

```ogt
fn history {
  doc "A conversation's lines so far, oldest first, with the choice made on each (a VN backlog)."
  param world World "The world."
  param conversation u32 "The conversation."
  returns List.StoryLine "Empty for an unknown conversation."
}
```

<sub>Module `story` line 288</sub>
