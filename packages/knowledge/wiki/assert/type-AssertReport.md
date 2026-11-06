<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="type-AssertReport"></a>

### `AssertReport` (struct)

The run's assertions since the report began (the probe's assertions member).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `epoch` | `number` | `0` | — | — | — | The report's number: take moves it on. |
| `failures` | `number` | `0` | — | — | — | Violating calls, summed over the sites. |
| `violations` | `readonly AssertViolation[]` | `[]` | — | — | — | Each violated site's first violation, in the order they happened. |
| `uncovered` | `readonly AssertUncovered[]` | `[]` | — | — | — | What never fired: a run whose sometimes never held did not test what it names. |
| `sites` | `readonly AssertSite[]` | `[]` | — | — | — | Every site seen or declared (default invariants once they fail), each with its buckets: the hints a search of the game's states starts from. |
| `sitesDropped` | `number` | `0` | — | — | — | Sites past the 4,096 kept. |
| `defaults` | `AssertCost` | `{ fields: 0, ticks: 0, values: 0, totalMs: 0, meanUs: 0, maxUs: 0 }` | — | — | — | The default invariants' cost. |

```ogt
struct AssertReport {
  doc "The run's assertions since the report began (the probe's assertions member)."
  field epoch u32 "The report's number: take moves it on."
  field failures u32 "Violating calls, summed over the sites."
  field violations List.AssertViolation "Each violated site's first violation, in the order they happened."
  field uncovered List.AssertUncovered "What never fired: a run whose sometimes never held did not test what it names."
  field sites List.AssertSite "Every site seen or declared (default invariants once they fail), each with its buckets: the hints a search of the game's states starts from."
  field sitesDropped u32 "Sites past the 4,096 kept."
  field defaults AssertCost "The default invariants' cost."
}
```

<sub>`api.ogt` line 25935</sub>
