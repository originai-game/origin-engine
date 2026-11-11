<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `saves` guide: Saves and the platform

How a game keeps the player's progress and talks to the platform: the save
flow from world.snapshot through a save slot and back, whole saves and
selections, what a save holds and what the session keeps, save versions and
migrations, sizes and timings, and og.platform's calls (store, slots,
leaderboards, purchases, identity, rooms), including a host that lacks a
service. Read before writing a save system or a call to og.platform.

The API: `og api show world`, `og api show platform`, `og api show game`
(GameDesc.saveVersion, GameDesc.migrations), `og api show reflect`.

<a id="guide-saves-1"></a>
### `saves §1` The save flow

A save is text: world.snapshot writes it, og.platform.saveSlot keeps it,
og.platform.loadSlot reads it back and world.restore applies it.

```ts
// Save (a command, a UI callback, or a system's last write of the tick).
const text = world.snapshot();
if (text.ok) og.platform.saveSlot(world, { slot: "slot1", text: text.value, meta: { name, chimes } });

// Title screen: the Continue label from the manifest alone.
const peek = og.platform.loadSlot(world, { slot: "slot1", metaOnly: true });

// Load: ask for the whole slot, restore once the answer arrives.
const req = og.platform.loadSlot(world, { slot: "slot1" });
// ... a later tick, on og.PlatformAnswered for req.value:
const answer = og.platform.result(world, req.value);
if (answer?.ok && answer.data.text) world.restore(answer.data.text);
```

- world.snapshot writes `og-save:3:` and then the world deflated and
  base64-encoded. Every build has it, publish included, and the same world
  saves to the same text on the native and web hosts.
- og.platform.saveSlot spreads the text over as many store keys as it
  needs (a key holds 64 KiB) and writes the slot's manifest last. A
  save cut short leaves the previous one whole; another device's save of
  the same slot landing meanwhile is a `conflict`, and the slot then holds
  that save, never a mix of the two.
- `meta` is what a title screen shows (`{ place, hour }`); it lives in the
  manifest, so `metaOnly` reads it without the parts.
- The restore applies when the tick ends if it is called in a system or a
  command, and at once in GameDesc.start or between ticks. The tick after
  it delivers og.WorldRestored (§4).
- Small settings (volume, controls, language) do not need a slot:
  og.platform.saveSettings and loadSettings keep one object, with a local
  fallback where the host has no store or is offline.

<a id="guide-saves-2"></a>
### `saves §2` Whole world or a selection

`world.snapshot()` (or `{}`) saves the whole world. Naming tags in
SnapshotDesc's `with`, or resources in `resources`, saves a selection.

| Choose | When |
|---|---|
| the whole world | quick save, checkpoints, a replay or a tool returning to an exact state, any game whose level is built at run time |
| a selection | progress and settings over a level the game rebuilds from its scene; a world too large for one slot (§6) |

- A selection keeps the game's progress and settings, and the game
  rebuilds the level from its scene. Its type table lists only the types it
  uses, so it is small and takes fewer store keys.
