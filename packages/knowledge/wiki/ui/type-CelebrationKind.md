<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CelebrationKind"></a>

### `CelebrationKind` (enum)

Which celebration plays.

A string: one of the variants below. The zero value is `"confetti"`.

| Variant | Doc |
|---|---|
| `"confetti"` | Confetti falls over the screen from its top edge, tumbling and fluttering. |
| `"burst"` | Rays, sparks and a ring thrown from the open container's centre (a reward, a level's star). |
| `"levelUp"` | A level up: a ring of light expands, rays turn behind a big LEVEL UP and the new level, sparks rise. |
| `"fireworks"` | Shells rise and burst in rings across the top of the screen. |

```ogt
enum CelebrationKind {
  doc "Which celebration plays."
  variant confetti "Confetti falls over the screen from its top edge, tumbling and fluttering."
  variant burst "Rays, sparks and a ring thrown from the open container's centre (a reward, a level's star)."
  variant levelUp "A level up: a ring of light expands, rays turn behind a big LEVEL UP and the new level, sparks rise."
  variant fireworks "Shells rise and burst in rings across the top of the screen."
}
```

<sub>Module `ui` line 3186</sub>
