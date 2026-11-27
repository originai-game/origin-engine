<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`interact`](../interact.md) · [index](../index.md)

<a id="type-InteractRule"></a>

### `InteractRule` (enum)

How an og.Interactor picks its focus among the candidates in range, in its cone and in sight.

A string: one of the variants below. The zero value is `"nearest"`.

| Variant | Doc |
|---|---|
| `"nearest"` | The nearest; equal distances in entity order. |
| `"centred"` | The smallest angle off its forward, then the nearest. |
| `"priority"` | The highest og.Interactable.priority, then the nearest. |

```ogt
enum InteractRule {
  doc "How an og.Interactor picks its focus among the candidates in range, in its cone and in sight."
  variant nearest "The nearest; equal distances in entity order."
  variant centred "The smallest angle off its forward, then the nearest."
  variant priority "The highest og.Interactable.priority, then the nearest."
}
```

<sub>Module `interact` line 4</sub>
