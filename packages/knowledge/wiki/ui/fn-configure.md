<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-configure"></a>

### `og.ui.configure`

Set how the UI looks (the scheme, density, fonts, style, sounds, motion…), typically in GameDesc.start; it takes effect from the next tick. The desc is a partial: a field left out keeps the last UiDesc's value, the one an earlier call or a look's style (og.render.applyLook) gave it, so a settings screen that configures sounds or reducedMotion keeps the style and the fonts; null clears a field to its default (`style: null` goes back to the default style). A UI left with no font at all (no font, no style with fonts, no fallback or locale font) warns: it would draw no text.

```ts
og.ui.configure(world: World, desc?: UiDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI is configured. |
| `desc` | `UiDesc` | yes | The fields to change; the others keep their values. |

**Returns** `Result<void>`: Refused for a dead font or style, or a desc that is not an object.

Binding symbol `og_ui_configure` · publish builds: kept

```ogt
fn configure {
  doc "Set how the UI looks (the scheme, density, fonts, style, sounds, motion…), typically in GameDesc.start; it takes effect from the next tick. The desc is a partial: a field left out keeps the last UiDesc's value, the one an earlier call or a look's style (og.render.applyLook) gave it, so a settings screen that configures sounds or reducedMotion keeps the style and the fonts; null clears a field to its default (`style: null` goes back to the default style). A UI left with no font at all (no font, no style with fonts, no fallback or locale font) warns: it would draw no text."
  param world World "The world whose UI is configured."
  param desc UiDesc "The fields to change; the others keep their values."
  returns Result.Void "Refused for a dead font or style, or a desc that is not an object."
}
```

<sub>Module `ui` line 299</sub>
