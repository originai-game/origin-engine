<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-statusGrid"></a>

### `og.ui.statusGrid`

Buffs and debuffs to the bar of MMOs: icons in rows, a sweep darkening each as it runs out, the time left under it (12m, 45, 4.2), blinking in its last five seconds, stacks at its corner, debuffs in their own row bordered by their type's colour and notched so their kind is not told by colour alone, more than max folding into a +n tile; the columns as wide as the row's widest timer, so the tiles stay evenly spaced and each time overhangs its tile centred (WoW's auras).

Parts: StatusGrid.tile[buff|debuff][type=<type>][expiring], StatusGrid.icon, StatusGrid.sweep, StatusGrid.time, StatusGrid.stacks, StatusGrid.more. States: on each tile, group time (lasting, running, expiring). Accessibility: list named Buffs and Debuffs; each tile an image named name, stacks and seconds. Probe ids: <id>/Buffs/<key>, <id>/Debuffs/<key>, their Time and Stacks, <id>/Buffs/More/Tile/Count.

```ts
og.ui.statusGrid(id: string, desc?: StatusGridDesc): StatusGridResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `StatusGridDesc` | yes | The effects. |

**Returns** `StatusGridResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn statusGrid {
  doc "Buffs and debuffs to the bar of MMOs: icons in rows, a sweep darkening each as it runs out, the time left under it (12m, 45, 4.2), blinking in its last five seconds, stacks at its corner, debuffs in their own row bordered by their type's colour and notched so their kind is not told by colour alone, more than max folding into a +n tile; the columns as wide as the row's widest timer, so the tiles stay evenly spaced and each time overhangs its tile centred (WoW's auras)."
  doc "Parts: StatusGrid.tile[buff|debuff][type=<type>][expiring], StatusGrid.icon, StatusGrid.sweep, StatusGrid.time, StatusGrid.stacks, StatusGrid.more. States: on each tile, group time (lasting, running, expiring). Accessibility: list named Buffs and Debuffs; each tile an image named name, stacks and seconds. Probe ids: <id>/Buffs/<key>, <id>/Debuffs/<key>, their Time and Stacks, <id>/Buffs/More/Tile/Count."
  host false
  param id String "Its id."
  param desc StatusGridDesc "The effects."
  returns StatusGridResult
}
```

<sub>Module `ui` line 6012</sub>
