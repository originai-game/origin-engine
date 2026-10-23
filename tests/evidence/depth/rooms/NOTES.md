# Depth S4: rooms and lockstep (branch `depth/rooms`)

Stream S4 of docs/depth.md §4, rebased onto main 24116264 (wave 2). The
contract is docs/design.md §2.8 (rooms) and §2.10 (Lockstep).

## Commits

| Commit | What |
|---|---|
| eb31aaa8 | `crates/og-net`: the room protocol, lockstep (seats, the tick gate's answer, start by the authority, late join by snapshot, hash batches, first divergent tick, snapshot exchange, the dump), the shared hash and the component-level diff, the relay and its client |
| bbce63bc | The IDL: og.platform's room members and events, module `net` (og.Lockstep, og.PlayerInput, og.Desync, submit/status/hash), the probe's `desync` dump kind, the host ABI `og_room_take`/`og_room_deliver`/`og_room_set_host`, and the regenerated outputs. Its message says "WIP (stream deferred)": on its own it does not build, because the hosts' ABI impls arrive in the next commit. The branch builds from 477075f1 on. |
| 477075f1 | og-runtime's net.rs (rooms, lockstep, the og.net system, dump_desync); the probe's tick gate; the host shims (node, web); `og relay`; `og run --room` |
| a46c5cb4, dcdbe3cb | The arena archetype (Coin Duel), `PlaytestDesc.room`, `packages/engine/tests/rooms.test.ts`, the desync fixture; the relay client now closes its socket before the process exits |
| 24ba3c71 | The web: the Worker's `room`/`room-events`/`room-host` messages, the shell's `RoomHost`, `playerRooms` over the Player's `room.*` |
| b30a7ee7 | docs/platform-asks.md #16 (rooms in Player protocol 2, with the message shapes) |
| e758ef6e, 458dfe0a | design §2.8 and §2.10; og-game §20 and an og-platform-sdk row; the archetype table and create's list; a clippy fix |

## Checks run (no gate, per AGENTS.md)

- `cargo test -p og-net --features ws` (6 tests), `-p og-probe` (4),
  `-p og-runtime --lib` (12), `-p og-idl` (1): all pass.
- `cargo clippy -p og-net --features ws -p og-runtime -p og-probe -p og-cli
  -p og-host-node --all-targets -D warnings` and `cargo fmt --check`:
  clean.
- `og api gen`, `og api gen --wiki` and `og api check` after the IDL
  change. Check: "the generated files match crates/og-api/api.ogt".
- `packages/engine`: `bun run typecheck` passes;
  `bun test tests/rooms.test.ts tests/platform-player.test.ts`: 6 pass.
- `packages/knowledge`: `bun run test` 42 pass; `bun scripts/scaffold.ts
  --write` (no change: it reads only `templates/game`).
- `packages/create`: `bun test` 14 pass.
- `og test templates/archetypes/arena` (its one playtest, `duel`): PASS, 11
  checks.

## The hashes

The two-runs-through-the-relay test: `og relay --port 0` and two `og run
templates/archetypes/arena --headless --no-render --room
ws://127.0.0.1:P/duel --ticks 600`, one with `--setup bot:1` and one with
`--setup bot:2`. Each process drives its own seat with a different
scripted input.

| Run | Peer 0 netHash @600 | Peer 1 netHash @600 | Start tick | Desync |
|---|---|---|---|---|
| First manual run | 6c19159801c383fb | 6c19159801c383fb | 1 | none |
| `og test` arena `duel` (rendered, with a `--commands` script) | 58a5ed14c33272ef | 58a5ed14c33272ef | 1 | none |
| Screenshot run, 400 ticks | fd45ac65948f33ae | fd45ac65948f33ae | 1 | none |

`netHash` is og.net's shared hash: the world without the peer's own
session. The full `hash` differs between the peers, as expected:
80252056ca22d73e against 0503fd47c61a5d91 in the first run, because each
peer's og.Input, og.Lockstep.localPlayer and room events are its own.
Each process ran with about 1,300 gate waits at an input delay of 2.

`arena-duel-peer1.png` is peer 1's frame at tick 360 in lockstep. It shows
Red 2 and Blue 3, both fighters, the coin, and "You are Blue": I read it.
Nothing compares it against a reference; it is not a gfx test.

## The desync dump

`packages/engine/tests/fixtures/desync`: two peers in lockstep, one system
that writes `Math.random()` into `Wobble.offset` at tick 100.

- Both processes log `WARN tick 110: lockstep: DESYNC at tick 101: this
  world hashes 5bfe9108ab2f8a48, peer 0's 74d14b3e9761f0f7; dump_desync
  names what differs`. The mismatch shows at the first hash batch after
  the divergence, tick 110.
- The game's og.Desync reader saw `{ tick: 101, peer: <the other> }`.
  Tick 101 is og.Time.tick of the world after the tick-100 system ran.
- `dump_desync` from peer 1:

```json
{
  "desync": true,
  "tick": 101,
  "local":  { "hash": "5bfe9108ab2f8a48", "peer": 1, "player": 1 },
  "remote": { "hash": "74d14b3e9761f0f7", "peer": 0, "player": 0 },
  "diff": {
    "differs": true,
    "components": [
      { "entity": "wobbler", "component": "Wobble",
        "fields": [ { "field": "offset", "local": 0.8222088813781738, "remote": 0.8578776717185974 } ] }
    ],
    "resources": [], "events": [], "blocks": [],
    "entities": { "onlyLocal": [], "onlyRemote": [] },
    "allocator": false, "truncated": 0
  }
}
```

Peer 0's dump is the mirror image. `rooms.test.ts` asserts the tick, the
og.Desync event, and exactly one component diff, `Wobble.offset` on
`wobbler`.

## Shared files touched

- `crates/og-api/api.ogt`: og.platform's room members and events, appended
  module `net`, and DumpKind `desync` with DesyncDump. They coexist with
  video, debug, path and timeline.
- `crates/og-idl/src/gen/host.rs`: the room ABI functions, plus the
  generated outputs (`packages/engine/types`, `schema`, `src/generated`,
  wiki `net.md` and `platform.md`).
- `crates/og-probe` (lib.rs, probe.rs, rpc.rs): the tick gate (`set_gate`,
  `held()` in step, frame and advance), the after-tick hook, DumpKind
  Desync through the registered `dump_desync`, and rpc's `desync`.
- og-runtime's tick: `engine.rs` is unchanged. Lockstep's hold and
  per-tick hash go through the probe's gate and after-tick hook, set in
  `probe.rs`. `e6.rs` mounts, registers and retires net. `save.rs`'s
  `restore_session` keeps og.Lockstep. `lib.rs` exports the room ABI.
  `net.rs` is new.
- `crates/og-cli`: `main.rs` (the `relay` dispatch beside `render`, and
  the usage), `run.rs` (`--room`), `serve_relay.rs` (new; `relay.rs` was
  already the signal forwarder).
- Host shims: `og-host-node` and `og-host-web` (`host.rs`, `host_e6.rs`);
  og-host-node's `lib.rs` and the new `room.rs` (connect, pump, close,
  `og_node_relay_start`).
