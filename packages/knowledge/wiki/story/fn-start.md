<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-start"></a>

### `og.story.start`

Start a conversation: it runs to its first line (or its choices) now, and og.DialogueShown is sent the next tick. Several may run at once.

```ts
og.story.start(world: World, desc?: StartDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `StartDesc` | yes | The dialogue, and optionally the node, cast and starting variables. |

**Returns** `Result<number>`: The conversation, or why not: an unloaded dialogue, an unknown node, speaker or variable.

Binding symbol `og_story_start` · publish builds: kept

```ogt
fn start {
  doc "Start a conversation: it runs to its first line (or its choices) now, and og.DialogueShown is sent the next tick. Several may run at once."
  param world World "The world."
  param desc StartDesc "The dialogue, and optionally the node, cast and starting variables."
  returns Result.u32 "The conversation, or why not: an unloaded dialogue, an unknown node, speaker or variable."
}
```

<sub>`api.ogt` line 14661</sub>
