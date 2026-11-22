<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="record-Flow"></a>

### `og.Flow` (component)

A flow og.flow.start made: the conversation it paces and its decks. og.flow writes it; read og.flow.view.

Token `og.Flow`, registered under the schema name `og.Flow`; its value type is `Flow`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | read only | Its og.story conversation. |
| `decks` | `number` | `0` | — | — | read only | Its decks. |
| `current` | `Entity` | `0` | — | — | read only | The deck playing now (an og.FlowDeck). |
| `node` | `number` | `0` | — | — | read only | The flow node playing, as its id (a hash of its title; og.flow.view names it); 0 before the first. |
| `pending` | `number` | `-1` | — | — | read only | The choice taken in the window (ChoiceView.index); -1: none yet. |
| `window` | `number` | `0` | — | — | read only | 0 closed, 1 open (choices shown), 2 locked (past flow.lock: the cut takes pending, else the default). |
| `paused` | `boolean` | `false` | — | — | read only | Held by og.flow.pause (or FlowStartDesc.paused) until og.flow.play. |
| `chapter` | `number` | `0` | — | — | read only | The node of the last chapter entered (its id, as `node`); 0: none. |
| `ended` | `boolean` | `false` | — | — | read only | The last segment played to its cut and the conversation ended. |
| `held` | `boolean` | `false` | — | — | read only | The segment playing stopped at its `flow.hold` marker: it waits for og.flow.advance (or a choice), its window open without time running out. |
| `previous` | `Entity` | `0` | — | — | read only | The deck dissolving out (`og-flow-dissolve`), playing on under the current one; 0: none. |
| `dissolve` | `number` | `0` | — | — | read only | The dissolve's length, ticks. |
| `dissolveLeft` | `number` | `0` | — | — | read only | Ticks left of the dissolve; 0: none. |
| `qte` | `readonly number[]` | `[]` | — | — | a list of at most 8 `u32` (`List.u32.8`): reads a frozen copy, writes a whole array, read only | Per choice (ChoiceView.index, the first 8): a quick-time event's progress this window, ticks held or presses. |

```ogt
component Flow {
  doc "A flow og.flow.start made: the conversation it paces and its decks. og.flow writes it; read og.flow.view."
  field conversation u32 "Its og.story conversation." {
    readonly true
  }
  field decks u32 "Its decks." {
    readonly true
  }
  field current Entity "The deck playing now (an og.FlowDeck)." {
    readonly true
  }
  field node u32 "The flow node playing, as its id (a hash of its title; og.flow.view names it); 0 before the first." {
    readonly true
  }
  field pending i32 "The choice taken in the window (ChoiceView.index); -1: none yet." {
    default -1
    readonly true
  }
  field window u32 "0 closed, 1 open (choices shown), 2 locked (past flow.lock: the cut takes pending, else the default)." {
    readonly true
  }
  field paused bool "Held by og.flow.pause (or FlowStartDesc.paused) until og.flow.play." {
    readonly true
  }
  field chapter u32 "The node of the last chapter entered (its id, as `node`); 0: none." {
    readonly true
  }
  field ended bool "The last segment played to its cut and the conversation ended." {
    readonly true
  }
  field held bool "The segment playing stopped at its `flow.hold` marker: it waits for og.flow.advance (or a choice), its window open without time running out." {
    readonly true
  }
  field previous Entity "The deck dissolving out (`og-flow-dissolve`), playing on under the current one; 0: none." {
    readonly true
  }
  field dissolve u32 "The dissolve's length, ticks." {
    readonly true
  }
  field dissolveLeft u32 "Ticks left of the dissolve; 0: none." {
    readonly true
  }
  field qte List.u32.8 "Per choice (ChoiceView.index, the first 8): a quick-time event's progress this window, ticks held or presses." {
    readonly true
  }
}
```

<sub>Module `flow` line 26</sub>
