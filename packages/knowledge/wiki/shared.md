<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `shared` module

Shared worlds: visitors of one large world see each other's avatars in real time, near them only. Each client runs its own world; a world server (`og world-server`) holds the avatars, checks each move and sends each client those near it. The game marks its avatar with og.SharedAvatar and moves it as always; every other visitor in sight is an og.RemoteAvatar entity the engine spawns, moves and despawns.

The host supplies the connection; game code names no URL or token. What the session delivers is host input, journaled with og.platform.journal.

See `shared-worlds`.

- **TypeScript**: functions are the namespace `og.shared`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `og.SharedAvatar`, `og.RemoteAvatar`, `og.shared.join`, `og.shared.status`; 18 more are advanced (`og api show shared --all`).
- **Index**: [every module and guide](index.md).

```ogt
module shared {
  doc "Shared worlds: visitors of one large world see each other's avatars in real time, near them only. Each client runs its own world; a world server (`og world-server`) holds the avatars, checks each move and sends each client those near it. The game marks its avatar with og.SharedAvatar and moves it as always; every other visitor in sight is an og.RemoteAvatar entity the engine spawns, moves and despawns."
  doc "The host supplies the connection; game code names no URL or token. What the session delivers is host input, journaled with og.platform.journal."
  # declarations follow, each on its own page
```

<sub>Module `shared` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`SharedState`](shared/type-SharedState.md) | enum |  | Where this client's session stands (og.SharedWorld.state). |
| [`SharedState()`](#column-SharedState) | column kind |  | A SharedState field: the variant's index. |
| [`og.SharedWorld`](shared/record-SharedWorld.md) | resource |  | This client's shared-world session: its settings, which the game sets, and where it stands, which the engine writes. |
| [`og.SharedAvatar`](shared/record-SharedAvatar.md) | component |  | Marks the one entity whose state this client shares: its og.Transform's position and its yaw (the rotation about y), the velocity its moves imply, and the fields of the game's avatar component (og.sh… |
| [`og.RemoteAvatar`](shared/record-RemoteAvatar.md) | component |  | Another visitor in sight, written by the engine: it spawns the entity with an og.Transform (and the game's avatar component, og.shared.shape) when the visitor comes into sight, moves it every tick, a… |
| [`og.SharedJoined`](shared/record-SharedJoined.md) | event |  | The server took this client in: sent at the start of the tick after its welcome arrived, and again after a reconnect. |
| [`og.SharedLeft`](shared/record-SharedLeft.md) | event |  | The session ended: og.shared.leave, a refusal, or reconnecting gave up. |
| [`og.PresenceEntered`](shared/record-PresenceEntered.md) | event |  | A visitor came into sight: its og.RemoteAvatar entity exists from this tick. |
| [`og.PresenceLeft`](shared/record-PresenceLeft.md) | event |  | A visitor left sight (or the world): its og.RemoteAvatar entity was despawned this tick. |
| [`og.SharedCorrected`](shared/record-SharedCorrected.md) | event |  | The server refused a move of this avatar (faster than its kind allows, or outside the world's bounds) and the engine put it back: at the state the server holds, plus what it moved here since. |
| [`SharedShapeDesc`](shared/type-SharedShapeDesc.md) | desc struct |  | What the game shares besides the transform (og.shared.shape). |
| [`SharedJoinDesc`](shared/type-SharedJoinDesc.md) | desc struct |  | Which world to join (og.shared.join). |
| [`SharedRemote`](shared/type-SharedRemote.md) | struct |  | Who an og.RemoteAvatar is. |
| [`SharedStatus`](shared/type-SharedStatus.md) | struct |  | The session, as a HUD or the probe reads it. |
| [`og.shared.shape`](shared/fn-shape.md) | function | `(world: World, desc?: SharedShapeDesc): Result<void>` | Set what the game shares besides the transform: its avatar component and its event component. |
| [`og.shared.join`](shared/fn-join.md) | function | `(world: World, desc?: SharedJoinDesc): Result<void>` | Join a shared world: the host asks its session provider for a connection (an endpoint and a token the engine never reads), the engine says hello, and og.SharedJoined arrives once the server takes thi… |
| [`og.shared.leave`](shared/fn-leave.md) | function | `(world: World): Result<void>` | Leave the shared world: the server is told, and every og.RemoteAvatar despawns at the next tick (og.SharedLeft says `left`). |
| [`og.shared.teleport`](shared/fn-teleport.md) | function | `(world: World, position: Vector3): Result<void>` | Move this avatar anywhere at once (a map click, a portal): its og.Transform is set and the next proposal is a teleport, which the server takes past the speed rule when its policy allows, and remote c… |
| [`og.shared.send`](shared/fn-send.md) | function | `<C extends AnyComponent>(world: World, component: C, value: Partial<Value<C>>): Result<void>` | Send an event to the visitors near this one (those whose sight holds this avatar): reliable, at most 1 KiB, rate-limited by the server. |
| [`og.shared.events`](shared/fn-events.md) | function | `<C extends AnyComponent>(world: World, component: C): Result<readonly Value<C>[]>` | The events delivered this tick, in the order they arrived, each with `from` set to its sender's og.RemoteAvatar when the component has that field. |
| [`og.shared.remote`](shared/fn-remote.md) | function | `(world: World, entity: Entity): SharedRemote \| null` | Who an og.RemoteAvatar is: its player id, name and kind. |
| [`og.shared.status`](shared/fn-status.md) | function | `(world: World): SharedStatus` | The session now: its state, what carries it, the avatars held, the server's tick, and the bytes and corrections so far. |

## Column kinds

Each is a factory, `SharedState(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-SharedState"></a>`SharedState` | `SharedState` | `Uint32Array` | 1 | A SharedState field: the variant's index. |

```ogt
column SharedState {
  doc "A SharedState field: the variant's index."
  array Uint32Array
  stride 1
  value SharedState
}
```

<sub>Module `shared` line 16</sub>
