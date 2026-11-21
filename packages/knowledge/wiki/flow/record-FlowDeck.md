<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="record-FlowDeck"></a>

### `og.FlowDeck` (component)

An entity og.flow plays segments on, with an og.Timeline, an og.VideoPlayer (its `wait` set) and an og.AudioSource. The flow's current deck plays; the others hold a segment it may cut to, paused at its first frame.

Token `og.FlowDeck`, registered under the schema name `og.FlowDeck`; its value type is `FlowDeck`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `flow` | `Entity` | `0` | — | — | read only | Its flow. |
| `node` | `number` | `0` | — | — | read only | The flow node whose segment it holds (its id, as og.Flow.node); 0: none. |

```ogt
component FlowDeck {
  doc "An entity og.flow plays segments on, with an og.Timeline, an og.VideoPlayer (its `wait` set) and an og.AudioSource. The flow's current deck plays; the others hold a segment it may cut to, paused at its first frame."
  field flow Entity "Its flow." {
    readonly true
  }
  field node u32 "The flow node whose segment it holds (its id, as og.Flow.node); 0: none." {
    readonly true
  }
}
```

<sub>Module `flow` line 70</sub>
