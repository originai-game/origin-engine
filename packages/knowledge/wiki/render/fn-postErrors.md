<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-postErrors"></a>

### `og.render.postErrors`

The custom post passes PostEffects.custom names that failed to compile on this device, each skipped (the frame draws as without it). The cook validates every pass for each back end, so this is a device's own refusal.

```ts
og.render.postErrors(world: World): readonly PostError[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |

**Returns** `readonly PostError[]`.

Binding symbol `og_render_post_errors` · publish builds: kept

```ogt
fn postErrors {
  doc "The custom post passes PostEffects.custom names that failed to compile on this device, each skipped (the frame draws as without it). The cook validates every pass for each back end, so this is a device's own refusal."
  param world World "The world asked."
  returns List.PostError
}
```

<sub>Module `render` line 3842</sub>
