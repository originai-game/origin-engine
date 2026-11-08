<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SettingEntry"></a>

### `SettingEntry` (desc struct)

One entry of a settings page, as data: a game adds, drops or reorders entries without touching the pattern.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key: its row's element id (<id>/Page/Rows/<key>) and the key of its change. |
| `kind` | `SettingKind` | `"toggle"` | — | — | — | Its control. |
| `label` | `string` | `""` | — | — | — | Its name. |
| `description` | `string` | `""` | — | — | — | What it does: shown in the detail panel while its row is hovered or focused. |
| `section` | `string` | `""` | — | — | — | A heading drawn above it where it differs from the entry before; empty is none. |
| `on` | `boolean` | `false` | — | — | — | A toggle's state. |
| `value` | `number` | `0` | — | — | — | A slider's value. |
| `min` | `number` | `0` | — | — | — | A slider's low end. |
| `max` | `number` | `1` | — | — | — | A slider's high end. |
| `step` | `number` | `0` | — | — | — | A slider's step; 0 is continuous. |
| `unit` | `string` | `""` | — | — | — | A slider's unit after its value; `%` writes a 0-to-1 value as a percent. |
| `options` | `readonly string[]` | `[]` | — | — | — | A choice's words. |
| `index` | `number` | `0` | — | — | — | A choice's word chosen. |
| `action` | `string` | `""` | — | — | — | A binding's input action (its name in the action map). |
| `display` | `boolean` | `false` | — | — | — | A display change (fullscreen, render scale, frame cap): applying it starts the confirm-or-revert countdown. |
| `dirty` | `boolean` | `false` | — | — | — | Changed and not yet applied: a mark beside its name. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but not to be changed (the backend cannot), with its description saying why. |
| `preview` | `Texture \| null` | `null` | — | — | — | A picture of its effect (art kind banner, 16:9) under its description in the detail panel; none draws no picture. |

```ogt
struct SettingEntry {
  doc "One entry of a settings page, as data: a game adds, drops or reorders entries without touching the pattern."
  desc true
  field key String "Its key: its row's element id (<id>/Page/Rows/<key>) and the key of its change."
  field kind SettingKind "Its control."
  field label String "Its name."
  field description String "What it does: shown in the detail panel while its row is hovered or focused."
  field section String "A heading drawn above it where it differs from the entry before; empty is none."
  field on bool "A toggle's state."
  field value f32 "A slider's value."
  field min f32 "A slider's low end."
  field max f32 "A slider's high end." {
    default 1
  }
  field step f32 "A slider's step; 0 is continuous."
  field unit String "A slider's unit after its value; `%` writes a 0-to-1 value as a percent."
  field options List.String "A choice's words."
  field index u32 "A choice's word chosen."
  field action String "A binding's input action (its name in the action map)."
  field display bool "A display change (fullscreen, render scale, frame cap): applying it starts the confirm-or-revert countdown."
  field dirty bool "Changed and not yet applied: a mark beside its name."
  field disabled bool "Shown but not to be changed (the backend cannot), with its description saying why."
  field preview Option.Texture "A picture of its effect (art kind banner, 16:9) under its description in the detail panel; none draws no picture."
}
```

<sub>Module `ui` line 4216</sub>
