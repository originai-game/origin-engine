<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`interact`](../interact.md) · [index](../index.md)

<a id="record-Interacted"></a>

### `og.Interacted` (event)

An og.Interactor used an og.Interactable: sent by og.systems.interact the tick it was pressed, or the tick its hold was complete, before the door, toggle or grab it carries changes.

See `interaction §1`.

Also known as: on interact, used, interaction event, picked up, lever pulled

Token `og.Interacted`, registered under the schema name `og.Interacted`; its value type is `Interacted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `interactor` | `Entity` | `0` | — | — | — | The og.Interactor. |
| `target` | `Entity` | `0` | — | — | — | The og.Interactable. |

```ogt
event Interacted {
  aliases "on interact, used, interaction event, picked up, lever pulled"
  doc "An og.Interactor used an og.Interactable: sent by og.systems.interact the tick it was pressed, or the tick its hold was complete, before the door, toggle or grab it carries changes."
  guide "interaction §1"
  field interactor Entity "The og.Interactor."
  field target Entity "The og.Interactable."
}
```

<sub>Module `interact` line 65</sub>
