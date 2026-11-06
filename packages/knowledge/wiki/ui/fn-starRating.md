<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-starRating"></a>

### `og.ui.starRating`

Stars earned: empty sockets, each earned star landing in turn from large with a turn and an overshoot (emphasis spring), a burst of sparks as it lands and a pitch rising star by star; the middle star of three sits higher and larger. Under reduced motion they fade in.

Parts: StarRating.socket, StarRating.star[earned]. States: on each star, group star (empty, landing, earned). Sounds: reward, pitched up a step per star. Accessibility: image named by name. Probe ids: <id>, <id>/Star#i.

```ts
og.ui.starRating(id: string, desc?: StarRatingDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `StarRatingDesc` | yes | How many and when. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn starRating {
  doc "Stars earned: empty sockets, each earned star landing in turn from large with a turn and an overshoot (emphasis spring), a burst of sparks as it lands and a pitch rising star by star; the middle star of three sits higher and larger. Under reduced motion they fade in."
  doc "Parts: StarRating.socket, StarRating.star[earned]. States: on each star, group star (empty, landing, earned). Sounds: reward, pitched up a step per star. Accessibility: image named by name. Probe ids: <id>, <id>/Star#i."
  host false
  param id String "Its id."
  param desc StarRatingDesc "How many and when."
  returns Void
}
```

<sub>`api.ogt` line 21466</sub>
