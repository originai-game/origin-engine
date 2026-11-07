<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SettingsTreeDesc"></a>

### `SettingsTreeDesc` (desc struct)

The settings as a tree of pages and typed entries, the pending state, and the confirm countdown.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `"Settings"` | — | — | — | The screen's title. |
| `pages` | `readonly SettingsPage[]` | `[]` | — | — | — | The pages, in tab order. |
| `page` | `string` | `""` | — | — | — | The page shown. |
| `pending` | `boolean` | `false` | — | — | — | Some entry is changed and not applied: Apply and Revert are live. |
| `confirmSeconds` | `number` | `0` | — | — | — | A display change was applied: the countdown's seconds left (0 none). At 0 after counting the result says revert. |
| `confirmTotal` | `number` | `15` | — | — | — | The countdown's whole, for its ring. |
| `applyLabel` | `string` | `"Apply"` | — | — | — | Apply. |
| `revertLabel` | `string` | `"Revert"` | — | — | — | Revert the pending changes. |
| `defaultsLabel` | `string` | `"Defaults"` | — | — | — | The page back to its defaults. |
| `confirmTitle` | `string` | `"Keep these display settings?"` | — | — | — | The countdown's question. |
| `keepLabel` | `string` | `"Keep"` | — | — | — | Keep the display change. |
| `revertingLabel` | `string` | `"Reverting in {n} s"` | — | — | — | The countdown's line, {n} its seconds. |
| `hudNote` | `string` | `"Move, resize and fade the HUD's elements over the game."` | — | — | — | The HUD page's line. |
| `editHudLabel` | `string` | `"Edit HUD"` | — | — | — | The HUD page's button. |
| `changedLabel` | `string` | `"changed"` | — | — | — | Said of a changed entry to a screen reader. |

```ogt
struct SettingsTreeDesc {
  doc "The settings as a tree of pages and typed entries, the pending state, and the confirm countdown."
  desc true
  field title String "The screen's title." {
    default "Settings"
  }
  field pages List.SettingsPage "The pages, in tab order."
  field page String "The page shown."
  field pending bool "Some entry is changed and not applied: Apply and Revert are live."
  field confirmSeconds f32 "A display change was applied: the countdown's seconds left (0 none). At 0 after counting the result says revert."
  field confirmTotal f32 "The countdown's whole, for its ring." {
    default 15
  }
  field applyLabel String "Apply." {
    default "Apply"
  }
  field revertLabel String "Revert the pending changes." {
    default "Revert"
  }
  field defaultsLabel String "The page back to its defaults." {
    default "Defaults"
  }
  field confirmTitle String "The countdown's question." {
    default "Keep these display settings?"
  }
  field keepLabel String "Keep the display change." {
    default "Keep"
  }
  field revertingLabel String "The countdown's line, {n} its seconds." {
    default "Reverting in {n} s"
  }
  field hudNote String "The HUD page's line." {
    default "Move, resize and fade the HUD's elements over the game."
  }
  field editHudLabel String "The HUD page's button." {
    default "Edit HUD"
  }
  field changedLabel String "Said of a changed entry to a screen reader." {
    default "changed"
  }
}
```

<sub>Module `ui` line 4330</sub>
