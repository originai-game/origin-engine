<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-MenuItem"></a>
### `MenuItem`

An item of a menu (NSH base's `MenuItem`): a focus stop of its menu that reports `onSelect` and closes it. The pack draws it (`Menu.item`, `Menu.item.danger`), the check before a checkable item's text (`Menu.indicator`, shown by its `checked:` classes) and its hint (`Menu.shortcut`).

`import { MenuItem } from "@origingame/engine/ui/components"` (kit `controls`, `packages/engine/src/ui/controls/menu.tsx`).

| Prop | Type | What |
|---|---|---|
| `onSelect?` | `((e: UiEvent) => void) \| undefined` | It was chosen: a press, Enter or Space. The menu closes unless `closeOnSelect` is false. |
| `disabled?` | `boolean \| undefined` |  |
| `checked?` | `boolean \| undefined` | Checkable: a setting it shows (`menuitemcheckbox`), or with `radio` one choice of a group (`menuitemradio`). The game owns the value; `checked:` styles apply. |
| `radio?` | `boolean \| undefined` |  |
| `danger?` | `boolean \| undefined` | An action that destroys or cannot be undone: the pack's `Menu.item.danger`. |
| `icon?` | `string \| undefined` | An icon name before its text. |
| `shortcut?` | `string \| undefined` | A hint after its text: a key or a count. |
| `closeOnSelect?` | `boolean \| undefined` | Keeps the menu open once chosen, so a setting shows its new value at once. |
| `label?` | `string \| undefined` |  |
| `class?` | `string \| undefined` | Classes of the document that uses it, after its own. |
| `children?` | `any` |  |

Pack slots: `Menu.frame`, `Menu.indicator`, `Menu.item`, `Menu.item.danger`, `Menu.label`, `Menu.separator`, `Menu.shortcut`.

A piece of a panel: drawn on a panel's ground.

Fixtures: `default`, `closed`.