- `Cargo.toml` (member og-net), `Cargo.lock`.
- `packages/engine`:
  - `src/e6.ts` and `src/index.ts` (`og.net`).
  - `src/node/index.ts`.
  - `src/node/run.ts`: `--room`, lobby pacing, and `netHash` and
    `lockstep` in `--json`.
  - `src/playtest.ts` and `src/node/playtest-run.ts` (`PlaytestDesc.room`).
  - `src/web/worker.ts` and `src/web/shell.ts`.
  - `shell/player.ts`, `shell/platform.ts` and `shell/page.ts`.

## Metal references owed

None. The stream adds no gfx test and no reference frame. The arena has
no `references/` and no frame playtest.

## Left out

- **The platform's rooms.** The web path is written and unit-tested
  against a stand-in Player. No Player answers `room.*` until ask #16
  lands, so on Play roomInfo's host is `none` today. No browser run of two
  pages was made (web-reference and wasm builds are outside this stream's
  loop).
- **A Play link to a room** (`?room=<id>`): named in ask #16, not read by
  the shell yet.
- **Rollback.** Prediction and rollback are not part of §4. Lockstep
  waits for the inputs, and `inputDelay` hides the latency.
- **No `then` or `runs` beside `room`.** A room playtest refuses both, and
  its processes stop at `room.ticks`: a `quit` step would land on
  different ticks, since a joiner moves to the authority's tick.
- The arena's `og polish` report and a recorded clip were not made.
