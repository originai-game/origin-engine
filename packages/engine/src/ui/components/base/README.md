# Items and identity

Small, composable documents for inventories and player identity. Every prop
has a default. They display game-owned data; they do not consume items,
advance cooldowns, grant levels or infer presence.

| Component | Inputs | Intents |
| --- | --- | --- |
| `ItemSlot` | `name`, `icon`, `count`, `rarity`, `cooldown` and `cooldownMax` in seconds, `keyLabel`, `selected`, `disabled`, `locked`, `loading`, `empty`, `showName` | `choose`; keeps an enclosing keyed row's id. Locked, disabled and loading slots do not activate. A cooling or empty slot can be selected for inspection or equipping. |
| `Avatar` | `name`, `portrait` texture, fallback `icon`, `level`, `status` (online, away, busy, offline), optional `statusText`, style-pack `frame` key, `loading` | None; wrap it in a game-owned action when identity is interactive. |

`--item-slot-size`, `--avatar-size` and `--avatar-radius` control their
dimensions. Every visible piece is a named part. Rarity, frame and status
are exposed as data attributes; presence is also worded so it does not rely
on colour. Native buttons supply focus, hover, press and keyboard/pad
activation. A pack may replace the look without changing the documents.

Review: the RPG sheet's `primitives` page in the default style shows ready,
selected/cooling, locked, unavailable, loading and empty slots; portrait and
fallback avatars, level, four presence states and loading.
