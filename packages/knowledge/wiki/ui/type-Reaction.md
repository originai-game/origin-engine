<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Reaction"></a>

### `Reaction` (enum)

How a unit stands toward the player, which colours its tag.

A string: one of the variants below. The zero value is `"hostile"`.

| Variant | Doc |
|---|---|
| `"hostile"` | An enemy (danger). |
| `"neutral"` | Neither (warn). |
| `"friendly"` | An ally or a friendly NPC (success). |
| `"player"` | Another player (info). |
| `"self"` | The player (primary). |

```ogt
enum Reaction {
  doc "How a unit stands toward the player, which colours its tag."
  variant hostile "An enemy (danger)."
  variant neutral "Neither (warn)."
  variant friendly "An ally or a friendly NPC (success)."
  variant player "Another player (info)."
  variant self "The player (primary)."
}
```

<sub>Module `ui` line 6411</sub>
