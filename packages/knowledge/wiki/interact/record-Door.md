<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`interact`](../interact.md) · [index](../index.md)

<a id="record-Door"></a>

### `og.Door` (component)

A door: using it (og.Interactable) flips `open`, or it follows an og.Toggle's `on`; with a navmesh link, og.nav.setLinkOpen opens and closes it with the door, so agents path through only while it is open. Moving its leaf is the game's (a tween, a hinge), read from `open`.

Also known as: door, gate, open close door, navmesh door, Unity NavMeshLink door, locked passage

Token `og.Door`, registered under the schema name `og.Door`; its value type is `Door`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `open` | `boolean` | `false` | — | — | — | Whether it is open; write it to open or close it from the game. |
| `toggle` | `Entity` | `0` | — | — | — | An og.Toggle (a lever, a pressure plate) whose `on` it follows; 0: using it flips it. |
| `mesh` | `Navigation` | `0` | — | — | — | The navmesh its link is on (og.nav.build, og.nav.load); 0: no link. |
| `link` | `number` | `0` | — | — | — | The link's id on `mesh` (og.nav.setLink, a baked `link` line). |
| `linkOpen` | `boolean` | `false` | — | — | read only | Written: what og.nav.setLinkOpen was last told. |
| `linked` | `boolean` | `false` | — | — | read only | Written: the link has been set at least once (on the first run, to `open`). |

```ogt
component Door {
  aliases "door, gate, open close door, navmesh door, Unity NavMeshLink door, locked passage"
  doc "A door: using it (og.Interactable) flips `open`, or it follows an og.Toggle's `on`; with a navmesh link, og.nav.setLinkOpen opens and closes it with the door, so agents path through only while it is open. Moving its leaf is the game's (a tween, a hinge), read from `open`."
  field open bool "Whether it is open; write it to open or close it from the game."
  field toggle Entity "An og.Toggle (a lever, a pressure plate) whose `on` it follows; 0: using it flips it."
  field mesh Navigation "The navmesh its link is on (og.nav.build, og.nav.load); 0: no link."
  field link u32 "The link's id on `mesh` (og.nav.setLink, a baked `link` line)."
  field linkOpen bool "Written: what og.nav.setLinkOpen was last told." {
    readonly true
  }
  field linked bool "Written: the link has been set at least once (on the first run, to `open`)." {
    readonly true
  }
}
```

<sub>Module `interact` line 69</sub>
