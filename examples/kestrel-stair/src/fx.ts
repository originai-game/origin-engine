// What moves on its own: the crypt's lamp hangs at the chain's last link
// (og.physics.ropePoints, the og.Rope's simulated points), the braziers
// flicker, the banners' wind gusts with the time of day, and the crypt is
// dim: its og.ReverbZone (src/sound.ts; FINDINGS #21, fixed: the engine
// eases the shared reverb, which this system did by hand) says how far the
// listener is inside, and the sky's light (og.Environment's intensity)
// falls away with it.

import { og, defineSystem } from "@origingame/engine";
import { Game } from "./components.ts";
import { begin, flush } from "./state.ts";
import { BRAZIERS, CRYPT, CRYPT_ROOM } from "./layout.ts";
import { entity, must } from "./world.ts";

/** The sky's light in the crypt against the open air's. */
const SKY = { crypt: 0.12, open: 0.45 };

export const Fx = defineSystem(
  "Fx",
  { reads: [og.ReverbZone], writes: [Game, og.Transform, og.PointLight, og.Environment], after: ["View"] },
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
      // Where the courier is, for the playtests: its head in the crypt's box.
      const b = CRYPT_ROOM;
      const head = { x: g.heroX, y: g.heroY + 1.5, z: g.heroZ };
      const inside = head.x > b.min[0] && head.x < b.max[0] && head.y > b.min[1] && head.y < b.max[1] && head.z > b.min[2] && head.z < b.max[2];
      if (inside !== g.inCrypt) g.inCrypt = inside;
      // The crypt is dim as its reverb is near: the sky's light falls away with the zone's weight
      // (the listener's, written by og.audio), the lamp and the door light it.
      const weight = must(world.get(entity(world, "crypt-reverb"), og.ReverbZone)).weight;
      if (weight !== g.reverb) {
        g.reverb = weight;
        const env = must(world.resource(og.Environment));
        env.intensity = SKY.open + (SKY.crypt - SKY.open) * weight;
      }
    } finally {
      flush(g);
    }
  },
);
