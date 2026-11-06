<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-floatingText"></a>

### `og.render.floatingText`

Spawn a floating text (damage numbers, +coins, `blocked`). It lives `lifetime` seconds of scaled time on its own; systems and commands may call it.

```ts
og.render.floatingText(world: World, desc?: FloatingTextDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer draws it. |
| `desc` | `FloatingTextDesc` | yes | The text, the point and the style. |

**Returns** `Result<void>`: Refused for a dead font, or no font when the UI has none.

Binding symbol `og_render_floating_text` · publish builds: kept

```ogt
fn floatingText {
  doc "Spawn a floating text (damage numbers, +coins, `blocked`). It lives `lifetime` seconds of scaled time on its own; systems and commands may call it."
  param world World "The world whose renderer draws it."
  param desc FloatingTextDesc "The text, the point and the style."
  returns Result.Void "Refused for a dead font, or no font when the UI has none."
}
```

<sub>Module `render` line 2469</sub>
