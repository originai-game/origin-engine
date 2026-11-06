<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-starBar"></a>

### `og.ui.starBar`

A level's score toward its stars (Candy Crush, Homescapes, Angry Birds): the score in the display face over a bar that fills in a warm gold, a star standing on the bar at each threshold, lit as the score passes it with a pop and the success sound, an outline until then. The bar runs start to end, mirrored right to left.

Parts: StarBar.frame[compact], StarBar.label, StarBar.value, StarBar.track, StarBar.fill, StarBar.star[lit]. States: on each star, group lit (off, on). Motion: the fill eases to the score on the ui spring; a star crossed pops on the emphasis spring (still under reduced motion). Sounds: success on each star crossed. Accessibility: a meter named by label with the score as its value and its stars as its description. Probe ids: <id>/Bar/Track, <id>/Bar/Stars/Star#i, <id>/Value.

```ts
og.ui.starBar(id: string, desc?: StarBarDesc): StarBarResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `StarBarDesc` | yes | The score and its thresholds. |

**Returns** `StarBarResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn starBar {
  doc "A level's score toward its stars (Candy Crush, Homescapes, Angry Birds): the score in the display face over a bar that fills in a warm gold, a star standing on the bar at each threshold, lit as the score passes it with a pop and the success sound, an outline until then. The bar runs start to end, mirrored right to left."
  doc "Parts: StarBar.frame[compact], StarBar.label, StarBar.value, StarBar.track, StarBar.fill, StarBar.star[lit]. States: on each star, group lit (off, on). Motion: the fill eases to the score on the ui spring; a star crossed pops on the emphasis spring (still under reduced motion). Sounds: success on each star crossed. Accessibility: a meter named by label with the score as its value and its stars as its description. Probe ids: <id>/Bar/Track, <id>/Bar/Stars/Star#i, <id>/Value."
  host false
  param id String "Its id."
  param desc StarBarDesc "The score and its thresholds."
  returns StarBarResult
}
```

<sub>`api.ogt` line 20013</sub>
