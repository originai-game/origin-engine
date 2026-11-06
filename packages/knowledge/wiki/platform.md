<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `platform` module

Platform services as host capabilities (docs/design.md §2.8): cloud store, leaderboards, telemetry, Origin Point and AI marks. Game code names a method and its arguments, never a credential, a URL or a cookie: the host owns them. On the web the shell forwards each call to the Player through origin-checked postMessage (Player protocol 2, docs/web-export.md); natively the host answers from a local JSON store (`og run --platform-store`, default `<project>/.og/platform.json`).

Calls are asynchronous: each returns a request number at once, and the answer arrives at the start of a later tick, when og.platform sends PlatformAnswered; read it with og.platform.result. Answers are host input, like key presses: the probe's `dump_platform` lists every call, its arguments, its answer and the ticks.

Errors carry the Player's stable codes: offline, timeout, invalid, unauthorized, not_found, conflict, too_large, rate_limited, unavailable, failed, cancelled (the player backed out of a purchase, an ad or a share sheet) and no_fill (no ad to show now); retryable is true for offline, timeout, unavailable, rate_limited and no_fill.

Purchases, ads, achievements, the player, friends, invites and share (products, purchase, showAd, achievementUnlock, player, friends, inviteSend, share, ...) are the platform's: accounts and money never enter the engine. Each call is a method name and its arguments, the host attaches the player's game-scoped credential, and every id it hands back (the player, a friend, a purchase, an invite) is opaque and scoped to this game. A host that lacks one answers it `unavailable`: ask og.platform.supports first. Natively and in playtests a deterministic fake answers them (`og run --platform-fake`, PlaytestDesc.platform); a published page never fakes them. Entitlements are always the host's answer, never the engine's log or a save: ask og.platform.entitlements when the game starts, and grant a consumable once per purchase with og.platform.grant.

Unasked host input (a pending purchase settled, a refund, an entitlement changed on another device, an invite arrived) is a notice: og.PlatformNotice at the start of a tick, read with og.platform.notice, logged with the answers. In a running lockstep or rollback session (og.Lockstep) a platform answer or notice is this peer's alone: a system that reads og.PlatformAnswered or og.PlatformNotice, or calls og.platform.result or og.platform.notice, while it writes shared state (anything but the session's own records: input, audio, camera, og.Lockstep) hears nothing and reads null, and the engine logs it once; carry what every peer must see in og.PlayerInput (og.net.submit).

Rooms (roomCreate, roomJoin, roomSend, ...) put this engine with other players' engines: the host carries them (natively `og relay` with `og run --room`, on the web the platform's rooms through the shell, docs/platform-asks.md) and its answers and the other peers' messages arrive as events at the start of a tick (og.RoomJoined, og.RoomPeerJoined, og.RoomPeerLeft, og.RoomMessage), host input like key presses. Deterministic play over a room is og.net's lockstep.

- **TypeScript**: functions are the namespace `og.platform`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module platform {
  doc "Platform services as host capabilities (docs/design.md §2.8): cloud store, leaderboards, telemetry, Origin Point and AI marks. Game code names a method and its arguments, never a credential, a URL or a cookie: the host owns them. On the web the shell forwards each call to the Player through origin-checked postMessage (Player protocol 2, docs/web-export.md); natively the host answers from a local JSON store (`og run --platform-store`, default `<project>/.og/platform.json`)."
  doc "Calls are asynchronous: each returns a request number at once, and the answer arrives at the start of a later tick, when og.platform sends PlatformAnswered; read it with og.platform.result. Answers are host input, like key presses: the probe's `dump_platform` lists every call, its arguments, its answer and the ticks."
  doc "Errors carry the Player's stable codes: offline, timeout, invalid, unauthorized, not_found, conflict, too_large, rate_limited, unavailable, failed, cancelled (the player backed out of a purchase, an ad or a share sheet) and no_fill (no ad to show now); retryable is true for offline, timeout, unavailable, rate_limited and no_fill."
  doc "Purchases, ads, achievements, the player, friends, invites and share (products, purchase, showAd, achievementUnlock, player, friends, inviteSend, share, ...) are the platform's: accounts and money never enter the engine. Each call is a method name and its arguments, the host attaches the player's game-scoped credential, and every id it hands back (the player, a friend, a purchase, an invite) is opaque and scoped to this game. A host that lacks one answers it `unavailable`: ask og.platform.supports first. Natively and in playtests a deterministic fake answers them (`og run --platform-fake`, PlaytestDesc.platform); a published page never fakes them. Entitlements are always the host's answer, never the engine's log or a save: ask og.platform.entitlements when the game starts, and grant a consumable once per purchase with og.platform.grant."
  doc "Unasked host input (a pending purchase settled, a refund, an entitlement changed on another device, an invite arrived) is a notice: og.PlatformNotice at the start of a tick, read with og.platform.notice, logged with the answers. In a running lockstep or rollback session (og.Lockstep) a platform answer or notice is this peer's alone: a system that reads og.PlatformAnswered or og.PlatformNotice, or calls og.platform.result or og.platform.notice, while it writes shared state (anything but the session's own records: input, audio, camera, og.Lockstep) hears nothing and reads null, and the engine logs it once; carry what every peer must see in og.PlayerInput (og.net.submit)."
  doc "Rooms (roomCreate, roomJoin, roomSend, ...) put this engine with other players' engines: the host carries them (natively `og relay` with `og run --room`, on the web the platform's rooms through the shell, docs/platform-asks.md) and its answers and the other peers' messages arrive as events at the start of a tick (og.RoomJoined, og.RoomPeerJoined, og.RoomPeerLeft, og.RoomMessage), host input like key presses. Deterministic play over a room is og.net's lockstep."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 14829</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`PlatformOrder`](platform/type-PlatformOrder.md) | enum |  | How a leaderboard sorts; the first submission to a board fixes it. |
