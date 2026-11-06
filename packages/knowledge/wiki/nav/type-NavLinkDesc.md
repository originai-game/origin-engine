<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="type-NavLinkDesc"></a>

### `NavLinkDesc` (desc struct)

An off-mesh link: a way between two points of a navmesh its surface does not join (a gap to jump, a ladder, a door, a teleporter). Each end snaps to the polygon under it within 1 m.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `start` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where the agent leaves the surface. |
| `end` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | Where it lands. |
| `kind` | `NavLinkKind` | `"jump"` | — | — | — | How it crosses. |
| `bidirectional` | `boolean` | `true` | — | — | — | Crossed both ways; false: from start to end only. |
| `cost` | `number` | `0` | ≥ 0 | `m` | — | What crossing costs a path, in metres of walking; 0 is the link's length. A dear link loses to a shorter walk round. |
| `open` | `boolean` | `true` | — | — | — | A closed link is never planned across (a shut door); agents re-plan when it opens or closes. |
| `duration` | `number` | `0` | ≥ 0 | `s` | — | Seconds a crossing takes; 0 is the kind's own. |
| `game` | `boolean` | `false` | — | — | — | The game moves the agent across (an animation, a cutscene): og.nav holds it at the start until og.nav.finishLink, then lands it at the end. |

```ogt
struct NavLinkDesc {
  doc "An off-mesh link: a way between two points of a navmesh its surface does not join (a gap to jump, a ladder, a door, a teleporter). Each end snaps to the polygon under it within 1 m."
  desc true
  field start Vector3 "Where the agent leaves the surface." {
    unit m
  }
  field end Vector3 "Where it lands." {
    unit m
  }
  field kind NavLinkKind "How it crosses."
  field bidirectional bool "Crossed both ways; false: from start to end only." {
    default true
  }
  field cost f32 "What crossing costs a path, in metres of walking; 0 is the link's length. A dear link loses to a shorter walk round." {
    min 0
    unit m
  }
  field open bool "A closed link is never planned across (a shut door); agents re-plan when it opens or closes." {
    default true
  }
  field duration f32 "Seconds a crossing takes; 0 is the kind's own." {
    min 0
    unit s
  }
  field game bool "The game moves the agent across (an animation, a cutscene): og.nav holds it at the start until og.nav.finishLink, then lands it at the end."
}
```

<sub>`api.ogt` line 13799</sub>
