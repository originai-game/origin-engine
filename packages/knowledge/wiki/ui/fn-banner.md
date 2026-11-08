<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-banner"></a>

### `og.ui.banner`

A full-width announcement: a wave or round band that slams in with a squash and a shake, a title card that fades up over a rule that draws out from its centre, a victory band with a burst and confetti, a defeat band, a warning with sweeping danger stripes. Its letters track in as it enters; it holds, then leaves on the style's exit ease. The tick-driven timing makes it the same in a replay.

Parts: Banner.band[<kind>], Banner.rule, Banner.eyebrow, Banner.title, Banner.subtitle, Banner.stripes, Banner.art. Slots: image (art kind banner). States: group phase (entering, holding, leaving, gone). Sounds: by kind on entering. Accessibility: alert (warning, defeat) or status, named by eyebrow, title and subtitle. Probe ids: <id>, <id>/Title, <id>/Eyebrow, <id>/Subtitle.

```ts
og.ui.banner(id: string, desc?: BannerDesc): BannerResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `BannerDesc` | yes | What it says and when. |

**Returns** `BannerResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn banner {
  doc "A full-width announcement: a wave or round band that slams in with a squash and a shake, a title card that fades up over a rule that draws out from its centre, a victory band with a burst and confetti, a defeat band, a warning with sweeping danger stripes. Its letters track in as it enters; it holds, then leaves on the style's exit ease. The tick-driven timing makes it the same in a replay."
  doc "Parts: Banner.band[<kind>], Banner.rule, Banner.eyebrow, Banner.title, Banner.subtitle, Banner.stripes, Banner.art. Slots: image (art kind banner). States: group phase (entering, holding, leaving, gone). Sounds: by kind on entering. Accessibility: alert (warning, defeat) or status, named by eyebrow, title and subtitle. Probe ids: <id>, <id>/Title, <id>/Eyebrow, <id>/Subtitle."
  host false
  param id String "Its id."
  param desc BannerDesc "What it says and when."
  returns BannerResult
}
```

<sub>Module `ui` line 5702</sub>
