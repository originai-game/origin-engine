<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-menu"></a>

### `og.ui.menu`

A menu of commands, checks, radios, separators, headers and submenus, hanging from an element or at a point. A submenu opens beside its row; where none fits beside the menu (a phone) it drills in, its rows replacing the menu's under a back row, and Back goes up a level.

```ts
og.ui.menu(id: string, open: boolean, desc?: MenuDesc): MenuChoice
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it is open. |
| `desc` | `MenuDesc` | yes | Entries and where it hangs. |

**Returns** `MenuChoice`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn menu {
  doc "A menu of commands, checks, radios, separators, headers and submenus, hanging from an element or at a point. A submenu opens beside its row; where none fits beside the menu (a phone) it drills in, its rows replacing the menu's under a back row, and Back goes up a level."
  host false
  param id String "Its id."
  param open bool "Whether it is open."
  param desc MenuDesc "Entries and where it hangs."
  returns MenuChoice
}
```

<sub>Module `ui` line 2756</sub>
