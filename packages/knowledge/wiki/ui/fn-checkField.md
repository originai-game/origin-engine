<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-checkField"></a>

### `og.ui.checkField`

A checkbox: a box that fills with primary and draws its tick (a dash when mixed) with a pop as it turns on (its selection group), with a label and a description; the whole row takes a press, Enter, Space or A. Plays toggleOn and toggleOff. Parts: `Checkbox.box[checked|mixed][invalid][hover|pressed|focused|disabled]`, `Checkbox.mark`, `Checkbox.row[hover|pressed|focused|disabled]` (the whole row). Role checkbox, checked when on (and its value `mixed`). Probe: `<id>`, `<id>/Box`, `<id>/Box/Mark`.

Also known as: checkbox, tick box, Godot CheckBox

```ts
og.ui.checkField(id: string, checked: boolean, desc?: CheckFieldDesc): SwitchResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `checked` | `boolean` | no | Its state from game state. |
| `desc` | `CheckFieldDesc` | yes | Its label and states. |

**Returns** `SwitchResult`.

Binding symbol `og_ui_check_field` · publish builds: kept

```ogt
fn checkField {
  doc "A checkbox: a box that fills with primary and draws its tick (a dash when mixed) with a pop as it turns on (its selection group), with a label and a description; the whole row takes a press, Enter, Space or A. Plays toggleOn and toggleOff. Parts: `Checkbox.box[checked|mixed][invalid][hover|pressed|focused|disabled]`, `Checkbox.mark`, `Checkbox.row[hover|pressed|focused|disabled]` (the whole row). Role checkbox, checked when on (and its value `mixed`). Probe: `<id>`, `<id>/Box`, `<id>/Box/Mark`."
  aliases "checkbox, tick box, Godot CheckBox"
  param id String "Its id."
  param checked bool "Its state from game state."
  param desc CheckFieldDesc "Its label and states."
  returns SwitchResult
}
```

<sub>Module `ui` line 3568</sub>
