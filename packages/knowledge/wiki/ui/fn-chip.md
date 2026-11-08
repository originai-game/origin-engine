<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-chip"></a>

### `og.ui.chip`

A chip: a control-filled tag; selected inverts to ink. For filters and tags.

```ts
og.ui.chip(id: string, text: string, desc?: ChipDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `text` | `string` | no | The label. |
| `desc` | `ChipDesc` | yes | Selected and disabled. |

**Returns** `Response`.

Binding symbol `og_ui_chip` · publish builds: kept

```ogt
fn chip {
  doc "A chip: a control-filled tag; selected inverts to ink. For filters and tags."
  param id String "Its id."
  param text String "The label."
  param desc ChipDesc "Selected and disabled."
  returns Response
}
```

<sub>Module `ui` line 1365</sub>
