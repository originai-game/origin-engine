<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-stop"></a>

### `og.story.stop`

End a conversation now (og.DialogueEnded if it was running) and forget it. An ended conversation stays viewable until it is stopped.

```ts
og.story.stop(world: World, conversation: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `conversation` | `number` | no | The conversation. |

**Returns** `boolean`: False for an unknown conversation.

Binding symbol `og_story_stop` · publish builds: kept

```ogt
fn stop {
  doc "End a conversation now (og.DialogueEnded if it was running) and forget it. An ended conversation stays viewable until it is stopped."
  param world World "The world."
  param conversation u32 "The conversation."
  returns bool "False for an unknown conversation."
}
```

<sub>`api.ogt` line 14773</sub>
