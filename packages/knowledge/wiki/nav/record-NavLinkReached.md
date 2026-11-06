<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="record-NavLinkReached"></a>

### `og.NavLinkReached` (event)

An agent reached an off-mesh link and started across it (systems read it the tick after): play the jump or the climb here. With `game`, move it and call og.nav.finishLink when it is over.

Token `og.NavLinkReached`, registered under the schema name `og.NavLinkReached`; its value type is `NavLinkReached`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `agent` | `Entity` | `0` | — | — | — | The agent. |
| `mesh` | `Navigation` | `0` | — | — | — | The navmesh. |
| `link` | `number` | `0` | — | — | — | The link's id. |
| `kind` | `number` | `0` | — | — | — | Its NavLinkKind, by index: 0 jump, 1 ladder, 2 door, 3 teleport. |
| `start` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where the agent leaves the surface. |
| `end` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where it lands. |
| `game` | `boolean` | `false` | — | — | — | The game moves it across. |

```ogt
event NavLinkReached {
  doc "An agent reached an off-mesh link and started across it (systems read it the tick after): play the jump or the climb here. With `game`, move it and call og.nav.finishLink when it is over."
  field agent Entity "The agent."
  field mesh Navigation "The navmesh."
  field link u32 "The link's id."
  field kind u32 "Its NavLinkKind, by index: 0 jump, 1 ladder, 2 door, 3 teleport."
  field start Vector3 "Where the agent leaves the surface." {
    worldspace world
    unit m
  }
  field end Vector3 "Where it lands." {
    worldspace world
    unit m
  }
  field game bool "The game moves it across."
}
```

<sub>`api.ogt` line 13819</sub>
