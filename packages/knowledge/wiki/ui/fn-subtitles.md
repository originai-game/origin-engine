<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-subtitles"></a>

### `og.ui.subtitles`

Subtitles and captions to the Xbox Accessibility Guidelines' 104: centred at the bottom of the safe area, at most two lines of at most about 42 characters each across 70 % of the width (the box wraps by words, never by letters), in the player's size (never under 18 px at 1080p), on a box of the player's opacity (an outline when it is clear), with the speaker's name in their colour, sound captions in brackets above speech, an arrow toward a speaker off screen, new lines fading in quickly (never sliding: text that moves is harder to read).

Parts: Subtitles.box, Subtitles.line, Subtitles.speaker, Subtitles.text, Subtitles.caption, Subtitles.arrow. Accessibility: a log whose new lines are read, each line text named speaker: text. Probe ids: <id>, <id>/<key>, its Speaker, Text, Arrow.

```ts
og.ui.subtitles(id: string, desc?: SubtitlesDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SubtitlesDesc` | yes | The lines and the settings. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn subtitles {
  doc "Subtitles and captions to the Xbox Accessibility Guidelines' 104: centred at the bottom of the safe area, at most two lines of at most about 42 characters each across 70 % of the width (the box wraps by words, never by letters), in the player's size (never under 18 px at 1080p), on a box of the player's opacity (an outline when it is clear), with the speaker's name in their colour, sound captions in brackets above speech, an arrow toward a speaker off screen, new lines fading in quickly (never sliding: text that moves is harder to read)."
  doc "Parts: Subtitles.box, Subtitles.line, Subtitles.speaker, Subtitles.text, Subtitles.caption, Subtitles.arrow. Accessibility: a log whose new lines are read, each line text named speaker: text. Probe ids: <id>, <id>/<key>, its Speaker, Text, Arrow."
  host false
  param id String "Its id."
  param desc SubtitlesDesc "The lines and the settings."
  returns Void
}
```

<sub>Module `ui` line 5753</sub>
