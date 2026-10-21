// What moves on its own: the crypt's lamp hangs at the chain's last link
// (og.physics.ropePoints, the og.Rope's simulated points), the braziers
// flicker, the banners' wind gusts with the time of day, and the crypt is
// a reverb zone: with the courier inside, the effects bus sends far more to
// the shared reverb and the room grows (og.AudioBusFx.reverb,
// og.Audio.reverbRoom), and the sky's light falls away (og.Environment's
// intensity), eased over half a second either way.

import { og, defineSystem } from "@origingame/engine";
import { Game, OneByOne } from "./components.ts";
import { begin, flush } from "./state.ts";
import { BRAZIERS, CRYPT, CRYPT_ROOM } from "./layout.ts";
import { entity, must } from "./world.ts";

/** The crypt's reverb against the open air's. */
const ROOM = { crypt: { send: 0.55, room: 0.92, damping: 0.25, sky: 0.12 }, open: { send: 0.12, room: 0.5, damping: 0.5, sky: 0.45 } };

export const Fx = defineSystem(
  "Fx",
  { reads: [OneByOne], writes: [Game, og.Transform, og.PointLight, og.Audio, og.AudioBusFx, og.Environment], after: ["View"] },
  (_q, time, world) => {
    const g = begin(world);
    try {
      // The lamp at the chain's end.
      const pts = og.physics.ropePoints(world, entity(world, "crypt-chain"));
      if (pts.ok && pts.value.length > 0) {
        const end = pts.value[pts.value.length - 1]!;
        must(world.set(entity(world, "crypt-lamp"), og.Transform, { position: { x: end.x, y: end.y - 0.15, z: end.z } }));
      }
      // What the cloths and the chain do, for the playtests: the cape's drop below its pins and how
      // far its hem trails behind the courier, how far a banner's free corner has blown from where
      // it hangs, the lamp's drop below the chain's anchor.
      if (time.tick % 6 === 0) {
        const cape = og.physics.clothPoints(world, entity(world, "hero-cape"));
        if (cape.ok && cape.value.length > 0) {
          const top = cape.value[0]!;
          const hem = cape.value[cape.value.length - 1]!;
          g.capeHang = top.y - hem.y;
          // Behind: against the courier's heading (0 faces +z).
          g.capeBehind = -((hem.x - g.heroX) * Math.sin(g.facing) + (hem.z - g.heroZ) * Math.cos(g.facing));
        }
        const banner = og.physics.clothPoints(world, entity(world, "banner-0"));
        if (banner.ok && banner.value.length > 0) {
          const corner = banner.value[banner.value.length - 1]!;
          const anchor = banner.value[7]!;
          g.bannerSway = Math.hypot(corner.x - anchor.x, corner.z - anchor.z);
        }
        if (pts.ok && pts.value.length > 0) g.lampDrop = CRYPT.chain.y - pts.value[pts.value.length - 1]!.y;
      }
      // The braziers' light flickers.
      // (Every other tick: twenty changes a second read as a flicker, at half the calls.)
      if (time.tick % 2 === 0) {
        BRAZIERS.forEach((_, i) => {
          const f = 8 + 1.2 * Math.sin(time.elapsed * 13 + i * 1.7) + 0.8 * Math.sin(time.elapsed * 29 + i * 3.1);
          must(world.set(entity(world, `brazier-${i}`), og.PointLight, { intensity: f }));
        });
      }
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
        // And the crypt is dim: the sky's light falls away inside, the lamp and the door light it.
        const env = must(world.resource(og.Environment));
        env.intensity = ROOM.open.sky + (ROOM.crypt.sky - ROOM.open.sky) * t;
      }
    } finally {
      flush(g);
    }
  },
);
