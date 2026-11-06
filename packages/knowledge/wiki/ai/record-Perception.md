<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="record-Perception"></a>

### `og.Perception` (component)

What an agent sees and hears. It sees the nearest og.Perceivable of another team within `sight`, inside the `fov` cone around `forward` (or the way it last moved), unless a collider is in the way; it hears a spatial sound that starts (og.SoundStarted) or a game noise (og.ai.noise) within `hearing` times its loudness, except its own: a sound of its entity or of an entity under it (og.Parent), or a noise whose NoiseDesc.source is one of them.

Token `og.Perception`, registered under the schema name `og.Perception`; its value type is `Perception`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sight` | `number` | `8` | 0 … 500 | `m` | tunable | How far it sees. |
| `fov` | `number` | `120` | 0 … 360 | `deg` | tunable | The width of its view cone (360: all round). |
| `eyeHeight` | `number` | `0` | -10 … 10 | `m` | — | Its eyes are this far above its position (+y). |
| `occlude` | `boolean` | `true` | — | — | — | Colliders hide what is behind them. |
| `occluders` | `number` | `0` | — | — | — | Collision layers that block sight (0: every layer). |
| `hearing` | `number` | `6` | 0 … 500 | `m` | tunable | How far it hears a sound of loudness 1 (a source's volume). |
| `memory` | `number` | `3` | 0 … 600 | `s` | tunable | How long a lost target stays its target. |
| `team` | `number` | `0` | — | — | — | Its team: it perceives og.Perceivable entities of other teams. |
| `forward` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Where it looks; zero: the way it last moved (before it moves, the way its Transform faces: +Z in 3D, +X in 2D). |
| `target` | `Entity` | `0` | — | — | read only | What it sees or remembers seeing (written by og.ai). |
| `sees` | `boolean` | `false` | — | — | read only | It sees the target now (written by og.ai). |
| `lastSeen` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only, worldspace `world` | Where it last saw the target (written by og.ai). |
| `sinceSeen` | `number` | `1000000` | — | `s` | read only | Seconds since it last saw a target (written by og.ai; 1e6 before the first time). |
| `heard` | `boolean` | `false` | — | — | read only | It heard something this tick (written by og.ai). |
| `heardAt` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | read only, worldspace `world` | Where it last heard something (written by og.ai). |
| `sinceHeard` | `number` | `1000000` | — | `s` | read only | Seconds since it last heard something (written by og.ai; 1e6 before the first time). |

```ogt
component Perception {
  doc "What an agent sees and hears. It sees the nearest og.Perceivable of another team within `sight`, inside the `fov` cone around `forward` (or the way it last moved), unless a collider is in the way; it hears a spatial sound that starts (og.SoundStarted) or a game noise (og.ai.noise) within `hearing` times its loudness, except its own: a sound of its entity or of an entity under it (og.Parent), or a noise whose NoiseDesc.source is one of them."
  field sight f32 "How far it sees." {
    default 8
    min 0
    max 500
    unit m
    tunable true
  }
  field fov f32 "The width of its view cone (360: all round)." {
    default 120
    min 0
    max 360
    unit deg
    tunable true
  }
  field eyeHeight f32 "Its eyes are this far above its position (+y)." {
    min -10
    max 10
    unit m
  }
  field occlude bool "Colliders hide what is behind them." {
    default true
  }
  field occluders u32 "Collision layers that block sight (0: every layer)."
  field hearing f32 "How far it hears a sound of loudness 1 (a source's volume)." {
    default 6
    min 0
    max 500
    unit m
    tunable true
  }
  field memory f32 "How long a lost target stays its target." {
    default 3
    min 0
    max 600
    unit s
    tunable true
  }
  field team u32 "Its team: it perceives og.Perceivable entities of other teams."
  field forward Vector3 "Where it looks; zero: the way it last moved (before it moves, the way its Transform faces: +Z in 3D, +X in 2D)."
  field target Entity "What it sees or remembers seeing (written by og.ai)." {
    readonly true
  }
  field sees bool "It sees the target now (written by og.ai)." {
    readonly true
  }
  field lastSeen Vector3 "Where it last saw the target (written by og.ai)." {
    worldspace world
    unit m
    readonly true
  }
  field sinceSeen f32 "Seconds since it last saw a target (written by og.ai; 1e6 before the first time)." {
    default 1000000
    unit s
    readonly true
  }
  field heard bool "It heard something this tick (written by og.ai)." {
    readonly true
  }
  field heardAt Vector3 "Where it last heard something (written by og.ai)." {
    worldspace world
    unit m
    readonly true
  }
  field sinceHeard f32 "Seconds since it last heard something (written by og.ai; 1e6 before the first time)." {
    default 1000000
    unit s
    readonly true
  }
}
```

<sub>`api.ogt` line 14177</sub>
