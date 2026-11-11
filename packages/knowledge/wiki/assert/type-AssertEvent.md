<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="type-AssertEvent"></a>

### `AssertEvent` (struct)

One event of the assertions' stream (probe.assertionEvents): a first firing or a first violation.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `outcome` | `string` | `""` | — | — | — | `fired` (a sometimes held, a reachable ran, an each's bucket held) or `violated` (an always failed, an unreachable ran), each once per site or bucket in a report. |
| `tick` | `number` | `0` | — | — | — | The tick it happened at, numbered as AssertViolation.tick. |
| `kind` | `AssertKind` | `"always"` | — | — | — | The site's kind. |
| `origin` | `AssertOrigin` | `"game"` | — | — | — | Who declared it. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `location` | `string` | `""` | — | — | — | Its place. |
| `key` | `string` | `""` | — | — | optional | An each's bucket. |
| `details` | `unknown` | `null` | — | — | — | What the call passed as details. |
| `entity` | `string \| null` | `null` | — | — | optional | A violation's entity (a default invariant). |
| `component` | `string \| null` | `null` | — | — | optional | A violation's record. |
| `field` | `string \| null` | `null` | — | — | optional | A violation's field. |
| `value` | `unknown` | `null` | — | — | optional | A violation's value. |
| `message` | `string` | `""` | — | — | optional | A violation in words. |

```ogt
struct AssertEvent {
  doc "One event of the assertions' stream (probe.assertionEvents): a first firing or a first violation."
  field outcome String "`fired` (a sometimes held, a reachable ran, an each's bucket held) or `violated` (an always failed, an unreachable ran), each once per site or bucket in a report."
  field tick u32 "The tick it happened at, numbered as AssertViolation.tick."
  field kind AssertKind "The site's kind."
  field origin AssertOrigin "Who declared it."
  field name String "Its name."
  field location String "Its place."
  field key String "An each's bucket." {
    optional true
  }
  field details Json "What the call passed as details."
  field entity Option.String "A violation's entity (a default invariant)." {
    optional true
  }
  field component Option.String "A violation's record." {
    optional true
  }
  field field Option.String "A violation's field." {
    optional true
  }
  field value Json "A violation's value." {
    optional true
  }
  field message String "A violation in words." {
    optional true
  }
}
```

<sub>Module `assert` line 103</sub>
