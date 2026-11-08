<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-spotlight"></a>

### `og.ui.spotlight`

A spotlight over everything (a tutorial step): the scrim on four sides of the hole, a primary edge around it. Presses on the dimmed parts stop there (clicked); presses in the hole reach what is under it. Shades are `<id>/Top`, `Left`, `Right`, `Bottom`; the hole is `<id>/Hole`.

```ts
og.ui.spotlight(id: string, hole: UiRect | null, desc?: SpotlightDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `hole` | `UiRect \| null` | no | The rectangle to leave lit, as og.ui.bounds answers; null dims everything. |
| `desc` | `SpotlightDesc` | yes | Room around the hole. |

**Returns** `Response`.

Binding symbol `og_ui_spotlight` · publish builds: kept

```ogt
fn spotlight {
  doc "A spotlight over everything (a tutorial step): the scrim on four sides of the hole, a primary edge around it. Presses on the dimmed parts stop there (clicked); presses in the hole reach what is under it. Shades are `<id>/Top`, `Left`, `Right`, `Bottom`; the hole is `<id>/Hole`."
  param id String "Its id."
  param hole Option.UiRect "The rectangle to leave lit, as og.ui.bounds answers; null dims everything."
  param desc SpotlightDesc "Room around the hole."
  returns Response
}
```

<sub>Module `ui` line 1284</sub>
