<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

<a id="record-TimelineStarted"></a>

### `og.TimelineStarted` (event)

A timeline started playing from stopped or ended (not on a resume from pause, nor on a loop).

Token `og.TimelineStarted`, registered under the schema name `og.TimelineStarted`; its value type is `TimelineStarted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The og.Timeline's entity. |
| `asset` | `TimelineAsset` | `0` | — | — | — | The timeline. |

```ogt
event TimelineStarted {
  doc "A timeline started playing from stopped or ended (not on a resume from pause, nor on a loop)."
  field entity Entity "The og.Timeline's entity."
  field asset TimelineAsset "The timeline."
}
```

<sub>`api.ogt` line 26161</sub>
