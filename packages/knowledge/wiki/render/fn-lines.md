<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-lines"></a>

### `og.render.lines`

Draw lines this frame. Systems and commands may call it; the list is cleared at every tick, so submit them every tick they should show (og.Lines keeps a static set).

```ts
og.render.lines(world: World, desc?: LinesDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws them. |
| `desc` | `LinesDesc` | yes | The points and the look. |

**Returns** `Result<void>`: Refused for fewer than two points or a colours list whose length is not the points'.

Binding symbol `og_render_lines` · publish builds: kept

```ogt
fn lines {
  doc "Draw lines this frame. Systems and commands may call it; the list is cleared at every tick, so submit them every tick they should show (og.Lines keeps a static set)."
  param world World "The world whose renderer draws them."
  param desc LinesDesc "The points and the look."
  returns Result.Void "Refused for fewer than two points or a colours list whose length is not the points'."
}
```

<sub>Module `render` line 3519</sub>