- Restoring a selection replaces the live entities that carry its tags by
  the saved ones, spawned anew in saved order, with each saved Entity field
  pointed at its entity's new id, and sets its resources. The rest of the
  world (other entities, og.Time, the modules' state, the session) stays as
  it is.
- A selection keeps the host's input whatever RestoreDesc.session says.

<a id="guide-saves-3"></a>
### `saves §3` What a save holds

A save holds the world and nothing else: every entity with its components
and names, every resource (og.Time's clock and the og.Rng streams too),
the pending events, the modules' state (physics, animation, flipbooks,
tweens, tilemap and terrain edits), each type with its schema hash, and
the game's save version. The restored world hashes as the saved one did,
on either host.

Keeping state savable:

- Module variables, closures and `Map`s outside the world are not world
  state: a snapshot, a replay and a hot reload lose them. Keep progress in
  components and resources.
- A list field holds at most a fixed number of one number type or handle
  (`list(u32(), 8)`). A few vectors can be numbered fields (`p0` to `p7`,
  as og.CameraRail does). A list of things with fields of their own (a ring
  of skid marks, the laps of a race) is an entity each, with its own
  component, looped over by a query. A long record (a lap's taped inputs) is
  text in `str` fields of at most 4,096 bytes each, chunked over entities.
- A handle (an og.load answer, a texture, a sound, a geometry, a material,
  an entity) is a 64-bit number. Keep it in a field of its own kind
  (`Asset()`, `Texture()`, `Sound()`, `Font()`, `Style()`, `Geometry()`,
  `Material()`, `Entity()`: every handle type has its factory, named as the
  type), never cast to another kind. A `u32` field refuses one (handles pass
  2^32: `4294967298 is not a u32`); an `f64` holds the number but loses its
  type.
- Asset and material handles are saved as their numbers. Load what a saved
  entity uses in the same order on every launch, in GameDesc.start, before
  restoring.
- Physics saves mesh, convex-hull and height-field colliders by their
  geometry's handle and scale, not their triangles (og.physics.setMesh).
  The meshes and height fields a save's colliders name must be loaded or
  registered before world.restore; a save naming one that is not is
  refused.

Saving inside a system: the snapshot is the world in the middle of that
tick, as that system sees it. What earlier systems wrote this tick is in
it; a structural change the system queued (applied when it returns) is
not; and it names the system. A world restored from it completes that
tick: its next step runs the systems after the saver and the engine's
(physics included) with the tick's own events, then ends the tick, and the
world goes on tick for tick as the saving one did, so a replay from it
matches. og.Time reads the saved tick. Make the snapshot the system's last
write of the tick: what the system does after the call is not in the
save. Between ticks (GameDesc.start, a command, a UI callback) the
snapshot is the world between two ticks.

<a id="guide-saves-4"></a>
### `saves §4` What the session keeps

world.restore replaces the world, not the session. These stay as they
were before the restore:

- the input: og.Input, og.Pointers, og.Gamepad, og.PointerLock (unless
  RestoreDesc.session takes the saved input);
- the player's bus volumes (og.Audio), action bindings and dead zones
  (og.input), and the UI's focus and hover;
- what lives outside the world: the platform's calls and clock, the
  language, the loaded assets, and the command channel's scheduled steps.
  Those keep their remaining delays though og.Time takes the saved tick, so
  an `og run --commands` script goes on to its last step or `quit` whether
  the save is of a later tick or an earlier one.

RestoreDesc.session off (the default): a loaded game reads the keys the
player holds now. On: the world is the one saved, hash and all, which a
tool returning to a state needs (`og explore`, a replay from a snapshot).

og.WorldRestored is delivered as the tick after the restore starts (after
a mid-tick save, the first whole tick, whose next step completes the saved
tick), not when the restore applies, so the world between the two hashes
as the save did. Everything in the world came back as saved, request
numbers and resources the session filled included; the session did not.
A system that reads og.WorldRestored resets what belongs to the session:
it forgets requests answered long ago, asks the store again for the
settings and progress, asks og.platform.entitlements again, and closes a
menu the player had open.

<a id="guide-saves-5"></a>
### `saves §5` Save versions and migrations

Every save carries GameDesc.saveVersion (else project.ogt's `saveVersion`,
else 1). Raise it when a release changes a saved type, and add the step
from the old version to GameDesc.migrations.

world.restore runs, in order: the engine's own migrations (an engine
upgrade's changes to its components and its modules' state, such as og.ui's
state of engine 0.7.7), then each game step from the save's version to
saveVersion, on the save (SaveEdit). A step that refuses or throws refuses
the restore with its reason and leaves the world as it was. A save of a
newer saveVersion is refused. The envelope must be `og-save:3`; earlier
envelope formats are refused with both versions named, never converted.
world.openSave opens a save with only the engine's migrations applied, to
read a save without restoring it or to migrate one by hand.

```ts
og.game({
  saveVersion: 3,
  migrations: [
    { from: 1, migrate: (s) => s.renameField("Wallet", "gold", "coins") },
    { from: 2, migrate: (s) => s.split("Stats", (v) => ({ Health: { hp: v.hp }, Mana: { mp: v.mp } })) },
  ],
});
```

| Change between releases | Step |
|---|---|
| a field renamed | `renameField` (`gold` saved, `coins` declared now: every balance arrives) |
| a type renamed | `renameType`; a later restore maps it field by field against the type of the new name |
| values reshaped | `transform`, with the old record in hand; `into` renames as it reshapes |
| a type split in two | `split` |
| a type gone | `dropType` |
| a component every saved entity now needs | `setRecords` on a type the save lacks |
| a new entity | `spawn` |
| fields added or removed, nothing to compute | `world.restore(text, { migrate: true })`, no step |

Without a step and without `migrate`, a save that holds data of a type
whose layout changed (a field added, removed or retyped, an enum's
variants) is refused; the reason names the type and its fields and the
fixes: `{ migrate: true }`, a migration in GameDesc.migrations, or keeping
a saved type's fields stable between releases. Two cases restore anyway:

- a type the save holds no data of (no component, resource value or
  event) takes this build's layout;
- an engine og.* type that only gained fields takes their declared
  defaults.

`{ migrate: true }` maps a changed type field by field, by name, as a hot
reload does: a field added since takes its default, a removed one is
dropped, an enum value is mapped by its variant's name, a field of another
type starts at its default, and a type this build no longer registers is
dropped. The host input the restore keeps (og.Input, og.Pointers,
og.Gamepad, og.PointerLock, og.Audio) is mapped either way: a save from
before og.Gamepad's eight pads restores without the flag.

Hashes after a migration: every peer and every restore of the same save
migrates to the same world and hash. renameField, renameType, transform,
dropType and spawn keep which types sit together, so they migrate to the
hash of a world built at the new version exactly. A split's world holds
the same records as a built one but its hash differs by the archetypes the
built one passed through (inserting the new types one at a time leaves an
empty archetype of the first, which the split never makes): compare a
split's result record by record, not by hash.

<a id="guide-saves-6"></a>
### `saves §6` Sizes and timings

| Measure | Value |
|---|---|
| one store key | 64 KiB of encoded JSON |
| keys per player and game | 64; 1 MiB in all |
| a slot | at most 17 keys (two banks of 8 parts and the manifest); text at most 512 KiB of UTF-8 |
| slot meta | 4 KiB encoded |
| a world of a few entities | about 11 KiB, most of it the table of every registered type |
| a 2D platformer, 189 entities on a 96 x 48 tile map | about 25 KiB |
| a 3D world, 93 entities with skinned characters, cloths and minds | about 78 KB: over one key, so a slot |
| 5,000 entities, half of them physics bodies | about 436 KiB; written in about 95 ms and restored in about 60 ms natively |

- A mid-size 3D world's save is over one key: use a slot, not storeSet.
- A slot writes its parts in the bank its last save did not use
  (`<slot>.a*` or `<slot>.b*`) and the manifest at the revision read before;
  that is why it takes two banks.
- A world that would take more than one slot saves a selection (§2).

<a id="guide-saves-7"></a>
### `saves §7` Platform calls

Every og.platform call is asynchronous. It returns a request number at
once; the answer arrives at the start of a later tick, when og.platform
sends og.PlatformAnswered (systems read it the tick after), and
og.platform.result has it from then on. Answers are host input, like key
presses: the probe's `dump_platform` lists every call, its arguments, its
answer and the ticks, and a replay with the same answers gives the same
world.

- Game code names a method and its arguments, never a credential, a URL or
  a cookie. The host owns them and attaches the player's game-scoped
  credential. Every id it hands back (the player, a friend, a purchase, an
  invite) is opaque and scoped to this game.
- On the web the shell forwards each call to the page that frames the game
  (PlatformInfo.host `player`) through origin-checked postMessage (protocol
  2). Natively the host answers from a local JSON store
  (`og run --platform-store`, default `<project>/.og/platform.json`).
- A failed call carries a stable code and `retryable` (`og api show
  PlatformError` lists the codes): retry a call only when `retryable` says
  it may succeed later, and treat `cancelled` as the player's choice, not
  an error to report.
- og.platform.now is the host's time as of this tick. Use it to know how
  long the game was closed (a save's time against a launch's), never to
  time play (that is og.Time). Its `clock` is `browser` while the framing
  page sends no time of its own.

The methods a host may answer are PlatformInfo.capabilities' (`og api
show PlatformInfo`).

A service the host lacks:

- It answers the call `unavailable`; nothing throws. Ask
  og.platform.supports(method) before offering the player a shop, an ad or
  a share button, and hide what it lacks.
- PlatformInfo.host `none` means nothing here answers platform calls:
  keep the game playable without them.
- Natively and in playtests a deterministic fake answers purchases, ads,
  achievements, the player, friends, invites and share
  (`og run --platform-fake`, PlaytestDesc.platform). A published page
  never fakes them.

<a id="guide-saves-8"></a>
### `saves §8` Purchases, entitlements and notices

Accounts and money never enter the engine: the host shows its own
purchase sheet and takes the money; the answer is its word on it.

1. og.platform.entitlements when the game starts and after every restore:
   what the player owns is always the host's answer, never the engine's log
   or a save.
2. og.platform.purchase; a `pending` answer is settled later by a
   `purchase` notice.
3. For a consumable, og.platform.grant (true the first time), deliver the
   goods, then og.platform.consume.

og.PlatformGrants records the last 256 grants in the world, so a purchase
seen again (a replay, a relaunch, a save restored from before consume)
grants once. It records; it is not proof of payment.

Unasked host input (a pending purchase settled, a refund, an entitlement
changed on another device, an invite arrived, a message from the framing
page) is a notice: og.PlatformNotice at the start of a tick, read with
og.platform.notice, logged with the answers. The engine keeps the last 256
notices and the last 256 room messages.

<a id="guide-saves-9"></a>
### `saves §9` Lockstep and rooms

Rooms (roomCreate, roomJoin, quickPlay, roomSend, ...) put this engine with
other players' engines. Natively `og relay` carries them (`og run --room`);
on the web the platform's rooms do, through the shell. Their answers and
the other peers' messages arrive as events at the start of a tick
(og.RoomJoined, og.RoomPeerJoined, og.RoomPeerLeft, og.RoomMessage), host
input like key presses. Deterministic play over a room is og.net's
lockstep (determinism §3).

In a running lockstep or rollback session (og.Lockstep), a platform answer,
a notice and a room message are this peer's alone. A system that reads
og.PlatformAnswered or og.PlatformNotice, or calls og.platform.result or
og.platform.notice, while it writes shared state (anything but the
session's own records: input, audio, camera, og.Lockstep) hears nothing and
reads null, and the engine logs it once. Carry what every peer must see in
og.PlayerInput (og.net.submit).

<a id="guide-saves-10"></a>
### `saves §10` Pitfalls

- Restoring before the assets a save uses are loaded: handles are numbers,
  so load in the same order on every launch, in GameDesc.start.
- Code after world.snapshot in the same system: it is not in the save.
- Progress in module variables: it is not in the save.
- Trusting a save for what the player bought: ask og.platform.entitlements.
- Granting a consumable without og.platform.grant: a replay or a restored
  save grants it twice.
- Not resetting on og.WorldRestored: an open menu, stale request numbers
  and old settings come back with the world.
- Changing a saved type's fields without raising saveVersion and adding a
  step: old saves are refused.
- Writing a large save with storeSet: anything over 64 KiB needs a slot.
- Comparing a split migration's result by hash (§5).
- Reading og.platform.result in a lockstep system that writes shared state:
  it reads null.
