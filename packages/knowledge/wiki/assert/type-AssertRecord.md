<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="type-AssertRecord"></a>

### `AssertRecord` (struct)

One assertion call the runtime reports to the engine (og.assert.record): its first call, a violation, or its first firing in a report.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `AssertKind` | `"always"` | — | — | — | The assertion's kind. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `location` | `string` | `""` | — | — | — | Where it is called (`src/player.ts:42`); empty when the build did not record one. |
| `condition` | `boolean` | `false` | — | — | — | The condition (true for reachable and unreachable, which have none). |
| `key` | `string` | `""` | — | — | — | The bucket of an each, as text; empty otherwise. |
| `details` | `unknown` | `null` | — | — | — | What the call passed as details (null for none). |
| `tick` | `number` | `0` | — | — | — | The tick of the call. |
| `declared` | `boolean` | `false` | — | — | — | A site the build found, sent before it runs: listed, not counted as a call. |

```ogt
struct AssertRecord {
  doc "One assertion call the runtime reports to the engine (og.assert.record): its first call, a violation, or its first firing in a report."
  field kind AssertKind "The assertion's kind."
  field name String "Its name."
  field location String "Where it is called (`src/player.ts:42`); empty when the build did not record one."
  field condition bool "The condition (true for reachable and unreachable, which have none)."
  field key String "The bucket of an each, as text; empty otherwise."
  field details Json "What the call passed as details (null for none)."
  field tick u32 "The tick of the call."
  field declared bool "A site the build found, sent before it runs: listed, not counted as a call."
}
```

<sub>`api.ogt` line 25848</sub>
