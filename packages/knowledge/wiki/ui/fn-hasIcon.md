<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-hasIcon"></a>

### `og.ui.hasIcon`

Whether the icon atlas has an icon of this name (components draw a fallback otherwise).

```ts
og.ui.hasIcon(name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The icon's name. |

**Returns** `boolean`.

Binding symbol `og_ui_has_icon` · publish builds: kept

```ogt
fn hasIcon {
  doc "Whether the icon atlas has an icon of this name (components draw a fallback otherwise)."
  param name String "The icon's name."
  returns bool
}
```

<sub>Module `ui` line 1009</sub>
