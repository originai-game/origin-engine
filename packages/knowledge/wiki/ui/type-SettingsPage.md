<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SettingsPage"></a>

### `SettingsPage` (desc struct)

One page of the settings tree: a tab, and its entries in order.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key: its tab's element id (<id>/Tabs/Row/<key>). |
| `label` | `string` | `""` | — | — | — | Its word. |
| `entries` | `readonly SettingEntry[]` | `[]` | — | — | — | Its entries; empty with hud draws the HUD layout page. |
| `hud` | `boolean` | `false` | — | — | — | The HUD layout page: a note, Edit HUD (which turns og.ui.setHudEdit on) and, while editing, og.ui.hudLayoutEditor itself. |

```ogt
struct SettingsPage {
  doc "One page of the settings tree: a tab, and its entries in order."
  desc true
  field key String "Its key: its tab's element id (<id>/Tabs/Row/<key>)."
  field label String "Its word."
  field entries List.SettingEntry "Its entries; empty with hud draws the HUD layout page."
  field hud bool "The HUD layout page: a note, Edit HUD (which turns og.ui.setHudEdit on) and, while editing, og.ui.hudLayoutEditor itself."
}
```

<sub>`api.ogt` line 20010</sub>
