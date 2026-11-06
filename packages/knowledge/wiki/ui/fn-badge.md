<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-badge"></a>

### `og.ui.badge`

A badge: a small status tag in its tone.

```ts
og.ui.badge(id: string, text: string, tone: Tone): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `text` | `string` | no | The label (a count, a word). |
| `tone` | `Tone` | no | Its tone. |

**Returns** `void`.

Binding symbol `og_ui_badge` · publish builds: kept

```ogt
fn badge {
  doc "A badge: a small status tag in its tone."
  param id String "Its id."
  param text String "The label (a count, a word)."
  param tone Tone "Its tone."
  returns Void
}
```

<sub>`api.ogt` line 17118</sub>
