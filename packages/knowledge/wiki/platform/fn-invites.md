<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-invites"></a>

### `og.platform.invites`

The invites to this game the player has not acted on, the one the game was opened with marked `opened`.

```ts
og.platform.invites(world: World): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `number`: The answer's data is `{ invites: [{ invite, from: { player, name }, payload, time, opened }] }`.

Binding symbol `og_platform_invites` · publish builds: kept

```ogt
fn invites {
  doc "The invites to this game the player has not acted on, the one the game was opened with marked `opened`."
  param world World "The world."
  returns u32 "The answer's data is `{ invites: [{ invite, from: { player, name }, payload, time, opened }] }`."
}
```

<sub>`api.ogt` line 15240</sub>
