<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-measureText"></a>

### `og.render.measureText`

The size text takes when og.render.text draws it, for centring, wrapping and layout.

```ts
og.render.measureText(world: World, desc?: TextMeasureDesc): Result<TextMetrics>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns the font. |
| `desc` | `TextMeasureDesc` | yes | The font, text and size. |

**Returns** `Result<TextMetrics>`: Refused for a dead font.

Binding symbol `og_render_measure_text` · publish builds: kept

```ogt
fn measureText {
  doc "The size text takes when og.render.text draws it, for centring, wrapping and layout."
  param world World "The world whose renderer owns the font."
  param desc TextMeasureDesc "The font, text and size."
  returns Result.TextMetrics "Refused for a dead font."
}
```

<sub>Module `render` line 2486</sub>
