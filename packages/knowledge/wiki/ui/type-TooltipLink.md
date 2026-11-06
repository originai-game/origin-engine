<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TooltipLink"></a>

### `TooltipLink` (desc struct)

What a keyword link in a tooltip's body opens: a nested tooltip of its own.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The `[link=key]` id in the body. |
| `title` | `string` | `""` | — | — | — | Its title. |
| `body` | `string` | `""` | — | — | — | Its body: rich text, with links of its own. |

```ogt
struct TooltipLink {
  doc "What a keyword link in a tooltip's body opens: a nested tooltip of its own."
  desc true
  field key String "The `[link=key]` id in the body."
  field title String "Its title."
  field body String "Its body: rich text, with links of its own."
}
```

<sub>`api.ogt` line 21191</sub>
