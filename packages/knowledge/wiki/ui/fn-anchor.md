<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-anchor"></a>

### `og.ui.anchor`

An anchored group built by `build`: a layer at a point of the safe area, over the tree, for HUD corners and touch controls.

```ts
og.ui.anchor(id: string, desc: AnchorDesc, build: UiBuild): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `AnchorDesc` | no | Where it sits and how it lays out. |
| `build` | `UiBuild` | no | Makes its children. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn anchor {
  doc "An anchored group built by `build`: a layer at a point of the safe area, over the tree, for HUD corners and touch controls."
  host false
  param id String "Its id."
  param desc AnchorDesc "Where it sits and how it lays out."
  param build UiBuild "Makes its children."
  returns Void
}
```

<sub>Module `ui` line 972</sub>
