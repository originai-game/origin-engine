<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-saveSettings"></a>

### `og.platform.saveSettings`

Save the game's settings under `og.settings` (the last save wins), with the same local fallback as loadSettings.

Also known as: preferences, PlayerPrefs, settings persistence, options, config file

```ts
og.platform.saveSettings(world: World, value: unknown): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `value` | `unknown` | no | The settings: an object, at most 64 KiB encoded. |

**Returns** `Result<number>`: Refused for a value that is not an object or is too large.

Binding symbol `og_platform_save_settings` · publish builds: kept

```ogt
fn saveSettings {
  doc "Save the game's settings under `og.settings` (the last save wins), with the same local fallback as loadSettings."
  aliases "preferences, PlayerPrefs, settings persistence, options, config file"
  param world World "The world."
  param value Json "The settings: an object, at most 64 KiB encoded."
  returns Result.u32 "Refused for a value that is not an object or is too large."
}
```

<sub>`api.ogt` line 15091</sub>
