<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`time`](../time.md) · [index](../index.md)

<a id="record-TimeGroup"></a>

### `og.TimeGroup` (component)

Which time group the entity is in (0 to 15): its animation (og.anim, flipbooks, 2D skeletons), its scaled tweens and its og.nav agent advance by scaledDt times og.time.setGroupScale's scale for the group, and its og.ParticleSystem's particles spawn, age and move at the group's scale alone (they follow neither the global scale nor hit stop). Physics stays on the global scale: a body in a slowed group falls at the world's pace. An entity without one is at the global pace.

Token `og.TimeGroup`, registered under the schema name `og.TimeGroup`; its value type is `TimeGroup`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `group` | `number` | `0` | ≤ 15 | — | — | The group, 0 to 15. |

```ogt
component TimeGroup {
  doc "Which time group the entity is in (0 to 15): its animation (og.anim, flipbooks, 2D skeletons), its scaled tweens and its og.nav agent advance by scaledDt times og.time.setGroupScale's scale for the group, and its og.ParticleSystem's particles spawn, age and move at the group's scale alone (they follow neither the global scale nor hit stop). Physics stays on the global scale: a body in a slowed group falls at the world's pace. An entity without one is at the global pace."
  field group u32 "The group, 0 to 15." {
    max 15
  }
}
```

<sub>Module `time` line 44</sub>
