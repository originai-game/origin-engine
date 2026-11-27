<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `interact` module

The records og.systems.interact carries out: an og.Interactor focuses one og.Interactable among og.play.candidates by its own rule (nearest, most centred, priority then nearest), uses it on a press of its action or after holding it, and reports og.Interacted. Using an og.Door opens or closes it and its navmesh link, an og.Toggle flips, an og.Grab is picked up and later thrown. Every number is the game's: a zero range focuses nothing.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module](index.md).

```ogt
module interact {
  doc "The records og.systems.interact carries out: an og.Interactor focuses one og.Interactable among og.play.candidates by its own rule (nearest, most centred, priority then nearest), uses it on a press of its action or after holding it, and reports og.Interacted. Using an og.Door opens or closes it and its navmesh link, an og.Toggle flips, an og.Grab is picked up and later thrown. Every number is the game's: a zero range focuses nothing."
  # declarations follow, each on its own page
```

<sub>Module `interact` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`InteractRule`](interact/type-InteractRule.md) | enum |  | How an og.Interactor picks its focus among the candidates in range, in its cone and in sight. |
| [`InteractRule()`](#column-InteractRule) | column kind |  | An InteractRule field: the variant's index. |
| [`og.Interactable`](interact/record-Interactable.md) | component |  | Something an og.Interactor can focus and use: found by og.play.candidates at its entity's position. |
| [`og.Interactor`](interact/record-Interactor.md) | component |  | Who interacts: each tick og.systems.interact looks from its og.Transform (a root entity's) plus `eye`, along its forward (+Z), for og.Interactable entities by og.play.candidates, focuses one by `rule… |
| [`og.Interacted`](interact/record-Interacted.md) | event |  | An og.Interactor used an og.Interactable: sent by og.systems.interact the tick it was pressed, or the tick its hold was complete, before the door, toggle or grab it carries changes. |
| [`og.Door`](interact/record-Door.md) | component |  | A door: using it (og.Interactable) flips `open`, or it follows an og.Toggle's `on`; with a navmesh link, og.nav.setLinkOpen opens and closes it with the door, so agents path through only while it is … |
| [`og.Toggle`](interact/record-Toggle.md) | component |  | Something on or off: using it (og.Interactable) flips `on`; a `plate` is on exactly while its entity's sensor collider holds anything (og.physics.sensorContents), and using it does nothing. |
| [`og.Grab`](interact/record-Grab.md) | component |  | A body an og.Interactor picks up when it uses it: while carried it is made kinematic and moved each tick to `carry` in the carrier's frame; the carrier's next press throws it, dynamic again, along th… |
| [`og.InteractEntities`](interact/record-InteractEntities.md) | resource |  | Every og.Interactor, og.Toggle and og.Door in entity order, written by og.systems.interact's first systems each tick for its last, which reads the others' records entity by entity. |

## Column kinds

Each is a factory, `InteractRule(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-InteractRule"></a>`InteractRule` | `InteractRule` | `Uint32Array` | 1 | An InteractRule field: the variant's index. |

```ogt
column InteractRule {
  doc "An InteractRule field: the variant's index."
  array Uint32Array
  stride 1
  value InteractRule
}
```

<sub>Module `interact` line 11</sub>
