<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-dropdown"></a>

### `og.ui.dropdown`

A dropdown (a select): a field-like trigger with the chosen option and a chevron that turns as it opens; its list opens below it (above without room, kept on screen) as a floating layer that takes the focus at the chosen option, with groups, icons, descriptions, a tick on the chosen and a filter past 10 options. A press, Enter, Space or A opens it and chooses; Back, Escape or a press outside closes it and the focus returns to the trigger. Plays open, select and close. Parts: `Dropdown.trigger[open][invalid][hover|pressed|focused|disabled]`, `Dropdown.list`, `Dropdown.option[selected][hover|focused|disabled]`, `Dropdown.group`. Role combobox (expanded while open) and a listbox of options. Probe: `<id>/Trigger`, `<id>.list/List/Option#i`.

Also known as: select, combobox, Unity Dropdown, Godot OptionButton

```ts
og.ui.dropdown(id: string, desc?: DropdownDesc): DropdownResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `DropdownDesc` | yes | Its options and states. |

**Returns** `DropdownResult`.

Binding symbol `og_ui_dropdown` · publish builds: kept

```ogt
fn dropdown {
  doc "A dropdown (a select): a field-like trigger with the chosen option and a chevron that turns as it opens; its list opens below it (above without room, kept on screen) as a floating layer that takes the focus at the chosen option, with groups, icons, descriptions, a tick on the chosen and a filter past 10 options. A press, Enter, Space or A opens it and chooses; Back, Escape or a press outside closes it and the focus returns to the trigger. Plays open, select and close. Parts: `Dropdown.trigger[open][invalid][hover|pressed|focused|disabled]`, `Dropdown.list`, `Dropdown.option[selected][hover|focused|disabled]`, `Dropdown.group`. Role combobox (expanded while open) and a listbox of options. Probe: `<id>/Trigger`, `<id>.list/List/Option#i`."
  aliases "select, combobox, Unity Dropdown, Godot OptionButton"
  param id String "Its id."
  param desc DropdownDesc "Its options and states."
  returns DropdownResult
}
```

<sub>Module `ui` line 1944</sub>
