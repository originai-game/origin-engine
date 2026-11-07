<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BannerDesc"></a>

### `BannerDesc` (desc struct)

A full-width announcement.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `BannerKind` | `"title"` | — | — | — | What it announces. |
| `title` | `string` | `""` | — | — | — | The big line. |
| `eyebrow` | `string` | `""` | — | — | — | A small line over it (Chapter II, Wave). |
| `subtitle` | `string` | `""` | — | — | — | A line under it. |
| `shownAt` | `number` | `0` | — | — | — | og.ui.tick() when it should start: a later tick restarts it. Past its time it is gone. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long it holds before it leaves; 0 is the kind's. |
| `anchor` | `Anchor` | `"center"` | — | — | — | Where it sits; the default is the centre. |
| `image` | `Texture \| null` | `null` | — | — | — | An art slot behind the title (a boss portrait, a region's emblem; banner kind, about 4:1). |

```ogt
struct BannerDesc {
  doc "A full-width announcement."
  desc true
  field kind BannerKind "What it announces."
  field title String "The big line."
  field eyebrow String "A small line over it (Chapter II, Wave)."
  field subtitle String "A line under it."
  field shownAt u32 "og.ui.tick() when it should start: a later tick restarts it. Past its time it is gone."
  field durationMs f32 "How long it holds before it leaves; 0 is the kind's." {
    unit ms
    min 0
  }
  field anchor Anchor "Where it sits; the default is the centre." {
    default center
  }
  field image Option.Texture "An art slot behind the title (a boss portrait, a region's emblem; banner kind, about 4:1)."
}
```

<sub>Module `ui` line 5677</sub>
