<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`sight`](../sight.md) · [index](../index.md)

<a id="record-Sight"></a>

### `og.Sight` (component)

A team's eyes on the og.SightMap: from its entity's world position it sees every cell within `radius` that no og.SightBlocker hides (symmetric: between two open cells, one sees the other exactly when the other sees it). A blocker's near face is seen. Height is not looked at.

See `world §11`.

Also known as: vision radius, sight range, unit vision, reveals fog, scout, line of sight source, field of view source

Token `og.Sight`, registered under the schema name `og.Sight`; its value type is `Sight`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `team` | `number` | `0` | ≤ 31 | — | — | The team that sees through it (as og.AiAgent.team, og.Perceivable.team). |
| `radius` | `number` | `8` | ≥ 0 | `m` | tunable | How far it sees; 0 sees nothing. |

```ogt
component Sight {
  aliases "vision radius, sight range, unit vision, reveals fog, scout, line of sight source, field of view source"
  doc "A team's eyes on the og.SightMap: from its entity's world position it sees every cell within `radius` that no og.SightBlocker hides (symmetric: between two open cells, one sees the other exactly when the other sees it). A blocker's near face is seen. Height is not looked at."
  guide "world §11"
  field team u32 "The team that sees through it (as og.AiAgent.team, og.Perceivable.team)." {
    max 31
  }
  field radius f32 "How far it sees; 0 sees nothing." {
    default 8
    min 0
    unit m
    tunable true
  }
}
```

<sub>Module `sight` line 45</sub>
