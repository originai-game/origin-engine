<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-ProductsDesc"></a>

### `ProductsDesc` (desc struct)

Which products to describe.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ids` | `readonly string[]` | `[]` | — | — | — | Product ids as the game's store lists them (1 to 64 of letters, digits, `.`, `_` and `-`), at most 100; empty: every product the game sells. |

```ogt
struct ProductsDesc {
  doc "Which products to describe."
  desc true
  field ids List.String "Product ids as the game's store lists them (1 to 64 of letters, digits, `.`, `_` and `-`), at most 100; empty: every product the game sells."
}
```

<sub>Module `platform` line 255</sub>
