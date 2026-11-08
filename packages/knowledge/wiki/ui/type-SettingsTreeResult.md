<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SettingsTreeResult"></a>

### `SettingsTreeResult` (struct)

What the player asked for this tick, for the game to apply to its settings.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `page` | `string \| null` | `null` | — | — | — | Another page's tab was pressed (or the tab actions stepped). |
| `change` | `SettingChange \| null` | `null` | — | — | — | An entry's control changed: the game holds it as pending (or applies it at once). |
| `action` | `string \| null` | `null` | — | — | — | An action entry was pressed. |
| `bindingChanged` | `boolean` | `false` | — | — | — | A binding row rebound an action (the action map is already changed). |
| `apply` | `boolean` | `false` | — | — | — | Apply was pressed: the game applies the pending entries, and starts the countdown if any is a display change. |
| `revert` | `boolean` | `false` | — | — | — | Revert was pressed: the game drops the pending entries. |
| `defaults` | `boolean` | `false` | — | — | — | Defaults was pressed: the game sets the page's entries to their defaults, pending. |
| `keep` | `boolean` | `false` | — | — | — | Keep was pressed during the countdown: the display change stays. |
| `confirmRevert` | `boolean` | `false` | — | — | — | The countdown's Revert was pressed, Back during it, or it reached 0: the game puts the display settings back. |
| `hudDone` | `boolean` | `false` | — | — | — | The HUD editor's Done was pressed. |

```ogt
struct SettingsTreeResult {
  doc "What the player asked for this tick, for the game to apply to its settings."
  field page Option.String "Another page's tab was pressed (or the tab actions stepped)."
  field change Option.SettingChange "An entry's control changed: the game holds it as pending (or applies it at once)."
  field action Option.String "An action entry was pressed."
  field bindingChanged bool "A binding row rebound an action (the action map is already changed)."
  field apply bool "Apply was pressed: the game applies the pending entries, and starts the countdown if any is a display change."
  field revert bool "Revert was pressed: the game drops the pending entries."
  field defaults bool "Defaults was pressed: the game sets the page's entries to their defaults, pending."
  field keep bool "Keep was pressed during the countdown: the display change stays."
  field confirmRevert bool "The countdown's Revert was pressed, Back during it, or it reached 0: the game puts the display settings back."
  field hudDone bool "The HUD editor's Done was pressed."
}
```

<sub>Module `ui` line 4299</sub>