| [`og.PlatformAnswered`](platform/record-PlatformAnswered.md) | event |  | An answer arrived: sent by og.platform at the start of the tick it was applied, read by systems the tick after (og.platform.result has it at once). |
| [`StoreSetDesc`](platform/type-StoreSetDesc.md) | desc struct |  | A write to the store. |
| [`LeaderboardSubmitDesc`](platform/type-LeaderboardSubmitDesc.md) | desc struct |  | A score. |
| [`LeaderboardQueryDesc`](platform/type-LeaderboardQueryDesc.md) | desc struct |  | Which entries. |
| [`PointDesc`](platform/type-PointDesc.md) | desc struct |  | Origin Point for something the player did. |
| [`AiMarkDesc`](platform/type-AiMarkDesc.md) | desc struct |  | Marks content the game shows as generated by AI, for the platform's disclosure. |
| [`SaveSlotDesc`](platform/type-SaveSlotDesc.md) | desc struct |  | A save slot: a long text (a world.snapshot()) kept across as many store keys as it needs. |
| [`LoadSlotDesc`](platform/type-LoadSlotDesc.md) | desc struct |  | Which slot to read, and how much of it. |
| [`PlatformError`](platform/type-PlatformError.md) | struct |  | Why a call failed. |
| [`PlatformResult`](platform/type-PlatformResult.md) | struct |  | The answer to one call. |
| [`PlatformInfo`](platform/type-PlatformInfo.md) | struct |  | What answers platform calls here. |
| [`og.platform.storeGet`](platform/fn-storeGet.md) | function | `(world: World, key: string): Result<number>` | Read one key of the player's store for this game. |
| [`og.platform.storeSet`](platform/fn-storeSet.md) | function | `(world: World, desc?: StoreSetDesc): Result<number>` | Write one key of the player's store for this game; with a revision, only if nobody wrote it since (another device, another tab). |
| [`og.platform.storeList`](platform/fn-storeList.md) | function | `(world: World): number` | The keys of the player's store and their revisions. |
| [`og.platform.leaderboardSubmit`](platform/fn-leaderboardSubmit.md) | function | `(world: World, desc?: LeaderboardSubmitDesc): Result<number>` | Submit a score; the board keeps the player's best. |
| [`og.platform.leaderboardTop`](platform/fn-leaderboardTop.md) | function | `(world: World, desc?: LeaderboardQueryDesc): number` | The best entries of a board, and the player's own. |
| [`og.platform.leaderboardAround`](platform/fn-leaderboardAround.md) | function | `(world: World, desc?: LeaderboardQueryDesc): number` | The entries around the player's rank. |
| [`og.platform.telemetry`](platform/fn-telemetry.md) | function | `(world: World, name: string, attrs?: unknown): Result<void>` | Record a game event for the game's analytics. |
| [`og.platform.point`](platform/fn-point.md) | function | `(world: World, desc?: PointDesc): Result<number>` | Ask for Origin Point for something the player did. |
| [`og.platform.aiMark`](platform/fn-aiMark.md) | function | `(world: World, desc?: AiMarkDesc): Result<number>` | Disclose AI-generated content the game shows. |
| [`og.platform.loadSettings`](platform/fn-loadSettings.md) | function | `(world: World): number` | Ask for the game's saved settings (volume, controls, language): store.get of the key `og.settings`. |
| [`og.platform.saveSettings`](platform/fn-saveSettings.md) | function | `(world: World, value: unknown): Result<number>` | Save the game's settings under `og.settings` (the last save wins), with the same local fallback as loadSettings. |
| [`og.platform.saveSlot`](platform/fn-saveSlot.md) | function | `(world: World, desc?: SaveSlotDesc): Result<number>` | Save a long text under a slot, in as many store keys as it needs: a mid-size 3D world's snapshot is over a key's 64 KiB (93 entities with skinned characters, cloths and minds: about 78 KB). |
| [`og.platform.loadSlot`](platform/fn-loadSlot.md) | function | `(world: World, desc?: LoadSlotDesc): Result<number>` | Read a slot saveSlot wrote: its manifest, then its parts, joined and checked against the manifest's length and checksum. |
| [`AdKind`](platform/type-AdKind.md) | enum |  | Which kind of ad. |
| [`PlatformNoticeKind`](platform/type-PlatformNoticeKind.md) | enum |  | What a notice is about. |
| [`og.PlatformNotice`](platform/record-PlatformNotice.md) | event |  | Unasked host input arrived: sent by og.platform at the start of the tick it was applied, read by systems the tick after; read it with og.platform.notice. |
| [`og.PlatformGrants`](platform/record-PlatformGrants.md) | resource |  | The purchases this world granted (og.platform.grant), so a consumable is granted once however its answer or notice comes back: a replay, a relaunch, a restored save. |
| [`PlatformNoticeData`](platform/type-PlatformNoticeData.md) | struct |  | A notice og.PlatformNotice announced. |
| [`ProductsDesc`](platform/type-ProductsDesc.md) | desc struct |  | Which products to describe. |
| [`PurchaseDesc`](platform/type-PurchaseDesc.md) | desc struct |  | A purchase to ask the player for. |
| [`AdDesc`](platform/type-AdDesc.md) | desc struct |  | An ad to show. |
| [`AchievementProgressDesc`](platform/type-AchievementProgressDesc.md) | desc struct |  | Progress towards an achievement. |
| [`FriendsDesc`](platform/type-FriendsDesc.md) | desc struct |  | How many friends. |
| [`InviteDesc`](platform/type-InviteDesc.md) | desc struct |  | An invite to this game. |
| [`ShareDesc`](platform/type-ShareDesc.md) | desc struct |  | What to share through the device's share sheet. |
| [`og.platform.products`](platform/fn-products.md) | function | `(world: World, desc?: ProductsDesc): Result<number>` | Describe the products the game sells: titles and prices as the player's store shows them. |
| [`og.platform.purchase`](platform/fn-purchase.md) | function | `(world: World, desc?: PurchaseDesc): Result<number>` | Ask the player to buy a product: the host shows its own purchase sheet and takes the money. |
| [`og.platform.entitlements`](platform/fn-entitlements.md) | function | `(world: World): number` | What the player owns in this game, as the host knows it: durables, active subscriptions and consumables not yet consumed. |
| [`og.platform.consume`](platform/fn-consume.md) | function | `(world: World, purchase: string): Result<number>` | Tell the host a consumable purchase was delivered: it leaves the entitlements. |
| [`og.platform.grant`](platform/fn-grant.md) | function | `(world: World, purchase: string): Result<boolean>` | Record that this world granted purchase `purchase` (in og.PlatformGrants): true the first time, false for one already granted, so a purchase's answer, notice or entitlement seen again (a replay, a re… |
| [`og.platform.granted`](platform/fn-granted.md) | function | `(world: World, purchase: string): boolean` | Whether this world granted purchase `purchase` (og.platform.grant). |
| [`og.platform.adReady`](platform/fn-adReady.md) | function | `(world: World, desc?: AdDesc): Result<number>` | Ask whether an ad can be shown now, to offer a rewarded one only when there is one. |
| [`og.platform.showAd`](platform/fn-showAd.md) | function | `(world: World, desc?: AdDesc): Result<number>` | Show an ad. While it shows, the host pauses the game through its lifecycle (no ticks, no audio: on the web the Player's pause and resume) and the answer arrives after it resumed, at a tick's start. |
| [`og.platform.achievementUnlock`](platform/fn-achievementUnlock.md) | function | `(world: World, id: string): Result<number>` | Unlock an achievement. |
| [`og.platform.achievementProgress`](platform/fn-achievementProgress.md) | function | `(world: World, desc?: AchievementProgressDesc): Result<number>` | Report progress towards an achievement; 100 unlocks it. |
| [`og.platform.achievements`](platform/fn-achievements.md) | function | `(world: World): number` | The player's achievements in this game. |
| [`og.platform.player`](platform/fn-player.md) | function | `(world: World): number` | Who is playing: an id scoped to this game (the same player has another id in another game) and the name to show. |
| [`og.platform.friends`](platform/fn-friends.md) | function | `(world: World, desc?: FriendsDesc): number` | The player's friends, those playing this game first. |
| [`og.platform.inviteSend`](platform/fn-inviteSend.md) | function | `(world: World, desc?: InviteDesc): Result<number>` | Invite a friend to this game, or offer an invite link through the share sheet. |
| [`og.platform.invites`](platform/fn-invites.md) | function | `(world: World): number` | The invites to this game the player has not acted on, the one the game was opened with marked `opened`. |
| [`og.platform.share`](platform/fn-share.md) | function | `(world: World, desc?: ShareDesc): Result<number>` | Share through the device's share sheet: the text, the game's own Play link and, with image, the frame drawn last. |
| [`og.platform.supports`](platform/fn-supports.md) | function | `(world: World, method: string): boolean` | Whether the host answers platform method `method` (`purchase.buy`, `ad.show`, `share`: og.platform.info's capabilities): what to offer the player before asking. |
| [`og.platform.notice`](platform/fn-notice.md) | function | `(world: World, notice: number): PlatformNoticeData \| null` | A notice og.PlatformNotice announced. |
| [`og.platform.result`](platform/fn-result.md) | function | `(world: World, request: number): PlatformResult \| null` | The answer to a call, once it arrived. |
| [`og.platform.info`](platform/fn-info.md) | function | `(world: World): PlatformInfo` | What answers platform calls here, and what it answers. |
| [`og.platform.now`](platform/fn-now.md) | function | `(world: World): number` | The host's wall-clock time, UTC milliseconds since 1970, as of this tick: what the host last told the engine (when it attached, and with every answer, applied at a tick's start like the answers) move… |
| [`RoomDesc`](platform/type-RoomDesc.md) | desc struct |  | A room to create. |
| [`QuickPlayDesc`](platform/type-QuickPlayDesc.md) | desc struct |  | Quick play: what this engine waits to be matched for (og.platform.quickPlay). |
| [`RoomPeer`](platform/type-RoomPeer.md) | struct |  | One peer in the room. |
| [`RoomInfo`](platform/type-RoomInfo.md) | struct |  | The room this engine is in. |
| [`RoomMessageData`](platform/type-RoomMessageData.md) | struct |  | A message another peer sent: what og.RoomMessage announced. |
| [`og.RoomJoined`](platform/record-RoomJoined.md) | event |  | roomCreate, roomJoin or quickPlay finished, or the host put this engine in a room (`og run --room`, a Play link to a room): sent at the start of the tick after the host answered. |
| [`og.RoomPeerJoined`](platform/record-RoomPeerJoined.md) | event |  | Another peer joined this engine's room. |
| [`og.RoomPeerLeft`](platform/record-RoomPeerLeft.md) | event |  | A peer left this engine's room (it left, closed, or lost its connection), after every message it sent; or, with peer equal to this engine's own number, this engine is in no room any more (roomLeave, … |
| [`og.RoomMessage`](platform/record-RoomMessage.md) | event |  | Another peer's message arrived; read it with og.platform.roomMessage. |
| [`og.platform.roomCreate`](platform/fn-roomCreate.md) | function | `(world: World, desc?: RoomDesc): Result<void>` | Create a room and join it. |
| [`og.platform.roomJoin`](platform/fn-roomJoin.md) | function | `(world: World, room: string): Result<void>` | Join a room by its id. |
| [`og.platform.quickPlay`](platform/fn-quickPlay.md) | function | `(world: World, desc?: QuickPlayDesc): Result<void>` | Quick play: wait to be matched with other players asking for the same mode and number of players, then play in a room of their own. |
| [`og.platform.roomLeave`](platform/fn-roomLeave.md) | function | `(world: World): Result<void>` | Leave the room: og.RoomPeerLeft with this engine's own peer number answers, and the other peers hear it leave after its last message. |
| [`og.platform.roomSend`](platform/fn-roomSend.md) | function | `(world: World, data: Uint8Array \| unknown, to?: number): Result<void>` | Send a message to every other peer in the room, or to one: a JSON value, or bytes (a Uint8Array). |
| [`og.platform.roomPeers`](platform/fn-roomPeers.md) | function | `(world: World): readonly RoomPeer[]` | The peers in this engine's room, this one included, by number; empty in none. |
| [`og.platform.roomInfo`](platform/fn-roomInfo.md) | function | `(world: World): RoomInfo` | The room this engine is in, and what carries rooms here. |
| [`og.platform.roomMessage`](platform/fn-roomMessage.md) | function | `(world: World, message: number): RoomMessageData \| null` | A message og.RoomMessage announced: who sent it and what it carries. |
