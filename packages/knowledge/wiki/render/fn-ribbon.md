<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-ribbon"></a>

### `og.render.ribbon`

Draw a ribbon this frame. Systems and commands may call it; the list is cleared at every tick.

```ts
og.render.ribbon(world: World, desc?: RibbonDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `desc` | `RibbonDesc` | yes | The points and the look. |

**Returns** `Result<void>`: Refused for a dead texture or fewer than two points.

Binding symbol `og_render_ribbon` · publish builds: kept

```ogt
fn ribbon {
  doc "Draw a ribbon this frame. Systems and commands may call it; the list is cleared at every tick."
  param world World "The world whose renderer draws it."
  param desc RibbonDesc "The points and the look."
  returns Result.Void "Refused for a dead texture or fewer than two points."
}
```

<sub>Module `render` line 2156</sub>
