<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="type-AssertViolation"></a>

### `AssertViolation` (struct)

A site's first violation.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `AssertKind` | `"always"` | — | — | — | The site's kind. |
| `origin` | `AssertOrigin` | `"game"` | — | — | — | Who declared it. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `location` | `string` | `""` | — | — | — | Its place. |
| `tick` | `number` | `0` | — | — | — | The first tick it failed. |
| `entity` | `string \| null` | `null` | — | — | — | The entity a default invariant found it on (its probe id). |
| `component` | `string \| null` | `null` | — | — | — | The record: a component or a resource. |
| `field` | `string \| null` | `null` | — | — | — | The field. |
| `value` | `unknown` | `null` | — | — | — | What was there (a NaN or an infinity as text). |
| `expected` | `unknown` | `null` | — | — | — | What was expected: a range, live, finite, the bounds. |
| `message` | `string` | `""` | — | — | — | What went wrong, in words. |
| `details` | `unknown` | `null` | — | — | — | What the call passed as details. |
| `failures` | `number` | `0` | — | — | — | How many calls violated the site in this report. |
| `lastTick` | `number` | `0` | — | — | — | The last tick it failed. |

```ogt
struct AssertViolation {
  doc "A site's first violation."
  field kind AssertKind "The site's kind."
  field origin AssertOrigin "Who declared it."
  field name String "Its name."
  field location String "Its place."
  field tick u32 "The first tick it failed."
  field entity Option.String "The entity a default invariant found it on (its probe id)."
  field component Option.String "The record: a component or a resource."
  field field Option.String "The field."
  field value Json "What was there (a NaN or an infinity as text)."
  field expected Json "What was expected: a range, live, finite, the bounds."
  field message String "What went wrong, in words."
  field details Json "What the call passed as details."
  field failures u32 "How many calls violated the site in this report."
  field lastTick u32 "The last tick it failed."
}
```

<sub>`api.ogt` line 25848</sub>
