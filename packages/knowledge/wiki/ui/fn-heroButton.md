<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-heroButton"></a>

### `og.ui.heroButton`

The one action a genre screen leads with (End turn, Summon, Play, Ready): a raised face lit from the top over a ledge of its own dark and a soft shadow on the scene, so it reads as a thing to press over any backdrop, beside the flat controls of og.ui.button. It lifts under the pointer and the focus and sinks when pressed; disabled it lies flat. A keycap after the label shows the action's binding on a desktop.

Parts: Pattern.button[primary|premium|secondary|danger][sm|md|lg], Pattern.buttonFace, Pattern.keycap. States: group interaction (normal, hover, pressed, focused, disabled). Motion: the ui token's lift and press. Sounds: the style's press. Accessibility: a button named by its label (and key). Probe id: <id>.

```ts
og.ui.heroButton(id: string, label: string, desc?: HeroButtonDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `label` | `string` | no | What it does. |
| `desc` | `HeroButtonDesc` | yes | Tone, size, keycap, disabled and width. |

**Returns** `Response`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn heroButton {
  doc "The one action a genre screen leads with (End turn, Summon, Play, Ready): a raised face lit from the top over a ledge of its own dark and a soft shadow on the scene, so it reads as a thing to press over any backdrop, beside the flat controls of og.ui.button. It lifts under the pointer and the focus and sinks when pressed; disabled it lies flat. A keycap after the label shows the action's binding on a desktop."
  doc "Parts: Pattern.button[primary|premium|secondary|danger][sm|md|lg], Pattern.buttonFace, Pattern.keycap. States: group interaction (normal, hover, pressed, focused, disabled). Motion: the ui token's lift and press. Sounds: the style's press. Accessibility: a button named by its label (and key). Probe id: <id>."
  host false
  param id String "Its id."
  param label String "What it does."
  param desc HeroButtonDesc "Tone, size, keycap, disabled and width."
  returns Response
}
```

<sub>Module `ui` line 4107</sub>
