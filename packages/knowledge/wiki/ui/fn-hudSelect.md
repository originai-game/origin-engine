<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-hudSelect"></a>

### `og.ui.hudSelect`

Selects an element to edit by its key (empty: none).

```ts
og.ui.hudSelect(key: string): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `key` | `string` | no | Its key. |

**Returns** `void`.

Binding symbol `og_ui_hud_select` · publish builds: kept

```ogt
fn hudSelect {
  doc "Selects an element to edit by its key (empty: none)."
  param key String "Its key."
  returns Void
}
```

<sub>Module `ui` line 1657</sub>
