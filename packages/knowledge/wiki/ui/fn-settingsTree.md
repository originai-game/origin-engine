<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-settingsTree"></a>

### `og.ui.settingsTree`

A game's settings as data (docs/ui.md §2, roadmap G1-7): the title, the pages as tabs (the tab actions step them), the shown page's entries as rows, each its name (a changed one marked) and its control at the end (a switch, a slider with its reading, a segmented control or a dropdown, og.ui.keyBindingRow for a binding by physical key code shown by the layout's label, a button, or the game's own control in a custom row), a heading where its section changes; whole rows scroll. At the end a detail panel with the hovered or focused entry's name and description. At the foot Defaults, Revert and Apply (Apply the primary, both quiet with nothing pending). After applying a display change the confirm dialog asks to keep it, its seconds counting down on a ring, Keep and Revert; at 0 it reverts. The HUD page mounts og.ui.hudLayoutEditor (the component, not a copy) while og.ui.hudEdit() is on, and Edit HUD to turn it on. On a phone in portrait the tabs are chips that scroll, the detail is the row's description under its name, and the foot's buttons span the width.

Parts: Settings.tab[selected], Settings.section, Settings.row[changed][disabled], Settings.detail, Settings.foot, Settings.confirm, Settings.ring. Sounds: tab on a tab, toggle on a switch, confirm on Apply and Keep, back on Revert. Accessibility: the tabs a tablist, each row a group named by its entry (and changed), the confirm an alertdialog whose countdown is a timer. Probe ids: <id>/Tabs/Row/<page>, <id>/Body/Page/Rows/<entry> (its control <entry>/Control), <id>/Foot/Apply, <id>/Foot/Revert, <id>/Foot/Defaults, <id>/Scrim/Confirm/Buttons/Keep, <id>/Scrim/Confirm/Buttons/Revert, <id>/Body/Page/Hud/Edit, <id>/Body/Page/Hud/Editor.

```ts
og.ui.settingsTree(world: World, id: string, desc: SettingsTreeDesc, custom: SettingsCustomBuild): SettingsTreeResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world (the bindings' action map and capture are world state). |
| `id` | `string` | no | Its id. |
| `desc` | `SettingsTreeDesc` | no | The pages and entries, the pending state, the countdown. |
| `custom` | `SettingsCustomBuild` | no | Builds a custom entry's control by its key, inside its row; the game's own widgets, their results its own (a game with no custom entry passes a function that does nothing). |

**Returns** `SettingsTreeResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn settingsTree {
  doc "A game's settings as data (docs/ui.md §2, roadmap G1-7): the title, the pages as tabs (the tab actions step them), the shown page's entries as rows, each its name (a changed one marked) and its control at the end (a switch, a slider with its reading, a segmented control or a dropdown, og.ui.keyBindingRow for a binding by physical key code shown by the layout's label, a button, or the game's own control in a custom row), a heading where its section changes; whole rows scroll. At the end a detail panel with the hovered or focused entry's name and description. At the foot Defaults, Revert and Apply (Apply the primary, both quiet with nothing pending). After applying a display change the confirm dialog asks to keep it, its seconds counting down on a ring, Keep and Revert; at 0 it reverts. The HUD page mounts og.ui.hudLayoutEditor (the component, not a copy) while og.ui.hudEdit() is on, and Edit HUD to turn it on. On a phone in portrait the tabs are chips that scroll, the detail is the row's description under its name, and the foot's buttons span the width."
  doc "Parts: Settings.tab[selected], Settings.section, Settings.row[changed][disabled], Settings.detail, Settings.foot, Settings.confirm, Settings.ring. Sounds: tab on a tab, toggle on a switch, confirm on Apply and Keep, back on Revert. Accessibility: the tabs a tablist, each row a group named by its entry (and changed), the confirm an alertdialog whose countdown is a timer. Probe ids: <id>/Tabs/Row/<page>, <id>/Body/Page/Rows/<entry> (its control <entry>/Control), <id>/Foot/Apply, <id>/Foot/Revert, <id>/Foot/Defaults, <id>/Scrim/Confirm/Buttons/Keep, <id>/Scrim/Confirm/Buttons/Revert, <id>/Body/Page/Hud/Edit, <id>/Body/Page/Hud/Editor."
  host false
  param world World "The world (the bindings' action map and capture are world state)."
  param id String "Its id."
  param desc SettingsTreeDesc "The pages and entries, the pending state, the countdown."
  param custom SettingsCustomBuild "Builds a custom entry's control by its key, inside its row; the game's own widgets, their results its own (a game with no custom entry passes a function that does nothing)."
  returns SettingsTreeResult
}
```

<sub>Module `ui` line 4181</sub>
