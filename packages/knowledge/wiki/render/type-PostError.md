<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-PostError"></a>

### `PostError` (struct)

A custom post pass the device failed to compile.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `post` | `Post` | `0` | — | — | — | The pass. |
| `message` | `string` | `""` | — | — | — | The device's message. |

```ogt
struct PostError {
  doc "A custom post pass the device failed to compile."
  field post Post "The pass."
  field message String "The device's message."
}
```

<sub>Module `render` line 3849</sub>
