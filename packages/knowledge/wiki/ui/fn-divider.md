<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-divider"></a>

### `og.ui.divider`

A hairline in the border colour across its parent.

```ts
og.ui.divider(id: string, vertical: boolean): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `vertical` | `boolean` | no | Stand it up (in a row). |

**Returns** `void`.

Binding symbol `og_ui_divider` · publish builds: kept

```ogt
fn divider {
  doc "A hairline in the border colour across its parent."
  param id String "Its id."
  param vertical bool "Stand it up (in a row)."
  returns Void
}
```

<sub>Module `ui` line 1428</sub>
