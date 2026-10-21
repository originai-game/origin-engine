// What moves on its own: the crypt's lamp hangs at the chain's last link
// (og.physics.ropePoints, the og.Rope's simulated points), the braziers
// flicker, the banners' wind gusts with the time of day, and the crypt is
// a reverb zone: with the courier inside, the effects bus sends far more to
// the shared reverb and the room grows (og.AudioBusFx.reverb,
// og.Audio.reverbRoom), eased over half a second either way.

import { og, defineSystem } from "@origingame/engine";
import { Game, OneByOne } from "./components.ts";
import { BRAZIERS, CRYPT_ROOM } from "./layout.ts";
import { entity, must } from "./world.ts";

/** The crypt's reverb against the open air's. */
const ROOM = { crypt: { send: 0.55, room: 0.92, damping: 0.25 }, open: { send: 0.12, room: 0.5, damping: 0.5 } };

export const Fx = defineSystem(
  "Fx",
  { reads: [OneByOne], writes: [Game, og.Transform, og.PointLight, og.Audio, og.AudioBusFx], after: ["View"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    // The lamp at the chain's end.
    const pts = og.physics.ropePoints(world, entity(world, "crypt-chain"));
    if (pts.ok && pts.value.length > 0) {
      const end = pts.value[pts.value.length - 1]!;
      must(world.set(entity(world, "crypt-lamp"), og.Transform, { position: { x: end.x, y: end.y - 0.15, z: end.z } }));
    }
    // The braziers' light flickers.
    BRAZIERS.forEach((_, i) => {
      const f = 8 + 1.2 * Math.sin(time.elapsed * 13 + i * 1.7) + 0.8 * Math.sin(time.elapsed * 29 + i * 3.1);
      must(world.set(entity(world, `brazier-${i}`), og.PointLight, { intensity: f }));
    });
    // The reverb zone.
    const b = CRYPT_ROOM;
    const head = { x: g.heroX, y: g.heroY + 1.5, z: g.heroZ };
    const inside = head.x > b.min[0] && head.x < b.max[0] && head.y > b.min[1] && head.y < b.max[1] && head.z > b.min[2] && head.z < b.max[2];
    if (inside !== g.inCrypt) g.inCrypt = inside;
    const want = inside ? ROOM.crypt : ROOM.open;
    const k = 1 - Math.exp(-time.dt * 4);
    const send = Math.abs(want.send - g.reverb) < 0.002 ? want.send : g.reverb + (want.send - g.reverb) * k;
    if (send !== g.reverb) {
      g.reverb = send;
      const t = (send - ROOM.open.send) / (ROOM.crypt.send - ROOM.open.send);
      must(world.set(entity(world, "sfx-bus"), og.AudioBusFx, { reverb: send }));
      const audio = must(world.resource(og.Audio));
      audio.reverbRoom = ROOM.open.room + (ROOM.crypt.room - ROOM.open.room) * t;
      audio.reverbDamping = ROOM.open.damping + (ROOM.crypt.damping - ROOM.open.damping) * t;
    }
  },
);
