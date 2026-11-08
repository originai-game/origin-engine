<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SpaceStep"></a>

### `SpaceStep` (enum)

A step of the style's spacing scale.

A string: one of the variants below. The zero value is `"md"`.

| Variant | Doc |
|---|---|
| `"md"` | The middle step (the default). |
| `"xxs"` | The smallest. |
| `"xs"` | Extra small. |
| `"sm"` | Small. |
| `"lg"` | Large. |
| `"xl"` | Extra large. |
| `"xxl"` | Twice extra large. |
| `"xl3"` | The largest. |

```ogt
enum SpaceStep {
  doc "A step of the style's spacing scale."
  variant md "The middle step (the default)."
  variant xxs "The smallest."
  variant xs "Extra small."
  variant sm "Small."
  variant lg "Large."
  variant xl "Extra large."
  variant xxl "Twice extra large."
  variant xl3 "The largest."
}
```

<sub>Module `ui` line 2764</sub>
