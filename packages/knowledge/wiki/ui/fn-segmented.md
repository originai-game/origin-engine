<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-segmented"></a>

### `og.ui.segmented`

A segmented control (segmented tabs): a track whose raised thumb slides to the chosen segment and takes its width on the style's spring (layout animation; it jumps under reduced motion). A press chooses; Left and Right step the choice while it has the focus, skipping disabled segments, mirrored right to left. The segments share the width they are given, never narrower than their labels: when they do not fit, the track scrolls sideways under a fade at the cut edge. Under a finger each segment is at least 44 px tall. Plays tab. Parts: `Segmented.track[focused|disabled]`, `Segmented.thumb`, `Segmented.item[selected][hover|pressed|disabled]`. Role radiogroup of radios. Probe: `<id>`, `<id>/Strip/Track/Item#i`, `<id>/Strip/Track/Thumb`.

Also known as: tabs, segmented tabs, tab bar

```ts
og.ui.segmented(id: string, desc?: SegmentedDesc): ChoiceIndexResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SegmentedDesc` | yes | Its segments. |

**Returns** `ChoiceIndexResult`.

Binding symbol `og_ui_segmented` · publish builds: kept

```ogt
fn segmented {
  doc "A segmented control (segmented tabs): a track whose raised thumb slides to the chosen segment and takes its width on the style's spring (layout animation; it jumps under reduced motion). A press chooses; Left and Right step the choice while it has the focus, skipping disabled segments, mirrored right to left. The segments share the width they are given, never narrower than their labels: when they do not fit, the track scrolls sideways under a fade at the cut edge. Under a finger each segment is at least 44 px tall. Plays tab. Parts: `Segmented.track[focused|disabled]`, `Segmented.thumb`, `Segmented.item[selected][hover|pressed|disabled]`. Role radiogroup of radios. Probe: `<id>`, `<id>/Strip/Track/Item#i`, `<id>/Strip/Track/Thumb`."
  aliases "tabs, segmented tabs, tab bar"
  param id String "Its id."
  param desc SegmentedDesc "Its segments."
  returns ChoiceIndexResult
}
```

<sub>Module `ui` line 3669</sub>
