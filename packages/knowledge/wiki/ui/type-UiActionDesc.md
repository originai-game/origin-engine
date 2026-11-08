<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiActionDesc"></a>

### `UiActionDesc` (struct)

A named UI action (docs/ui.md §2.1) bound to og-input's controls: a component asks for an action, not a key.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name: confirm, back, tabNext, tabPrev, details, or the game's own. |
| `keys` | `readonly string[]` | `[]` | — | — | — | Key codes (KeyboardEvent.code: `Enter`, `KeyE`). |
| `buttons` | `readonly string[]` | `[]` | — | — | — | Standard-mapping pad buttons (GamepadButton's names: `a`, `rightBumper`). |
| `trigger` | `UiActionTrigger` | `"press"` | — | — | — | When it fires. |
| `holdMs` | `number` | `0` | — | `ms` | — | How long a hold is (0: 500). |
| `toggle` | `boolean` | `false` | — | — | — | A hold's toggle alternative (XAG 107): a tap turns it on, the next tap off; og.ui.actionHeld answers true between. |
| `label` | `string` | `""` | — | — | — | What it says when no document gives it a hint. |
| `action` | `string` | `""` | — | — | optional | A gameplay action (og.input.defineAction) it follows instead of keys and buttons: it is down when that action is, and its glyph is that action's binding on the device used last, so the player's rebinding moves the prompt too. Empty: keys and buttons. |

```ogt
struct UiActionDesc {
  doc "A named UI action (docs/ui.md §2.1) bound to og-input's controls: a component asks for an action, not a key."
  field name String "Its name: confirm, back, tabNext, tabPrev, details, or the game's own."
  field keys List.String "Key codes (KeyboardEvent.code: `Enter`, `KeyE`)."
  field buttons List.String "Standard-mapping pad buttons (GamepadButton's names: `a`, `rightBumper`)."
  field trigger UiActionTrigger "When it fires."
  field holdMs f32 "How long a hold is (0: 500)." {
    unit ms
  }
  field toggle bool "A hold's toggle alternative (XAG 107): a tap turns it on, the next tap off; og.ui.actionHeld answers true between."
  field label String "What it says when no document gives it a hint."
  field action String "A gameplay action (og.input.defineAction) it follows instead of keys and buttons: it is down when that action is, and its glyph is that action's binding on the device used last, so the player's rebinding moves the prompt too. Empty: keys and buttons." {
    optional true
  }
}
```

<sub>Module `ui` line 7967</sub>
