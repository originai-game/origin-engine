<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-TouchZone"></a>

### `TouchZone` (desc struct)

Where on the play surface a touch binding's touches must start, in fractions of it from the top left; a zone of no size is all of it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | 0 … 1 | `ratio` | — | Its left edge. |
| `y` | `number` | `0` | 0 … 1 | `ratio` | — | Its top edge. |
| `width` | `number` | `0` | 0 … 1 | `ratio` | — | Its width (0: the whole surface). |
| `height` | `number` | `0` | 0 … 1 | `ratio` | — | Its height (0: the whole surface). |

```ogt
struct TouchZone {
  doc "Where on the play surface a touch binding's touches must start, in fractions of it from the top left; a zone of no size is all of it."
  desc true
  field x f32 "Its left edge." {
    min 0
    max 1
    unit ratio
  }
  field y f32 "Its top edge." {
    min 0
    max 1
    unit ratio
  }
  field width f32 "Its width (0: the whole surface)." {
    min 0
    max 1
    unit ratio
  }
  field height f32 "Its height (0: the whole surface)." {
    min 0
    max 1
    unit ratio
  }
}
```

<sub>Module `input` line 68</sub>
