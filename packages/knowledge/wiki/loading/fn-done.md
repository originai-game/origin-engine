<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`loading`](../loading.md) · [index](../index.md)

<a id="fn-done"></a>

### `og.loading.done`

The game's loading is done: a `page` screen uncovers the canvas, the page hears `og:loaded`, and with `game` the play gate (if any) shows. Call it once og.loading.idle is true, so the first frame shown draws the whole scene. Calling it again does nothing.

See `loading §13`.

```ts
og.loading.done(world: World): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `Result<void>`.

Binding symbol `og_loading_done` · publish builds: kept

```ogt
fn done {
  doc "The game's loading is done: a `page` screen uncovers the canvas, the page hears `og:loaded`, and with `game` the play gate (if any) shows. Call it once og.loading.idle is true, so the first frame shown draws the whole scene. Calling it again does nothing."
  guide "loading §13"
  param world World "The world."
  returns Result.Void
}
```

<sub>Module `loading` line 42</sub>
