<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`play`](../play.md) · [index](../index.md)

<a id="fn-candidates"></a>

### `og.play.candidates`

Every entity carrying a component within a range and a cone of a point, nearest first (equal distances in entity order), with its distance, angle and whether it is in sight: what an interaction prompt, a lock-on or an aim assist picks from by its own rule. Inside a system it sees the colliders as castRay does.

Also known as: nearby entities, targets in range, lock-on candidates, interactables in view, find in cone, Unity OverlapSphere then filter

```ts
og.play.candidates(world: World, desc?: CandidatesDesc): Result<readonly Candidate[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `CandidatesDesc` | yes | Where, how far, the cone, the component and sight. |

**Returns** `Result<readonly Candidate[]>`: The candidates; refused for a zero forward, a range or point that is not finite, or a component that was never declared.

Binding symbol `og_play_candidates` · publish builds: kept

```ogt
fn candidates {
  doc "Every entity carrying a component within a range and a cone of a point, nearest first (equal distances in entity order), with its distance, angle and whether it is in sight: what an interaction prompt, a lock-on or an aim assist picks from by its own rule. Inside a system it sees the colliders as castRay does."
  aliases "nearby entities, targets in range, lock-on candidates, interactables in view, find in cone, Unity OverlapSphere then filter"
  param world World "The world."
  param desc CandidatesDesc "Where, how far, the cone, the component and sight."
  returns Result.List.Candidate "The candidates; refused for a zero forward, a range or point that is not finite, or a component that was never declared."
}
```

<sub>Module `play` line 141</sub>
