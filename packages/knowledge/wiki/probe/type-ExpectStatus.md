<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ExpectStatus"></a>

### `ExpectStatus` (enum)

How an expectation came out.

A string: one of the variants below. The zero value is `"pass"`.

| Variant | Doc |
|---|---|
| `"pass"` | It held. |
| `"fail"` | It did not: the result carries the tick and the observed value. |
| `"unchecked"` | It reads a frame and the host has no renderer. |

```ogt
enum ExpectStatus {
  doc "How an expectation came out."
  variant pass "It held."
  variant fail "It did not: the result carries the tick and the observed value."
  variant unchecked "It reads a frame and the host has no renderer."
}
```

<sub>Module `probe` line 527</sub>
