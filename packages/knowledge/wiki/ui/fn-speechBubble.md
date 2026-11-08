<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-speechBubble"></a>

### `og.ui.speechBubble`

A speech bubble to the bar of Animal Crossing, Sea of Thieves and The Sims: a rounded bubble over the speaker with a tail pointing down to them, the speaker's name, the text typing out with a soft tick of sound, popping in and fading out after a time that grows with its length; a shout jagged and bold, a thought round with its tail as bubbles; it can hold on the edge when the speaker is off screen.

Parts: SpeechBubble.bubble[shout|thought], SpeechBubble.tail, SpeechBubble.speaker, SpeechBubble.text. States: group speech (typing, shown, leaving). Sounds: counterTick as it types. Accessibility: text named the speaker, value the whole text (a screen reader hears it at once). Probe ids: <id>, <id>/Bubble/Text, <id>/Bubble/Speaker.

```ts
og.ui.speechBubble(id: string, desc?: SpeechBubbleDesc): AnchorInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SpeechBubbleDesc` | yes | The words. |

**Returns** `AnchorInfo`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn speechBubble {
  doc "A speech bubble to the bar of Animal Crossing, Sea of Thieves and The Sims: a rounded bubble over the speaker with a tail pointing down to them, the speaker's name, the text typing out with a soft tick of sound, popping in and fading out after a time that grows with its length; a shout jagged and bold, a thought round with its tail as bubbles; it can hold on the edge when the speaker is off screen."
  doc "Parts: SpeechBubble.bubble[shout|thought], SpeechBubble.tail, SpeechBubble.speaker, SpeechBubble.text. States: group speech (typing, shown, leaving). Sounds: counterTick as it types. Accessibility: text named the speaker, value the whole text (a screen reader hears it at once). Probe ids: <id>, <id>/Bubble/Text, <id>/Bubble/Speaker."
  host false
  param id String "Its id."
  param desc SpeechBubbleDesc "The words."
  returns AnchorInfo
}
```

<sub>Module `ui` line 6784</sub>
