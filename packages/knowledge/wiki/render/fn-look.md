<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="fn-look"></a>

### `og.render.look`

The look last applied (by og.render.applyLook or GameDesc.look), as og.load.lookInfo describes it; null before any.

```ts
og.render.look(world: World): LookInfo | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |

**Returns** `LookInfo | null`.

Binding symbol `og_render_look` · publish builds: kept

```ogt
fn look {
  doc "The look last applied (by og.render.applyLook or GameDesc.look), as og.load.lookInfo describes it; null before any."
  param world World "The world asked."
  returns Option.LookInfo
}
```

<sub>Module `render` line 2851</sub>
