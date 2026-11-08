<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BreadcrumbsDesc"></a>

### `BreadcrumbsDesc` (desc struct)

The path to where the player is; every crumb but the last is a link back.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `items` | `readonly Crumb[]` | `[]` | — | — | — | The crumbs, the current page last. |
| `maxShown` | `number \| null` | `null` | — | — | — | The most crumbs shown before the middle collapses into a menu; null is 3 on a phone, else 5. |
| `moreLabel` | `string` | `""` | — | — | — | The collapsed crumbs' button's accessible name; empty is Show path. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it; empty is Breadcrumb. |

```ogt
struct BreadcrumbsDesc {
  doc "The path to where the player is; every crumb but the last is a link back."
  desc true
  field items List.Crumb "The crumbs, the current page last."
  field maxShown Option.u32 "The most crumbs shown before the middle collapses into a menu; null is 3 on a phone, else 5."
  field moreLabel String "The collapsed crumbs' button's accessible name; empty is Show path."
  field width Option.Size "Its width; null grows."
  field label String "What a screen reader calls it; empty is Breadcrumb."
}
```

<sub>Module `ui` line 2238</sub>
