<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-capabilities"></a>

### `og.render.capabilities`

The renderer's tier and limits on this device.

```ts
og.render.capabilities(world: World): RenderCapabilities
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer is asked. |

**Returns** `RenderCapabilities`.

Binding symbol `og_render_capabilities` · publish builds: kept

```ogt
fn capabilities {
  doc "The renderer's tier and limits on this device."
  param world World "The world whose renderer is asked."
  returns RenderCapabilities
}
```

<sub>Module `render` line 3123</sub>
