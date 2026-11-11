<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-MotionGoalKind"></a>

### `MotionGoalKind` (enum)

What a goal asks for.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No goal: the character holds where its last goal left it. |
| `"pose"` | A library pose where the character stands, keeping its facing. |
| `"place"` | A library pose at a place, facing a heading. |
| `"face"` | Turn to a heading where the character stands. |
| `"walkTo"` | Walk or run to a point, then take a pose. |
| `"jump"` | Jump: prepare, take off, fly the simulation's ballistic arc, land and recover. |

```ogt
enum MotionGoalKind {
  doc "What a goal asks for."
  variant none "No goal: the character holds where its last goal left it."
  variant pose "A library pose where the character stands, keeping its facing."
  variant place "A library pose at a place, facing a heading."
  variant face "Turn to a heading where the character stands."
  variant walkTo "Walk or run to a point, then take a pose."
  variant jump "Jump: prepare, take off, fly the simulation's ballistic arc, land and recover."
}
```

<sub>Module `motion` line 6</sub>
