<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`audio`](../audio.md) · [index](../index.md)

<a id="type-AudioEventOrder"></a>

### `AudioEventOrder` (enum)

How a AudioEvent chooses its next sound.

A string: one of the variants below. The zero value is `"random"`.

| Variant | Doc |
|---|---|
| `"random"` | A weighted random pick each time (AudioEvent.weights), drawn from the engine's seeded stream. |
| `"roundRobin"` | The sounds in turn, from the first. |
| `"shuffle"` | Random, every sound once before any repeats (and never the last one twice in a row across rounds). |

```ogt
enum AudioEventOrder {
  doc "How a AudioEvent chooses its next sound."
  variant random "A weighted random pick each time (AudioEvent.weights), drawn from the engine's seeded stream."
  variant roundRobin "The sounds in turn, from the first."
  variant shuffle "Random, every sound once before any repeats (and never the last one twice in a row across rounds)."
}
```

<sub>`api.ogt` line 13235</sub>
