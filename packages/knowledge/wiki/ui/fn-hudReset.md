<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-hudReset"></a>

### `og.ui.hudReset`

Puts an element back where it was laid out: its key, empty for the selected one, `*` for every one.

```ts
og.ui.hudReset(key: string): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `key` | `string` | no | Its key, empty or `*`. |

**Returns** `void`.

Binding symbol `og_ui_hud_reset` · publish builds: kept

```ogt
fn hudReset {
  doc "Puts an element back where it was laid out: its key, empty for the selected one, `*` for every one."
  param key String "Its key, empty or `*`."
  returns Void
}
```

<sub>Module `ui` line 1615</sub>
