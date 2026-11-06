<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PaginationDesc"></a>

### `PaginationDesc` (desc struct)

Previous and next, the first and last pages and the pages around the current one.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pages` | `number` | `1` | — | — | — | How many pages. |
| `page` | `number` | `0` | — | — | — | The current page, from 0 (keep PageResult.page). |
| `siblings` | `number \| null` | `null` | — | — | — | Pages shown each side of the current one; null is 1. |
| `compact` | `boolean \| null` | `null` | — | — | — | Only `4 / 20` between the arrows; null is a phone's. |
| `bumpers` | `boolean` | `false` | — | — | — | The tabPrev and tabNext actions page. |
| `prevLabel` | `string` | `""` | — | — | — | Previous's accessible name; empty is Previous page. |
| `nextLabel` | `string` | `""` | — | — | — | Next's accessible name; empty is Next page. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it; empty is Pagination. |

```ogt
struct PaginationDesc {
  doc "Previous and next, the first and last pages and the pages around the current one."
  desc true
  field pages u32 "How many pages." {
    default 1
  }
  field page u32 "The current page, from 0 (keep PageResult.page)."
  field siblings Option.u32 "Pages shown each side of the current one; null is 1."
  field compact Option.bool "Only `4 / 20` between the arrows; null is a phone's."
  field bumpers bool "The tabPrev and tabNext actions page."
  field prevLabel String "Previous's accessible name; empty is Previous page."
  field nextLabel String "Next's accessible name; empty is Next page."
  field width Option.Size "Its width; null fits."
  field label String "What a screen reader calls it; empty is Pagination."
}
```

<sub>`api.ogt` line 18110</sub>
