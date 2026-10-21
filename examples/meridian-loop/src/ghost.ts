// The tapes and the ghost of the best lap (the Tapes system, over the
// tapes' chunks). Each tick of a race the driver's input code goes on the
// run's tape, for the replay. While a lap is driven, the car's pose at the
// start of every second tick goes on the current lap's tape (`tape-cur-*`); a
// lap that beats the record keeps its tape as the best (src/race.ts). From
// the start of every lap the ghost is drawn where the best lap's car was at
// the same time into its lap, between two recorded poses, its wheels at
// their places on the body turning with the distance it has covered.
//
// Why poses and not the best lap's inputs replayed on a second og.Vehicle:
// a raycast car's wheels land on any collider below them, whatever its
// og.Collider layers and mask, so a ghost car overlapping the player's
// (both start each lap on the same spot) lifts it off the road (FINDINGS
// #3).

import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Phase, Race, RUN_TICKS, TapeChunk, TAPE_TICKS } from "./components.ts";
import { tapeView } from "./tapes.ts";
import { bytesOf, euler, lerpAngle, POSE_BYTES, POSE_EVERY, poseAt, poseBytes, quat } from "./tape.ts";
import { fromStart, nearest, SAMPLES } from "./track.ts";
import { entity, must } from "./world.ts";

/** The wheels on a car's body (x across, y up, z forward), front left first as og.physics.vehicleWheels gives them. */
export const WHEEL_AT: readonly [number, number, number][] = [
  [0.85, 0.0, 1.3],
  [-0.85, 0.0, 1.3],
  [0.85, 0.0, -1.3],
  [-0.85, 0.0, -1.3],
];

const PARK = { x: 0, y: -50, z: 0 };

/** Under the world, out of sight. */
export function hideGhost(world: World): void {
  must(world.set(entity(world, "ghost"), og.Transform, { position: PARK }));
  for (let i = 0; i < 4; i++) must(world.set(entity(world, `ghost-wheel-${i}`), og.Transform, { position: PARK }));
}

function rotate(q: { x: number; y: number; z: number; w: number }, v: [number, number, number]): { x: number; y: number; z: number } {
  const [vx, vy, vz] = v;
  const tx = 2 * (q.y * vz - q.z * vy);
  const ty = 2 * (q.z * vx - q.x * vz);
  const tz = 2 * (q.x * vy - q.y * vx);
  return { x: vx + q.w * tx + (q.y * tz - q.z * ty), y: vy + q.w * ty + (q.z * tx - q.x * tz), z: vz + q.w * tz + (q.x * ty - q.y * tx) };
}

export const Tapes = defineSystem("Tapes", { writes: [Race, TapeChunk, og.Transform], after: ["Driver"] }, (q, time, world) => {
  const r = must(world.resource(Race));
  if (r.held || time.scaledDt === 0 || r.phase !== Phase.Racing) return;
  // r.clock has moved on this tick: the lap is `into` ticks old at the start of it.
  const into = r.clock - 1 - r.lapStart;
  // Per-entity reads and writes first (og-core §4), then the tape's columns.
  const car = must(world.get(entity(world, "car"), og.Transform));
  let place: { position: { x: number; y: number; z: number }; quaternion: { x: number; y: number; z: number; w: number }; spin: number } | null = null;
  const view = tapeView(q);
  if (view) {
    // The run's inputs, two bytes a tick, for the replay.
    const ro = r.runLen < RUN_TICKS ? view.offset("run", r.runLen * 2) : -1;
    if (ro >= 0) {
      const [hi, lo] = bytesOf(r.code);
      view.bytes[ro] = hi;
      view.bytes[ro + 1] = lo;
      r.runLen += 1;
    }
    if (into >= 0 && into % POSE_EVERY === 0) {
      const len = r.curLen;
      const o = view.offset("cur", len * POSE_BYTES);
      if ((len + 1) * POSE_EVERY <= TAPE_TICKS && o >= 0) {
        const e = euler(car.quaternion);
        const p = car.position;
        view.bytes.set(poseBytes({ x: p.x, y: p.y, z: p.z, yaw: e.yaw, pitch: e.pitch, roll: e.roll }), o);
        r.curLen = len + 1;
      }
    }
    if (r.ghostActive && r.bestLen > 0) {
      const len = r.bestLen;
      const tick = Math.max(0, into + 1);
      const f = tick / POSE_EVERY;
      const i0 = Math.min(len - 1, Math.floor(f));
      const i1 = Math.min(len - 1, i0 + 1);
      const u = i1 === i0 ? 0 : f - i0;
      const a0 = poseAt(view.bytes, view.offset("best", i0 * POSE_BYTES));
      const a1 = poseAt(view.bytes, view.offset("best", i1 * POSE_BYTES));
      const x = a0.x + (a1.x - a0.x) * u;
      const y = a0.y + (a1.y - a0.y) * u;
      const z = a0.z + (a1.z - a0.z) * u;
      const quaternion = quat(lerpAngle(a0.yaw, a1.yaw, u), lerpAngle(a0.pitch, a1.pitch, u), lerpAngle(a0.roll, a1.roll, u));
      r.ghostTick = tick;
      // The ghost's own progress: the distance it has covered turns its wheels.
      const gi = nearest(x, z, r.ghostAt, r.ghostAt < 0 ? 0 : 14);
      r.ghostAt = gi;
      place = { position: { x, y, z }, quaternion, spin: SAMPLES[gi]!.s / 0.35 };
      // Past the end of its lap the ghost waits beyond the line a second, then goes.
      if (tick > len * POSE_EVERY + 60) {
        r.ghostActive = false;
        place = null;
      }
    }
  }
  if (r.ghostActive && place) {
    must(world.set(entity(world, "ghost"), og.Transform, { position: place.position, quaternion: place.quaternion }));
    const spin = { x: Math.sin(place.spin / 2), y: 0, z: 0, w: Math.cos(place.spin / 2) };
    for (let i = 0; i < 4; i++) {
      const off = rotate(place.quaternion, WHEEL_AT[i]!);
      const q2 = place.quaternion;
      const wq = { x: q2.w * spin.x + q2.x * spin.w + q2.y * spin.z - q2.z * spin.y, y: q2.w * spin.y - q2.x * spin.z + q2.y * spin.w + q2.z * spin.x, z: q2.w * spin.z + q2.x * spin.y - q2.y * spin.x + q2.z * spin.w, w: q2.w * spin.w - q2.x * spin.x - q2.y * spin.y - q2.z * spin.z };
      must(world.set(entity(world, `ghost-wheel-${i}`), og.Transform, { position: { x: place.position.x + off.x, y: place.position.y + off.y, z: place.position.z + off.z }, quaternion: wq }));
    }
  } else if (!r.ghostActive && r.ghostTick !== 0) {
    r.ghostTick = 0;
    hideGhost(world);
  }
  void fromStart;
});
