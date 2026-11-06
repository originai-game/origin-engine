<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ExpectOperation"></a>

### `ExpectOperation` (struct)

The expect operation's payload.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `what` | `string` | `""` | — | — | — | The expectation, as after `expect:` in a script, such as `state:started==true`. |

```ogt
struct ExpectOperation {
  doc "The expect operation's payload."
  field what String "The expectation, as after `expect:` in a script, such as `state:started==true`."
}
```

<sub>Module `probe` line 125</sub>
