<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-icon"></a>

### `og.ui.icon`

An icon of the atlas (UiDesc.icons) by name: a texture cell, or an icon set's distance-field glyph fitted to its square; an unknown name draws an empty slot and warns. Rich text takes the same icons inline with `[icon=name]`. The probe reads the name.

```ts
og.ui.icon(id: string, name: string, desc?: IconDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `name` | `string` | no | The icon's name. |
| `desc` | `IconDesc` | yes | Size and ink. |

**Returns** `void`.

Binding symbol `og_ui_icon` · publish builds: kept

```ogt
fn icon {
  doc "An icon of the atlas (UiDesc.icons) by name: a texture cell, or an icon set's distance-field glyph fitted to its square; an unknown name draws an empty slot and warns. Rich text takes the same icons inline with `[icon=name]`. The probe reads the name."
  param id String "Its id."
  param name String "The icon's name."
  param desc IconDesc "Size and ink."
  returns Void
}
```

<sub>Module `ui` line 1611</sub>
