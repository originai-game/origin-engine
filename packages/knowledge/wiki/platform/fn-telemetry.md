<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-telemetry"></a>

### `og.platform.telemetry`

Record a game event for the game's analytics.

```ts
og.platform.telemetry(world: World, name: string, attrs?: unknown): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `name` | `string` | no | The event: `level.started`, `level.completed`, `boss.defeated` (letters, digits, `.`, `_`; the Player files it as `game.custom.<name>`). |
| `attrs` | `unknown` | yes | Its attributes: an object of at most 16 strings (up to 512 characters), numbers, booleans or null, keys a lowercase letter then letters, digits and `_`. |

**Returns** `Result<void>`: Refused for a malformed name or attributes. Fire and forget: nothing answers.

Binding symbol `og_platform_telemetry` · publish builds: kept

```ogt
fn telemetry {
  doc "Record a game event for the game's analytics."
  param world World "The world."
  param name String "The event: `level.started`, `level.completed`, `boss.defeated` (letters, digits, `.`, `_`; the Player files it as `game.custom.<name>`)."
  param attrs Json "Its attributes: an object of at most 16 strings (up to 512 characters), numbers, booleans or null, keys a lowercase letter then letters, digits and `_`." {
    optional true
  }
  returns Result.Void "Refused for a malformed name or attributes. Fire and forget: nothing answers."
}
```

<sub>Module `platform` line 162</sub>
