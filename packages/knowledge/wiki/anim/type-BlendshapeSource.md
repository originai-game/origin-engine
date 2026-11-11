<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-BlendshapeSource"></a>

### `BlendshapeSource` (enum)

Who decides blendshape weights.

A string: one of the variants below. The zero value is `"component"`.

| Variant | Doc |
|---|---|
| `"component"` | Game code writes w0...w7. |
| `"clips"` | The playing clips' weight channels write them. |

```ogt
enum BlendshapeSource {
  doc "Who decides blendshape weights."
  variant component "Game code writes w0...w7."
  variant clips "The playing clips' weight channels write them."
}
```

<sub>Module `anim` line 46</sub>
