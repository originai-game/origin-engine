<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-WaypointKind"></a>

### `WaypointKind` (enum)

What a waypoint marks.

A string: one of the variants below. The zero value is `"objective"`.

| Variant | Doc |
|---|---|
| `"objective"` | A quest or mode objective (gold). |
| `"marker"` | The player's own marker (info). |
| `"ping"` | A teammate's ping (warn), pulsing; it leaves after its time. |
| `"enemy"` | A spotted enemy (danger). |
| `"ally"` | A teammate (their colour). |
| `"loot"` | Loot (its rarity). |

```ogt
enum WaypointKind {
  doc "What a waypoint marks."
  variant objective "A quest or mode objective (gold)."
  variant marker "The player's own marker (info)."
  variant ping "A teammate's ping (warn), pulsing; it leaves after its time."
  variant enemy "A spotted enemy (danger)."
  variant ally "A teammate (their colour)."
  variant loot "Loot (its rarity)."
}
```

<sub>`api.ogt` line 22357</sub>
