<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-image"></a>

### `og.ui.image`

A texture region, or a solid rectangle; with ImageDesc.aspect, a fitting side follows the other.

```ts
og.ui.image(id: string, desc?: ImageDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ImageDesc` | yes | Texture, region, tint, size and aspect ratio. |

**Returns** `void`.

Binding symbol `og_ui_image` · publish builds: kept

```ogt
fn image {
  doc "A texture region, or a solid rectangle; with ImageDesc.aspect, a fitting side follows the other."
  param id String "Its id."
  param desc ImageDesc "Texture, region, tint, size and aspect ratio."
  returns Void
}
```

<sub>Module `ui` line 677</sub>
