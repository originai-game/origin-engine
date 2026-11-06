<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="fn-view"></a>

### `og.story.view`

What a conversation shows now, resolved in the active locale.

Also known as: dialogue line, conversation, dialog box, Yarn Spinner, Ink, line and choices

```ts
og.story.view(world: World, conversation: number): DialogueView | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `conversation` | `number` | no | The conversation. |

**Returns** `DialogueView | null`: Null for a conversation that does not exist (never started, or stopped).

Binding symbol `og_story_view` · publish builds: kept

```ogt
fn view {
  doc "What a conversation shows now, resolved in the active locale."
  aliases "dialogue line, conversation, dialog box, Yarn Spinner, Ink, line and choices"
  param world World "The world."
  param conversation u32 "The conversation."
  returns Option.DialogueView "Null for a conversation that does not exist (never started, or stopped)."
}
```

<sub>`api.ogt` line 14722</sub>
