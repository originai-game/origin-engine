<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SettingsCustomBuild"></a>

### `SettingsCustomBuild` (callback)

Builds a custom settings entry's control inside its row: ui calls made inside nest under it.

Game code supplies this function.

```ts
type SettingsCustomBuild = (key: string) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `key` | `string` | no | The entry's key. |

```ogt
callback SettingsCustomBuild {
  doc "Builds a custom settings entry's control inside its row: ui calls made inside nest under it."
  param key String "The entry's key."
}
```

<sub>Module `ui` line 4178</sub>
