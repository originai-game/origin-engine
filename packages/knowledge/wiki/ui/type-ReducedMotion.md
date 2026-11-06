<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ReducedMotion"></a>

### `ReducedMotion` (enum)

What an animation does under reduced motion (docs/ui.md §7.7).

A string: one of the variants below. The zero value is `"fade"`.

| Variant | Doc |
|---|---|
| `"fade"` | Movement and scale jump; opacity and colour fade quickly (the default). |
| `"none"` | Everything jumps. |
| `"keep"` | Plays as authored: motion that carries meaning, kept small. |

```ogt
enum ReducedMotion {
  doc "What an animation does under reduced motion (docs/ui.md §7.7)."
  variant fade "Movement and scale jump; opacity and colour fade quickly (the default)."
  variant none "Everything jumps."
  variant keep "Plays as authored: motion that carries meaning, kept small."
}
```

<sub>`api.ogt` line 20162</sub>
