<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-friends"></a>

### `og.platform.friends`

The player's friends, those playing this game first.

```ts
og.platform.friends(world: World, desc?: FriendsDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `FriendsDesc` | yes | How many. |

**Returns** `number`: The answer's data is `{ friends: [{ player, name, online, playing }] }`, ids scoped to this game; `unauthorized` for a guest.

Binding symbol `og_platform_friends` · publish builds: kept

```ogt
fn friends {
  doc "The player's friends, those playing this game first."
  param world World "The world."
  param desc FriendsDesc "How many." {
    optional true
  }
  returns u32 "The answer's data is `{ friends: [{ player, name, online, playing }] }`, ids scoped to this game; `unauthorized` for a guest."
}
```

<sub>`api.ogt` line 15285</sub>
