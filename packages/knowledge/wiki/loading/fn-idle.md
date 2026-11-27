<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`loading`](../loading.md) · [index](../index.md)

<a id="fn-idle"></a>

### `og.loading.idle`

Whether the renderer has nothing left that would make a frame draw less than the scene: no pipelines compiling, no textures waiting to upload, no font faces fetched for the locale, no streamed model still fading in (og.render.reveal). True on a host that makes them at once (natively, and without a renderer).

```ts
og.loading.idle(world: World): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `boolean`.

Binding symbol `og_loading_idle` · publish builds: kept

```ogt
fn idle {
  doc "Whether the renderer has nothing left that would make a frame draw less than the scene: no pipelines compiling, no textures waiting to upload, no font faces fetched for the locale, no streamed model still fading in (og.render.reveal). True on a host that makes them at once (natively, and without a renderer)."
  param world World "The world."
  returns bool
}
```

<sub>Module `loading` line 48</sub>
