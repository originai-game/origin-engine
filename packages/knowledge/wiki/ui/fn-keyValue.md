<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-keyValue"></a>

### `og.ui.keyValue`

A key (inkMuted) and its value (ink, at the end) on one row: stats, settings summaries, receipts.

```ts
og.ui.keyValue(id: string, key: string, value: string, mono: boolean): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `key` | `string` | no | The key. |
| `value` | `string` | no | The value. |
| `mono` | `boolean` | no | The value in the monospaced font (prices, ids). |

**Returns** `void`.

Binding symbol `og_ui_key_value` · publish builds: kept

```ogt
fn keyValue {
  doc "A key (inkMuted) and its value (ink, at the end) on one row: stats, settings summaries, receipts."
  param id String "Its id."
  param key String "The key."
  param value String "The value."
  param mono bool "The value in the monospaced font (prices, ids)."
  returns Void
}
```

<sub>`api.ogt` line 17140</sub>
