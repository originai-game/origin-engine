<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-loadSettings"></a>

### `og.platform.loadSettings`

Ask for the game's saved settings (volume, controls, language): store.get of the key `og.settings`. Where the Player has no store, or is offline, the shell answers from the browser's own storage; natively, from the local store.

```ts
og.platform.loadSettings(world: World): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `number`: The request; its answer's data is `{ key, value, revision }` (value null before the first save).

Binding symbol `og_platform_load_settings` · publish builds: kept

```ogt
fn loadSettings {
  doc "Ask for the game's saved settings (volume, controls, language): store.get of the key `og.settings`. Where the Player has no store, or is offline, the shell answers from the browser's own storage; natively, from the local store."
  param world World "The world."
  returns u32 "The request; its answer's data is `{ key, value, revision }` (value null before the first save)."
}
```

<sub>`api.ogt` line 15071</sub>
