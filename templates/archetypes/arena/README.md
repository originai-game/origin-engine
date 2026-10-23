# Coin Duel — the arena archetype (2D, two players)

Two fighters, Red and Blue, on one floor race for one coin. Each touch
scores a point and moves the coin; the first to five wins the round. Move
with WASD or the arrows (or a pad's left stick), dash with Space or Shift.

金币对决：两名玩家在同一场地抢一枚金币，碰到得一分，金币换位置，先到五分者赢。通过房间（`og relay` 或平台房间）以锁步同步对战。

The game plays alone, with the local keys driving Red. It also plays over a
**room in lockstep** (docs/design.md §2.8, §2.10): each process runs the
whole world, and only the two players' inputs and the world's hash cross
the room.

```sh
og relay                                              # terminal 1: ws://127.0.0.1:47900/<room>
og run . --room ws://127.0.0.1:47900/duel --realtime  # terminals 2 and 3
og run . --headless --setup bot:1 --ticks 600 --json  # alone, scripted
og test .                                             # tests/duel.og.ts: two processes through a relay
```

## How it is made deterministic

- `start` sets `og.Lockstep` (`enabled`, `players: 2`, `inputDelay: 2`,
  `hashEvery: 10`). It spawns two fighters, each with an `og.PlayerInput`
  whose `player` is its seat, and the coin.
- `Submit` is the **only** system that reads `og.Input` and `og.Gamepad`.
  It hands this peer's input to `og.net.submit`.
- `Move` and `CoinDrop` read `og.PlayerInput`, `Tuning` and `og.Rng` only:
  the same on every peer.
- `Hud` draws this peer's own line ("You are Red", "Waiting for 1 more")
  and keeps nothing in the world.

A system that read `og.Input` to move a fighter, or one that used
`Math.random()`, `Date.now()` or a module variable, would part the worlds:
`og.Desync` names the first divergent tick, and `dump_desync` names the
component and field (`og run --commands "...:dump_desync:d.json"`).

## Commands

| Command | What it does |
|---|---|
| `bot:<n>` | This peer's input comes from scripted bot n (it runs toward the coin and weaves); 0 hands the input back to the keys. It is input, so each peer's bot drives only that peer's seat. |
| `warp:<player>:<x>:<y>` | Put fighter 0 (Red) or 1 (Blue) at a point of the floor (0..16 × 0..9). |

## Tunables

`Tuning`: `speed`, `dashSpeed`, `dashTicks`, `dashCooldown`, `target`,
`restTicks`. A tunable must be the same on every peer. Change it in code or
on every peer at the same tick; otherwise `og.Desync` reports it.

## What a Maker changes first

The fighters' look (they are solid sprites), the rules in `src/game.ts`,
and `players` for more seats (up to 16; seats nobody holds read zero
input). Keep every rule a function of `og.PlayerInput`, the world and
`og.Rng`.
