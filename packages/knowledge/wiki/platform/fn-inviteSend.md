<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-inviteSend"></a>

### `og.platform.inviteSend`

Invite a friend to this game, or offer an invite link through the share sheet. The invited player opens the game with the invite in og.platform.invites.

```ts
og.platform.inviteSend(world: World, desc?: InviteDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `InviteDesc` | yes | Whom, the words and what the game hands them. |

**Returns** `Result<number>`: Refused for a malformed friend id, a message over 120 characters or a payload over 1 KiB. The answer's data is `{ invite, to }` (to null for a link); `cancelled` when the player closed the share sheet.

Binding symbol `og_platform_invite_send` · publish builds: kept

```ogt
fn inviteSend {
  doc "Invite a friend to this game, or offer an invite link through the share sheet. The invited player opens the game with the invite in og.platform.invites."
  param world World "The world."
  param desc InviteDesc "Whom, the words and what the game hands them." {
    optional true
  }
  returns Result.u32 "Refused for a malformed friend id, a message over 120 characters or a payload over 1 KiB. The answer's data is `{ invite, to }` (to null for a link); `cancelled` when the player closed the share sheet."
}
```

<sub>Module `platform` line 409</sub>
