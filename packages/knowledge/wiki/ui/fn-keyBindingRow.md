<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-keyBindingRow"></a>

### `og.ui.keyBindingRow`

A key-binding row (the settings tree's controls page, docs/ui.md §11): the action's name and a slot per binding on a device (keyboard and mouse, or a pad, the device used last by default) showing its glyphs from og.input (keys as caps, pad buttons as their family's discs, composites as their parts). A press on a slot (Enter, A) starts og.input.startCapture for it: the slot pulses `Press a key…` with the cancel hint until the player presses something (Escape, the pad's back or the timeout cancels). A capture with no conflict is applied at once; one that shares a control asks inline (`W is used by Move forward`: Swap, Replace, Cancel) with og.input's conflict report. A locked action shows a lock and takes no press; a rebound one shows a reset. Plays select, confirm, cancel and error. Parts: `Rebind.row[capturing][conflict][locked]`, `Rebind.slot[empty][capturing][conflict][hover|pressed|focused|disabled]`, `Rebind.conflict`; the caps are the action bar's `ActionBar.glyph[key|face|shoulder]`, the reset an icon button (`Button.base[quiet][icon]`). A group named by the action, slots as buttons named `<action>, <slot>: <binding>`. Probe: `<id>/Main/Bound/Slots/Slot#i`, `<id>/Conflict/Swap`, `<id>/Conflict/Replace`, `<id>/Conflict/Cancel`, `<id>/Main/Bound/Reset` (once rebound). Where its slots and Reset do not fit beside each other at their least width (200 % text, a narrow column), the row stacks them, the first slot over the second, so every row of a list stacks alike; a key cluster (W A S D) keeps its order right to left.

```ts
og.ui.keyBindingRow(world: World, id: string, desc?: KeyBindingRowDesc): KeyBindingResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world (the action map is world state). |
| `id` | `string` | no | Its id. |
| `desc` | `KeyBindingRowDesc` | yes | The action, device and labels. |

**Returns** `KeyBindingResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn keyBindingRow {
  doc "A key-binding row (the settings tree's controls page, docs/ui.md §11): the action's name and a slot per binding on a device (keyboard and mouse, or a pad, the device used last by default) showing its glyphs from og.input (keys as caps, pad buttons as their family's discs, composites as their parts). A press on a slot (Enter, A) starts og.input.startCapture for it: the slot pulses `Press a key…` with the cancel hint until the player presses something (Escape, the pad's back or the timeout cancels). A capture with no conflict is applied at once; one that shares a control asks inline (`W is used by Move forward`: Swap, Replace, Cancel) with og.input's conflict report. A locked action shows a lock and takes no press; a rebound one shows a reset. Plays select, confirm, cancel and error. Parts: `Rebind.row[capturing][conflict][locked]`, `Rebind.slot[empty][capturing][conflict][hover|pressed|focused|disabled]`, `Rebind.conflict`; the caps are the action bar's `ActionBar.glyph[key|face|shoulder]`, the reset an icon button (`Button.base[quiet][icon]`). A group named by the action, slots as buttons named `<action>, <slot>: <binding>`. Probe: `<id>/Main/Bound/Slots/Slot#i`, `<id>/Conflict/Swap`, `<id>/Conflict/Replace`, `<id>/Conflict/Cancel`, `<id>/Main/Bound/Reset` (once rebound). Where its slots and Reset do not fit beside each other at their least width (200 % text, a narrow column), the row stacks them, the first slot over the second, so every row of a list stacks alike; a key cluster (W A S D) keeps its order right to left."
  host false
  param world World "The world (the action map is world state)."
  param id String "Its id."
  param desc KeyBindingRowDesc "The action, device and labels."
  returns KeyBindingResult
}
```

<sub>`api.ogt` line 19870</sub>
