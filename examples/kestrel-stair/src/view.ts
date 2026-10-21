// The camera and what follows it. Behind the title a slow pass up the
// stair; in a run a third-person view behind the courier (Tuning's
// distance and pitch), turned by Q and E (the right stick) and swung
// slowly behind a walking courier (src/hero.ts), kept out of the walls and
// under the crypt's roof by a ray from the courier's head; at the end a
// high view from the summit back down the stair. The audio listener rides
// the camera, so the braziers, the imps and the crypt's drip are heard from
// where the player looks.

import { og, defineSystem } from "@origingame/engine";
import type { Vector3 } from "@origingame/engine";
import { Game, OneByOne, Phase, Scene, Tuning } from "./components.ts";
import { BELL, START } from "./layout.ts";
import { entity, LAYER, must } from "./world.ts";

export function spawnListener(world: import("@origingame/engine").World): void {
  const l = must(world.spawn({ name: "listener" }));
  must(world.insert(l, og.AudioListener, { volume: 1 }));
}

export const View = defineSystem(
  "View",
  { reads: [Game, Tuning, OneByOne], writes: [og.Camera, og.Transform], after: ["Pickups"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    const cam = must(world.resource(og.Camera));
    let eye: Vector3;
    let focus: Vector3;
    let snap = g.start;
    if (g.phase === Phase.Title) {
      // A slow pass looking up the stair from the court.
      const a = time.elapsed * 0.04;
      focus = { x: Math.sin(a) * 3, y: 7, z: -18 };
      eye = { x: Math.sin(a) * 20, y: 19 + Math.sin(a * 0.7) * 2, z: 50 };
    } else if (g.phase === Phase.Ending && g.scene === Scene.End) {
      // The bell and the courier, close.
      focus = { x: BELL.x, y: BELL.y + 1.6, z: BELL.z };
      eye = { x: BELL.x + 4.2, y: BELL.y + 2.6, z: BELL.z + 5.2 };
    } else if (g.phase === Phase.Over) {
      // From behind the bell, back down the whole stair.
      focus = { x: 0, y: 4, z: 0 };
      eye = { x: BELL.x - 2, y: BELL.y + 6, z: BELL.z - 5 };
    } else {
      const head = { x: g.heroX, y: g.heroY + 1.55, z: g.heroZ };
      const back = { x: Math.sin(g.yaw), z: Math.cos(g.yaw) };
      const d = t.cameraDistance;
      const want = { x: head.x + back.x * d * Math.cos(t.cameraPitch), y: head.y + d * Math.sin(t.cameraPitch), z: head.z + back.z * d * Math.cos(t.cameraPitch) };
      // Keep the eye out of the walls and under roofs: a ray from the head towards it.
      const dir = { x: want.x - head.x, y: want.y - head.y, z: want.z - head.z };
      const len = Math.hypot(dir.x, dir.y, dir.z);
      const hit = og.physics.raycast(world, { origin: head, direction: { x: dir.x / len, y: dir.y / len, z: dir.z / len }, maxDistance: len, filter: { mask: LAYER.level | LAYER.gate } });
      const reach = hit.ok && hit.value ? Math.max(0.6, hit.value.distance - 0.3) : len;
      eye = { x: head.x + (dir.x / len) * reach, y: head.y + (dir.y / len) * reach, z: head.z + (dir.z / len) * reach };
      focus = { x: head.x - back.x * 1.5, y: head.y - 0.2, z: head.z - back.z * 1.5 };
      if (hit.ok && hit.value) snap = true;
    }
    const at = cam.position;
    const far = Math.hypot(eye.x - at.x, eye.y - at.y, eye.z - at.z) > 14;
    const k = snap || far ? 1 : 1 - Math.exp(-time.dt * 9);
    const pos = { x: at.x + (eye.x - at.x) * k, y: at.y + (eye.y - at.y) * k, z: at.z + (eye.z - at.z) * k };
    cam.position = pos;
    cam.quaternion = og.transform.lookAt(pos, focus);
    cam.fov = 55;
    cam.near = 0.1;
    cam.far = 400;
    // The listener rides the camera.
    must(world.set(entity(world, "listener"), og.Transform, { position: pos, quaternion: cam.quaternion }));
    void START;
  },
);
