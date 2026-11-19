<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-SharedOpAnswered"></a>

### `og.SharedOpAnswered` (event)

The server's answer to this client's og.shared.put or og.shared.remove. Refused when the object's version was not the one expected, it lies outside this visitor's sight, or ops came faster than the server's events a second.

Token `og.SharedOpAnswered`, registered under the schema name `og.SharedOpAnswered`; its value type is `SharedOpAnswered`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `op` | `number` | `0` | — | — | — | The op, as og.shared.put or og.shared.remove returned it. |
| `taken` | `boolean` | `false` | — | — | — | Whether the server applied it. |
| `version` | `number` | `0` | — | — | — | The object's version now (0: there is none). |

```ogt
event SharedOpAnswered {
  doc "The server's answer to this client's og.shared.put or og.shared.remove. Refused when the object's version was not the one expected, it lies outside this visitor's sight, or ops came faster than the server's events a second."
  field op u32 "The op, as og.shared.put or og.shared.remove returned it."
  field taken bool "Whether the server applied it."
  field version u32 "The object's version now (0: there is none)."
}
```

<sub>Module `shared` line 134</sub>
