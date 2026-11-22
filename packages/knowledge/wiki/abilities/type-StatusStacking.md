<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`abilities`](../abilities.md) · [index](../index.md)

<a id="type-StatusStacking"></a>

### `StatusStacking` (enum)

What a second og.Status of the same kind on the same target does to the first.

A string: one of the variants below. The zero value is `"refresh"`.

| Variant | Doc |
|---|---|
| `"refresh"` | The first stays and takes the second's ticks; the second goes. |
| `"stack"` | The first stays, adds the second's stacks up to maxStacks and takes its ticks; the second goes. |
| `"strongest"` | The stronger stays (a tie keeps the first, with the second's ticks); the other goes. |

```ogt
enum StatusStacking {
  doc "What a second og.Status of the same kind on the same target does to the first."
  variant refresh "The first stays and takes the second's ticks; the second goes."
  variant stack "The first stays, adds the second's stacks up to maxStacks and takes its ticks; the second goes."
  variant strongest "The stronger stays (a tie keeps the first, with the second's ticks); the other goes."
}
```

<sub>Module `abilities` line 5</sub>
