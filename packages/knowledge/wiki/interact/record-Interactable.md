<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`interact`](../interact.md) · [index](../index.md)

<a id="record-Interactable"></a>

### `og.Interactable` (component)

Something an og.Interactor can focus and use: found by og.play.candidates at its entity's position. With `holdTicks` the action must be held on it that long (a door to force, a body to search); released sooner, nothing happens.

See `interaction §1`.

Also known as: usable, interaction target, use prompt target, press E to interact, pickup, Unity interactable, Godot interactable area

Token `og.Interactable`, registered under the schema name `og.Interactable`; its value type is `Interactable`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `priority` | `number` | `0` | — | — | — | Ranks it under InteractRule.priority: the highest is focused first. |
| `holdTicks` | `number` | `0` | ≥ 0 | `ticks` | tunable | Ticks the action is held on it before it is used; 0 uses it on the press. |
| `disabled` | `boolean` | `false` | — | — | — | Never focused (a locked door, a lever used up). |

```ogt
component Interactable {
  aliases "usable, interaction target, use prompt target, press E to interact, pickup, Unity interactable, Godot interactable area"
  doc "Something an og.Interactor can focus and use: found by og.play.candidates at its entity's position. With `holdTicks` the action must be held on it that long (a door to force, a body to search); released sooner, nothing happens."
  guide "interaction §1"
  field priority u32 "Ranks it under InteractRule.priority: the highest is focused first."
  field holdTicks u32 "Ticks the action is held on it before it is used; 0 uses it on the press." {
    min 0
    unit ticks
    tunable true
  }
  field disabled bool "Never focused (a locked door, a lever used up)."
}
```

<sub>Module `interact` line 19</sub>
