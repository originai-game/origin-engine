<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ScreenTransitionDesc"></a>

### `ScreenTransitionDesc` (desc struct)

How a screen transition looks.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `ScreenWipe` | `"fade"` | — | — | — | The movement. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | Cover and reveal together; 0 is twice motion.duration.slow. |
| `color` | `Paint \| null` | `null` | — | — | — | The cover's colour (the texture's tint); null is backdrop. |
| `texture` | `Texture \| null` | `null` | — | — | — | A texture stretched over the surface as the cover; null is the flat colour. |
| `from` | `WipeSide` | `"left"` | — | — | — | Where a wipe or a slide comes from. |

```ogt
struct ScreenTransitionDesc {
  doc "How a screen transition looks."
  desc true
  field kind ScreenWipe "The movement."
  field durationMs f32 "Cover and reveal together; 0 is twice motion.duration.slow." {
    unit ms
    min 0
  }
  field color Option.Paint "The cover's colour (the texture's tint); null is backdrop."
  field texture Option.Texture "A texture stretched over the surface as the cover; null is the flat colour."
  field from WipeSide "Where a wipe or a slide comes from."
}
```

<sub>Module `ui` line 1902</sub>
