<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-BannerKind"></a>

### `BannerKind` (enum)

What a banner announces, which sets its look and timing.

A string: one of the variants below. The zero value is `"title"`.

| Variant | Doc |
|---|---|
| `"title"` | A title card: an area or chapter name over a thin rule (Elden Ring's place names), slow fade (3 s). |
| `"wave"` | A wave or round: a band across the screen with a large title that slams in (2.4 s). |
| `"victory"` | A win: a gold band with a burst (3.5 s); plays success. |
| `"defeat"` | A loss: a dark band in danger (3.5 s); plays error. |
| `"warning"` | A warning across the screen (Boss approaching): danger stripes that pulse (3 s); plays warning. |

```ogt
enum BannerKind {
  doc "What a banner announces, which sets its look and timing."
  variant title "A title card: an area or chapter name over a thin rule (Elden Ring's place names), slow fade (3 s)."
  variant wave "A wave or round: a band across the screen with a large title that slams in (2.4 s)."
  variant victory "A win: a gold band with a burst (3.5 s); plays success."
  variant defeat "A loss: a dark band in danger (3.5 s); plays error."
  variant warning "A warning across the screen (Boss approaching): danger stripes that pulse (3 s); plays warning."
}
```

<sub>Module `ui` line 5666</sub>
