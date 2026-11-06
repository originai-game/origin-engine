<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="type-AssertUncovered"></a>

### `AssertUncovered` (struct)

A sometimes, reachable or each bucket that never fired.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `AssertKind` | `"always"` | — | — | — | The site's kind. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `location` | `string` | `""` | — | — | — | Its place. |
| `key` | `string` | `""` | — | — | optional | An each's bucket that never held. |
| `ran` | `boolean` | `false` | — | — | optional | Whether it was called at all (false: the run never got there). |

```ogt
struct AssertUncovered {
  doc "A sometimes, reachable or each bucket that never fired."
  field kind AssertKind "The site's kind."
  field name String "Its name."
  field location String "Its place."
  field key String "An each's bucket that never held." {
    optional true
  }
  field ran bool "Whether it was called at all (false: the run never got there)." {
    optional true
  }
}
```

<sub>`api.ogt` line 25910</sub>
