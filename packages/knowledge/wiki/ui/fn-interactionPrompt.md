<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-interactionPrompt"></a>

### `og.ui.interactionPrompt`

An interaction prompt to the bar of The Last of Us, Elden Ring and Genshin Impact: the action's glyph for the device used last (a key cap, a pad's face button) and the verb, the thing's name over it, a ring filling round the glyph for a hold and a pop when it completes, dimmed with its reason when it cannot be used; it springs in when it appears and stays upright and readable at any distance it shows.

Parts: InteractionPrompt.frame[disabled], InteractionPrompt.glyph, InteractionPrompt.hold, InteractionPrompt.verb, InteractionPrompt.name, InteractionPrompt.reason. States: group use (ready, holding, disabled). Accessibility: button named the verb and name, value the glyph's text. Probe ids: <id>, <id>/Glyph, <id>/Verb.

```ts
og.ui.interactionPrompt(world: World, id: string, desc?: InteractionPromptDesc): AnchorInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world (for the glyph). |
| `id` | `string` | no | Its id. |
| `desc` | `InteractionPromptDesc` | yes | The action and the thing. |

**Returns** `AnchorInfo`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn interactionPrompt {
  doc "An interaction prompt to the bar of The Last of Us, Elden Ring and Genshin Impact: the action's glyph for the device used last (a key cap, a pad's face button) and the verb, the thing's name over it, a ring filling round the glyph for a hold and a pop when it completes, dimmed with its reason when it cannot be used; it springs in when it appears and stays upright and readable at any distance it shows."
  doc "Parts: InteractionPrompt.frame[disabled], InteractionPrompt.glyph, InteractionPrompt.hold, InteractionPrompt.verb, InteractionPrompt.name, InteractionPrompt.reason. States: group use (ready, holding, disabled). Accessibility: button named the verb and name, value the glyph's text. Probe ids: <id>, <id>/Glyph, <id>/Verb."
  host false
  param world World "The world (for the glyph)."
  param id String "Its id."
  param desc InteractionPromptDesc "The action and the thing."
  returns AnchorInfo
}
```

<sub>Module `ui` line 6495</sub>
