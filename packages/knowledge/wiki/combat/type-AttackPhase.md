<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`combat`](../combat.md) · [index](../index.md)

<a id="type-AttackPhase"></a>

### `AttackPhase` (enum)

Where an og.Attack is in its move.

A string: one of the variants below. The zero value is `"idle"`.

| Variant | Doc |
|---|---|
| `"idle"` | No move. |
| `"startup"` | Before the move's active window. |
| `"active"` | In its active window: its hit volumes strike. |
| `"recovery"` | After the active window, until the move ends. |

```ogt
enum AttackPhase {
  doc "Where an og.Attack is in its move."
  variant idle "No move."
  variant startup "Before the move's active window."
  variant active "In its active window: its hit volumes strike."
  variant recovery "After the active window, until the move ends."
}
```

<sub>Module `combat` line 4</sub>
