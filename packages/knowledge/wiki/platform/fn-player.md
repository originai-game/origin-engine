<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-player"></a>

### `og.platform.player`

Who is playing: an id scoped to this game (the same player has another id in another game) and the name to show.

```ts
og.platform.player(world: World): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `number`: The answer's data is `{ player, name, guest }`, guest true for a player without an account.

Binding symbol `og_platform_player` · publish builds: kept

```ogt
fn player {
  doc "Who is playing: an id scoped to this game (the same player has another id in another game) and the name to show."
  param world World "The world."
  returns u32 "The answer's data is `{ player, name, guest }`, guest true for a player without an account."
}
```

<sub>Module `platform` line 394</sub>
