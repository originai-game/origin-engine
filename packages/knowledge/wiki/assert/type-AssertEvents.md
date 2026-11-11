<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="type-AssertEvents"></a>

### `AssertEvents` (struct)

The assertions' stream since the last read.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `epoch` | `number` | `0` | — | — | — | The report the events belong to now (take moves it on). |
| `events` | `readonly AssertEvent[]` | `[]` | — | — | — | In the order they happened. |
| `dropped` | `number` | `0` | — | — | — | Events past the 4,096 held between reads. |

```ogt
struct AssertEvents {
  doc "The assertions' stream since the last read."
  field epoch u32 "The report the events belong to now (take moves it on)."
  field events List.AssertEvent "In the order they happened."
  field dropped u32 "Events past the 4,096 held between reads."
}
```

<sub>Module `assert` line 132</sub>
