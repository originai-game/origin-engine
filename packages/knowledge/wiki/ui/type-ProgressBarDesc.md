<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ProgressBarDesc"></a>

### `ProgressBarDesc` (desc struct)

A progress bar.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `number` | `0` | 0 … 1 | `ratio` | — | Done, 0 to 1. |
| `buffer` | `number` | `0` | 0 … 1 | `ratio` | — | A second, paler fill ahead of value (downloaded, ahead of installed), 0 to 1. |
| `style` | `ProgressStyle` | `"bar"` | — | — | — | How it shows. |
| `steps` | `number` | `0` | — | — | — | For style steps: how many; 0 is 5. |
| `label` | `string` | `""` | — | — | — | A caption over it at the start (Downloading); empty is none. |
| `valueText` | `string` | `""` | — | — | — | The value in words at the end (2.1 of 4.5 GB); empty is the percent, with showValue. |
| `showValue` | `boolean` | `false` | — | — | — | Show the value at the end. |
| `tone` | `Tone` | `"neutral"` | — | — | — | Its fill's tone; neutral is the primary. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `thickness` | `number` | `0` | ≥ 0 | `cssPx` | — | The track's height; 0 is 8 (6 on a phone). |
| `sheen` | `boolean` | `true` | — | — | — | A highlight sweeps over the fill while it is short of done (still under reduced motion it stops). |

```ogt
struct ProgressBarDesc {
  doc "A progress bar."
  desc true
  field value f32 "Done, 0 to 1." {
    unit ratio
    min 0
    max 1
  }
  field buffer f32 "A second, paler fill ahead of value (downloaded, ahead of installed), 0 to 1." {
    unit ratio
    min 0
    max 1
  }
  field style ProgressStyle "How it shows."
  field steps u32 "For style steps: how many; 0 is 5."
  field label String "A caption over it at the start (Downloading); empty is none."
  field valueText String "The value in words at the end (2.1 of 4.5 GB); empty is the percent, with showValue."
  field showValue bool "Show the value at the end."
  field tone Tone "Its fill's tone; neutral is the primary."
  field width Option.Size "Its width; null grows."
  field thickness f32 "The track's height; 0 is 8 (6 on a phone)." {
    unit cssPx
    min 0
  }
  field sheen bool "A highlight sweeps over the fill while it is short of done (still under reduced motion it stops)." {
    default true
  }
}
```

<sub>Module `ui` line 5368</sub>
