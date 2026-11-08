<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-calendar"></a>

### `og.ui.calendar`

A month grid of days.

```ts
og.ui.calendar(id: string, desc?: CalendarDesc): CalendarResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `CalendarDesc` | yes | The month, today, the selection and marks. |

**Returns** `CalendarResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn calendar {
  doc "A month grid of days."
  host false
  param id String "Its id."
  param desc CalendarDesc "The month, today, the selection and marks."
  returns CalendarResult
}
```

<sub>Module `ui` line 2904</sub>
