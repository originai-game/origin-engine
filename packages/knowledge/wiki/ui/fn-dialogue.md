<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-dialogue"></a>

### `og.ui.dialogue`

A dialogue box: a portrait slot, the speaker in primary ink and the line typed out by the tick. A press or Activate while typing shows the whole line; once shown it answers advanced. The line is `<id>/Body/Line` and the probe reads what is shown.

Also known as: dialogue box, speech box, visual novel text box, subtitles box

```ts
og.ui.dialogue(id: string, desc?: DialogueDesc): DialogueResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `DialogueDesc` | yes | Speaker, text, portrait and speed. |

**Returns** `DialogueResult`.

Binding symbol `og_ui_dialogue` · publish builds: kept

```ogt
fn dialogue {
  doc "A dialogue box: a portrait slot, the speaker in primary ink and the line typed out by the tick. A press or Activate while typing shows the whole line; once shown it answers advanced. The line is `<id>/Body/Line` and the probe reads what is shown."
  aliases "dialogue box, speech box, visual novel text box, subtitles box"
  param id String "Its id."
  param desc DialogueDesc "Speaker, text, portrait and speed."
  returns DialogueResult
}
```

<sub>`api.ogt` line 17283</sub>
