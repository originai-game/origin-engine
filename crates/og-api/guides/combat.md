# Combat: attacks, combos, health and hits

How og.systems.combat plays melee attacks and takes damage: moves with startup, active and recovery ticks, combos chained by buffered presses inside a window, cancels, hit volumes struck once a swing, health, invulnerability and stagger. Read before building a melee fighter, an action game's enemies or anything that hits and gets hurt. Hit volumes themselves are in `physics`; buffered presses in `input`.

The API: `og api show combat`, `og api show play`, `og api show input`.

## 1. Attacks, moves and links

List the systems in the game and give each attacker the records:

```ts
og.game({ systems: [...og.systems.combat, ...mySystems] });
```

- An **og.AttackMove** is one move's frame data on an entity of its own:
  `startup`, `active` and `recovery` in ticks, its `damage`, which hit
  volumes strike (`volumes`, a bit mask), its cancel window and its
  `links`. Any number of attackers may play the same move entity.
- An **og.AttackLink** is a way into a move: a press of `action` while the
  move it leaves is at a tick in `[open, close)` starts `to`. A link listed
  in og.Attack.starters is taken while idle and its window is not read.
- An **og.Attack** on the attacker holds what plays (`move`, `tick`,
  `phase`, `swing`), its `starters` and the og.HitVolume entities its moves
  drive (`volumes`: bit i of a move's mask is `volumes[i]`).

A light-light-heavy combo is three moves and four links: a starter into
light 1, light 1 into light 2, light 2 into heavy, and whatever else the
game wants (a dodge into nothing is not a link: see cancels below).

**Windows and the buffer.** A press is read through og.input.consume, so a
press a few ticks before a window opens waits in the action's buffer
(ActionDesc.bufferTicks) and is taken the tick the window opens; a press
after the window closes, or one older than the buffer, does nothing. Only a
link that can be taken that tick consumes its press, so an early press stays
buffered. Typical numbers at 60 Hz: a buffer of 6 to 10 ticks, a link window
over the last half of the active window and most of recovery (8 to 16
ticks), light moves of 4 to 8 startup ticks and heavy ones of 12 to 20.
These are starting points to tune by feel, not rules.

**Cancels.** og.Attack.cancelable holds while idle and inside the playing
move's `[cancelOpen, cancelClose)`. A dodge, a jump or a block that may cut a
move checks it and writes `move` 0 (and its own state); outside the window
it waits. A window that opens after the active ticks lets recovery be
cancelled without cutting a swing short.

**Forcing a move.** Write og.Attack.next for a move chosen by code (an
enemy's AI, a scripted finisher): it starts at the next run whatever the
windows, and is cleared once taken.

**Time.** A move advances one tick each tick time moves. While the time
scale is 0 (a hit stop) nothing advances and no link is taken; the buffer
keeps the press.

**Each swing strikes once.** Every move started counts a `swing`, written to
the attacker's volumes with `owner` and `active`; og.systems.combat counts a
volume's hit on a target once a swing, so a move cannot take damage twice
from one target however long its active window is.

## 2. Health, damage and hurt

- An **og.Health** is what a target has left. A hit (og.HitEvent, read the
  tick after it is reported) takes the attacker's move's `damage` from
  `value`, never below 0, and sends og.Damaged. Reaching 0 sets `dead` and
  sends og.Died once; a dead target takes nothing more until the game
  revives it (`dead` false and a value). Nothing heals on its own.
- A hit is read the tick after it struck and takes the damage of the move
  whose swing struck it (og.Attack.swingMove), even if the game has cut
  that move since. A cut move's volumes go quiet the tick it is cut, so it
  strikes nothing more.
- An **og.Hurt** beside it says how a hit lands: for `invulnerableTicks`
  from a hit further hits are ignored (`invulnerableUntil` is the first tick
  it can be hurt again; a dodge writes it for its own i-frames), for
  `staggerTicks` its own og.Attack plays nothing (the move playing is cut),
  and og.Damaged.knockback carries `knockback` along the hit, away from the
  volume. Applying the knockback is the game's (a velocity, a character
  controller's push), as are hit stop, flashes, sounds and numbers: read
  og.Damaged and og.Died in the game's systems.

Typical numbers at 60 Hz: 20 to 40 invulnerable ticks for a player, 0 to 10
for a crowd of enemies, 10 to 20 stagger ticks for a light hit. A zero value
does nothing.

**Projectiles.** A shot needs no og.Attack: an **og.Projectile** beside its
og.HitVolume carries its `damage`, its `shooter` (og.Damaged.source and
og.Died.source, never struck by it) and its `pierce`. Leave the volume's
`owner` 0: each projectile is its own owner, so it strikes a target once,
and two projectiles strike it twice. A hit takes its damage under the
target's og.Hurt as a move's does (invulnerability, stagger, knockback);
og.Damaged.move is 0. Only a strike that takes damage counts in `struck`:
a target without og.Health (a graze ring, a wall), a dead one or one in its
i-frames lets the shot pass. Once `struck` reaches `pierce` the projectile
is spent and og.systems.combat despawns it, with its descendants, before it
strikes again. A pierce of 0 is never spent (a beam). Projectiles never
count as attackers, so a screen of them has no cap. Moving them, and
despawning them at a range, after a time or at a wall (an og.HitEvent with
`hurt` 0), are the game's.

Three shots of 1 damage into a drone with 3 health: each strikes it once
and is spent, and the third kills it.

```ts recipe
import { defineSystem, og } from "@origingame/engine";
import type { Entity, Result, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const SHIPS = 1;
const DRONES = 2;

// The game's: each shot flies up 0.25 m a tick.
const Fly = defineSystem("Fly", { reads: [og.Projectile], writes: [og.Transform] }, (q) => {
  for (const a of q.archetypes()) {
    const position = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) position[i * 3 + 1]! += 0.25;
  }
});

const Check = defineSystem("Check", { reads: [og.Health, og.Died], after: ["og.combat"] }, (_q, time, world) => {
  for (const d of world.events(og.Died)) {
    og.assert.always(d.source === world.find("ship"), "the ship's shot killed the drone", {});
    og.assert.reachable("the drone died", {});
  }
  if (time.tick !== 100) return;
  const shots = ["shot0", "shot1", "shot2"].filter((n) => world.find(n) !== null).length;
  const health = must(world.get(world.find("drone") as Entity, og.Health));
  og.assert.always(health.dead && shots === 0, "three shots, each spent at its strike, took its 3 health", { shots, health: health.value });
});

export default og.game({
  systems: [...og.systems.combat, Fly, Check],
  start(world: World) {
    const ship = must(world.spawn({ name: "ship" }));
    must(world.insert(ship, og.HurtVolume, { shape: "sphere", radius: 0.5, team: SHIPS }));
    const drone = must(world.spawn({ name: "drone", position: { x: 0, y: 5, z: 0 } }));
    must(world.insert(drone, og.HurtVolume, { shape: "sphere", radius: 0.5, team: DRONES }));
    must(world.insert(drone, og.Health, { value: 3 }));
    for (let i = 0; i < 3; i++) {
      const shot = must(world.spawn({ name: `shot${i}`, position: { x: 0, y: -i, z: 0 } }));
      must(world.insert(shot, og.HitVolume, { shape: "sphere", radius: 0.1, active: true, team: SHIPS }));
      must(world.insert(shot, og.Projectile, { damage: 1, shooter: ship, pierce: 1 }));
    }
  },
});
```

To change a rule (armour that ignores stagger, damage scaled by a stat),
copy `systems/combat.ts` from the engine package into the game and list the
copy in GameDesc.systems instead of og.systems.combat.
