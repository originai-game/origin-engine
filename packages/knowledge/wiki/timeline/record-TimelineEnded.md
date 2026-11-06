<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

<a id="record-TimelineEnded"></a>

### `og.TimelineEnded` (event)

A timeline without loop reached its duration and stopped playing.

Token `og.TimelineEnded`, registered under the schema name `og.TimelineEnded`; its value type is `TimelineEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The og.Timeline's entity. |
| `asset` | `TimelineAsset` | `0` | — | — | — | The timeline. |

```ogt
event TimelineEnded {
  doc "A timeline without loop reached its duration and stopped playing."
  field entity Entity "The og.Timeline's entity."
  field asset TimelineAsset "The timeline."
}
```

<sub>`api.ogt` line 26200</sub>
