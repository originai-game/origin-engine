<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`path`](../path.md) · [index](../index.md)

<a id="fn-find"></a>

### `og.path.find`

A path a loaded scene declares (`path "<name>" { ... }`), by name.

```ts
og.path.find(world: World, name: string): Result<Path>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The path's name in its scene. |

**Returns** `Result<Path>`: Refused for a name no spawned scene declares; the reason lists the names.

Binding symbol `og_path_find` · publish builds: kept

```ogt
fn find {
  doc "A path a loaded scene declares (`path \"<name>\" { ... }`), by name."
  param world World "The world."
  param name String "The path's name in its scene."
  returns Result.Path "Refused for a name no spawned scene declares; the reason lists the names."
}
```

<sub>`api.ogt` line 25984</sub>
