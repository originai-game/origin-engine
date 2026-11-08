<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiInlineImage"></a>

### `UiInlineImage` (desc struct)

A picture rich text draws inline by name ([img=<name>]).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | The name markup gives it. |
| `texture` | `Texture \| null` | `null` | — | — | — | The texture, from og.load.texture; null leaves the name unregistered. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The region drawn, [u0, v0, u1, v1] with v down; null is the whole texture. |
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height of the region as drawn; 0 is square. |

```ogt
struct UiInlineImage {
  doc "A picture rich text draws inline by name ([img=<name>])."
  desc true
  field name String "The name markup gives it."
  field texture Option.Texture "The texture, from og.load.texture; null leaves the name unregistered."
  field uv Option.Array.f32.4 "The region drawn, [u0, v0, u1, v1] with v down; null is the whole texture."
  field aspect f32 "Width over height of the region as drawn; 0 is square." {
    min 0
  }
}
```

<sub>Module `ui` line 839</sub>
