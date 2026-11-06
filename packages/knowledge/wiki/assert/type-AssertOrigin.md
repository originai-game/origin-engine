<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="type-AssertOrigin"></a>

### `AssertOrigin` (enum)

Who declared an assertion.

A string: one of the variants below. The zero value is `"game"`.

| Variant | Doc |
|---|---|
| `"game"` | The game's og.assert calls. |
| `"default"` | The engine's default invariants from reflection. |
| `"playtest"` | A playtest's `invariant:` steps (PlaytestDesc.invariants). |

```ogt
enum AssertOrigin {
  doc "Who declared an assertion."
  variant game "The game's og.assert calls."
  variant default "The engine's default invariants from reflection."
  variant playtest "A playtest's `invariant:` steps (PlaytestDesc.invariants)."
}
```

<sub>Module `assert` line 30</sub>
