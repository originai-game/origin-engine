<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CelebrationDesc"></a>

### `CelebrationDesc` (desc struct)

A celebration effect.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `CelebrationKind` | `"confetti"` | — | — | — | Which one. |
| `at` | `number` | `0` | — | — | — | og.ui.tick() when it fires: a later tick fires it again. |
| `colors` | `readonly Paint[]` | `[]` | — | — | — | Its colours; empty is the primary, gold and the style's chart series. |
| `intensity` | `number` | `1` | 0 … 3 | — | — | How much: confetti count and burst reach scale by it. |
| `title` | `string` | `"Level up"` | — | — | — | levelUp: the big word (LEVEL UP). |
| `value` | `string` | `""` | — | — | — | levelUp: the new level (12). |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long it lasts; 0 is the kind's (confetti 3 s, burst 1 s, levelUp 2.6 s, fireworks 3.5 s). |

```ogt
struct CelebrationDesc {
  doc "A celebration effect."
  desc true
  field kind CelebrationKind "Which one."
  field at u32 "og.ui.tick() when it fires: a later tick fires it again."
  field colors List.Paint "Its colours; empty is the primary, gold and the style's chart series."
  field intensity f32 "How much: confetti count and burst reach scale by it." {
    default 1
    min 0
    max 3
  }
  field title String "levelUp: the big word (LEVEL UP)." {
    default "Level up"
  }
  field value String "levelUp: the new level (12)."
  field durationMs f32 "How long it lasts; 0 is the kind's (confetti 3 s, burst 1 s, levelUp 2.6 s, fireworks 3.5 s)." {
    unit ms
    min 0
  }
}
```

<sub>Module `ui` line 5659</sub>
