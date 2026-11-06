<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MenuChoice"></a>

### `MenuChoice` (struct)

What a menu answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `chosen` | `string \| null` | `null` | — | — | — | The id of the entry chosen: close it. |
| `dismissed` | `boolean` | `false` | — | — | — | It asked to close (a choice, Back, a press outside). |

```ogt
struct MenuChoice {
  doc "What a menu answered this tick."
  field chosen Option.String "The id of the entry chosen: close it."
  field dismissed bool "It asked to close (a choice, Back, a press outside)."
}
```

<sub>`api.ogt` line 17967</sub>
