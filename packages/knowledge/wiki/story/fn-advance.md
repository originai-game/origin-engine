<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-advance"></a>

### `og.story.advance`

Move a conversation past its line: it runs to the next line, its choices or its end.

```ts
og.story.advance(world: World, conversation: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `conversation` | `number` | no | The conversation. |

**Returns** `Result<void>`: Refused while it waits on choices, after it ended, or for an unknown conversation.

Binding symbol `og_story_advance` · publish builds: kept

```ogt
fn advance {
  doc "Move a conversation past its line: it runs to the next line, its choices or its end."
  param world World "The world."
  param conversation u32 "The conversation."
  returns Result.Void "Refused while it waits on choices, after it ended, or for an unknown conversation."
}
```

<sub>`api.ogt` line 14757</sub>
