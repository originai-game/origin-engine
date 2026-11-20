<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`loading`](../loading.md) · [index](../index.md)

<a id="fn-stage"></a>

### `og.loading.stage`

Report a stage of the game's loading: the web page sends it to the page as `og:stage` (and to the framing page), and the engine's splash shows its label and fraction. Presentation only: nothing a tick reads changes. Systems may call it.

See `loading §13`.

```ts
og.loading.stage(world: World, desc?: LoadingStageDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `LoadingStageDesc` | yes | The stage, its fraction and its label. |

**Returns** `Result<void>`: Refused for a fraction outside 0 to 1.

Binding symbol `og_loading_stage` · publish builds: kept

```ogt
fn stage {
  doc "Report a stage of the game's loading: the web page sends it to the page as `og:stage` (and to the framing page), and the engine's splash shows its label and fraction. Presentation only: nothing a tick reads changes. Systems may call it."
  guide "loading §13"
  param world World "The world."
  param desc LoadingStageDesc "The stage, its fraction and its label."
  returns Result.Void "Refused for a fraction outside 0 to 1."
}
```

<sub>Module `loading` line 36</sub>
