<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-dropdown"></a>

### `og.ui.dropdown`

A dropdown (a select): a field-like trigger with the chosen option and a chevron that turns as it opens; its list opens below it (above without room, kept on screen) as a floating layer that takes the focus at the chosen option, with groups, icons, descriptions, a tick on the chosen and a filter past 10 options. A press, Enter, Space or A opens it and chooses; Back, Escape or a press outside closes it and the focus returns to the trigger. Plays open, select and close. Parts: `Dropdown.trigger[open][invalid][hover|pressed|focused|disabled]`, `Dropdown.list`, `Dropdown.option[selected][hover|focused|disabled]`, `Dropdown.group`. Role combobox (expanded while open) and a listbox of options. Probe: `<id>/Trigger`, `<id>.list/Panel/Content/Option#i`.

```ts
og.ui.dropdown(id: string, desc?: DropdownDesc): DropdownResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `DropdownDesc` | yes | Its options and states. |

**Returns** `DropdownResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn dropdown {
  doc "A dropdown (a select): a field-like trigger with the chosen option and a chevron that turns as it opens; its list opens below it (above without room, kept on screen) as a floating layer that takes the focus at the chosen option, with groups, icons, descriptions, a tick on the chosen and a filter past 10 options. A press, Enter, Space or A opens it and chooses; Back, Escape or a press outside closes it and the focus returns to the trigger. Plays open, select and close. Parts: `Dropdown.trigger[open][invalid][hover|pressed|focused|disabled]`, `Dropdown.list`, `Dropdown.option[selected][hover|focused|disabled]`, `Dropdown.group`. Role combobox (expanded while open) and a listbox of options. Probe: `<id>/Trigger`, `<id>.list/Panel/Content/Option#i`."
  host false
  param id String "Its id."
  param desc DropdownDesc "Its options and states."
  returns DropdownResult
}
```

<sub>Module `ui` line 4064</sub>
