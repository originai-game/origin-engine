<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-PosterizeMode"></a>

### `PosterizeMode` (enum)

How Stylise.posterize steps a colour.

A string: one of the variants below. The zero value is `"channel"`.

| Variant | Doc |
|---|---|
| `"channel"` | Each display channel rounded to its own step, the classic posterize: a colour off the grey axis can change hue as light moves it across one channel's step before the others. |
| `"tone"` | The colour's value (its largest channel) and its tint (the channels over that value) stepped apart, each to posterize's steps: a change of light, exposure or mood moves a colour's value and never its hue, so a near-grey stays grey and a gradient of one hue stays that hue. |

```ogt
enum PosterizeMode {
  doc "How Stylise.posterize steps a colour."
  variant channel "Each display channel rounded to its own step, the classic posterize: a colour off the grey axis can change hue as light moves it across one channel's step before the others."
  variant tone "The colour's value (its largest channel) and its tint (the channels over that value) stepped apart, each to posterize's steps: a change of light, exposure or mood moves a colour's value and never its hue, so a near-grey stays grey and a gradient of one hue stays that hue."
}
```

<sub>`api.ogt` line 5216</sub>
