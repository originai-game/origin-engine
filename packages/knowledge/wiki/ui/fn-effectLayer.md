<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-effectLayer"></a>

### `og.ui.effectLayer`

A layer (docs/ui.md §3.3): its subtree, built by `build`, drawn offscreen and composited through a material (a card dissolving as it is spent, a shine passing over a rare item, a hologram panel, a hit's glitch, a locked tile in grey), the material's parameters set from one amount. Its children keep their input and accessibility. Part: `Layer.frame`. Probe: `<id>` and its children.

```ts
og.ui.effectLayer(id: string, desc: EffectLayerDesc, build: UiBuild): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `EffectLayerDesc` | no | The material and its amount. |
| `build` | `UiBuild` | no | Builds what it draws. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn effectLayer {
  doc "A layer (docs/ui.md §3.3): its subtree, built by `build`, drawn offscreen and composited through a material (a card dissolving as it is spent, a shine passing over a rare item, a hologram panel, a hit's glitch, a locked tile in grey), the material's parameters set from one amount. Its children keep their input and accessibility. Part: `Layer.frame`. Probe: `<id>` and its children."
  host false
  param id String "Its id."
  param desc EffectLayerDesc "The material and its amount."
  param build UiBuild "Builds what it draws."
  returns Void
}
```

<sub>Module `ui` line 1925</sub>
