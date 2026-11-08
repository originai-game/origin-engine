<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Choreo"></a>

### `Choreo` (union)

Timelines composed into one play (docs/ui.md §3.1).

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "play"` | [`ChoreoPlay`](../ui/type-ChoreoPlay.md#type-ChoreoPlay) | One timeline. |
| `kind: "sequence"` | [`ChoreoSteps`](../ui/type-ChoreoSteps.md#type-ChoreoSteps) | One after another. |
| `kind: "parallel"` | [`ChoreoSteps`](../ui/type-ChoreoSteps.md#type-ChoreoSteps) | All at once. |
| `kind: "stagger"` | [`ChoreoStagger`](../ui/type-ChoreoStagger.md#type-ChoreoStagger) | One timeline over many targets. |
| `kind: "cascade"` | [`ChoreoCascade`](../ui/type-ChoreoCascade.md#type-ChoreoCascade) | Steps chained on callbacks. |

```ogt
union Choreo {
  doc "Timelines composed into one play (docs/ui.md §3.1)."
  tag kind
  case play ChoreoPlay "One timeline."
  case sequence ChoreoSteps "One after another."
  case parallel ChoreoSteps "All at once."
  case stagger ChoreoStagger "One timeline over many targets."
  case cascade ChoreoCascade "Steps chained on callbacks."
}
```

<sub>Module `ui` line 4989</sub>
