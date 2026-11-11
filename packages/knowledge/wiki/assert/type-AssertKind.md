<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="type-AssertKind"></a>

### `AssertKind` (enum)

What an assertion asks.

A string: one of the variants below. The zero value is `"always"`.

| Variant | Doc |
|---|---|
| `"always"` | Its condition holds on every call: a false one is a violation. |
| `"sometimes"` | Its condition holds on at least one call in the run. |
| `"reachable"` | It runs at least once in the run. |
| `"unreachable"` | It never runs: a call is a violation. |
| `"each"` | Its condition holds once for each distinct key it was called with. |

```ogt
enum AssertKind {
  doc "What an assertion asks."
  variant always "Its condition holds on every call: a false one is a violation."
  variant sometimes "Its condition holds on at least one call in the run."
  variant reachable "It runs at least once in the run."
  variant unreachable "It never runs: a call is a violation."
  variant each "Its condition holds once for each distinct key it was called with."
}
```

<sub>Module `assert` line 8</sub>
