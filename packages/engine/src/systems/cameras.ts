// The camera controllers of og.systems.cameras (IDL `camera`: og.OrbitCamera,
// og.FollowCamera, og.FirstPersonCamera, og.Follow2D). They are game-level systems the engine ships
// in TypeScript: a game lists them in GameDesc.systems, so the schedule shows
// them and nothing moves the camera unless the game asked. Each drives
// og.Camera from the first entity in query order that carries its component.
//
// The engine's og.camera system (Rust, after the game's systems) adds what
// needs more than one query: og.CameraRail, og.CameraZone (on the frame), and
// the decay of shake, punch and kick.
//
// og.Camera is world state (snapshots, the world hash), so every number here
// comes from + - * / and sqrt, which JavaScript engines round identically:
// angles go through `sincos` below, never Math.sin, and rotations through
// og.transform.lookAt, which the engine computes.

import { defineSystem, transform, ui, BUILTINS, type SystemToken, type Token, type WorldImpl } from "../runtime.ts";

const T = BUILTINS as unknown as Record<string, Token>;
const Transform = T.Transform!;
const Input = T.Input!;
const Camera = T.Camera!;
const OrbitCamera = T.OrbitCamera!;
const FollowCamera = T.FollowCamera!;
const FirstPersonCamera = T.FirstPersonCamera!;
const Follow2D = T.Follow2D!;
const Pointers = T.Pointers!;
const PointerLock = T.PointerLock!;
const Gamepad = T.Gamepad!;
const Lockstep = T.Lockstep!;

/** The seat this device plays: og.Lockstep.localPlayer while a room runs, else 0. */
function deviceSeat(world: WorldImpl): number {
  const l = world.resource(Lockstep);
  if (!l.ok) return 0;
  const v = l.value as unknown as { running: boolean; localPlayer: number };
  return v.running ? v.localPlayer : 0;
}

/** The first row whose `seat` is -1 (any device) or this device's: -1 when none. */
function driver(a: ArchetypeLike, seat: ArrayLike<number>, mine: number): number {
  for (let i = 0; i < a.count; i++) if (seat[i]! < 0 || seat[i] === mine) return i;
  return -1;
}

const PI = 3.141592653589793;
const TAU = 2 * PI;

/** sin and cos of `x` with basic arithmetic only: the same bits on every host. */
export function sincos(x: number): [number, number] {
  // Reduce to r in [-pi/4, pi/4] and a quadrant.
  const q = Math.round(x / (PI / 2));
  const r = x - q * (PI / 2);
  const r2 = r * r;
  // Taylor series to r^15 / r^14: below 1e-15 on [-pi/4, pi/4].
  let s = 1 / 1307674368000;
  s = s * -r2 + 1 / 6227020800;
  s = s * -r2 + 1 / 39916800;
  s = s * -r2 + 1 / 362880;
  s = s * -r2 + 1 / 5040;
  s = s * -r2 + 1 / 120;
  s = s * -r2 + 1 / 6;
  s = r - r * r2 * s;
  let c = 1 / 87178291200;
  c = c * -r2 + 1 / 479001600;
  c = c * -r2 + 1 / 3628800;
  c = c * -r2 + 1 / 40320;
  c = c * -r2 + 1 / 720;
  c = c * -r2 + 1 / 24;
  c = c * -r2 + 1 / 2;
  c = 1 - r2 * c;
  switch (((q % 4) + 4) % 4) {
    case 0:
      return [s, c];
    case 1:
      return [c, -s];
    case 2:
      return [-s, -c];
    default:
      return [-c, s];
  }
}

const clamp = (v: number, lo: number, hi: number) => (v < lo ? lo : v > hi ? hi : v);

type V3 = { x: number; y: number; z: number };
type Q = { x: number; y: number; z: number; w: number };

/** q * v * q^-1. */
function rotate(q: Q, v: V3): V3 {
  const tx = 2 * (q.y * v.z - q.z * v.y);
  const ty = 2 * (q.z * v.x - q.x * v.z);
  const tz = 2 * (q.x * v.y - q.y * v.x);
  return {
    x: v.x + q.w * tx + (q.y * tz - q.z * ty),
    y: v.y + q.w * ty + (q.z * tx - q.x * tz),
    z: v.z + q.w * tz + (q.x * ty - q.y * tx),
  };
}

/** The fraction of the gap a smoothed follower closes in one tick. */
const ease = (dt: number, smoothTime: number) => (smoothTime > 0 ? Math.min(1, dt / smoothTime) : 1);

interface CameraValue {
  position: V3;
  quaternion: Q;
}

function camera(world: WorldImpl): CameraValue | null {
  const c = world.resource(Camera);
  return c.ok ? (c.value as unknown as CameraValue) : null;
}

function aim(cam: CameraValue, eye: V3, target: V3): void {
  cam.position = eye;
  cam.quaternion = transform.lookAt(eye, target) as Q;
}

interface ArchetypeLike {
  readonly count: number;
  column(t: Token): Record<string, ArrayLike<number> & { [i: number]: number }>;
}
interface QueryLike {
  archetypes(): readonly ArchetypeLike[];
}
interface InputValue {
  pointer: { x: number; y: number };
  pointerInside: boolean;
  wheel: { x: number; y: number };
  buttonDown(b: string): boolean;
  buttonPressed(b: string): boolean;
}

/** og.orbitCamera: OrbitControls over og.Input. */
export const orbitCamera = defineSystem(
  "og.orbitCamera",
  { reads: [Transform, Input], writes: [OrbitCamera, Camera] },
  (q: QueryLike, time: { dt: number }, world: WorldImpl) => {
    for (const a of q.archetypes()) {
      if (a.count === 0) continue;
      const o = a.column(OrbitCamera);
      const p = a.column(Transform).position;
      let azimuth = o.azimuth[0]!;
      let polar = o.polar[0]!;
      let distance = o.distance[0]!;
      const input = world.resource(Input);
      if (o.enabled[0] && input.ok) {
        const i = input.value as unknown as InputValue;
        const ptr = o.pointer;
        if (i.buttonPressed("left") && i.pointerInside) {
          o.dragging[0] = 1;
          ptr[0] = i.pointer.x;
          ptr[1] = i.pointer.y;
        }
        if (o.dragging[0] && i.buttonDown("left")) {
          const height = Math.max(1, (ui.viewport(world) as { height: number }).height);
          const k = (TAU / height) * o.rotateSpeed[0]!;
          azimuth -= (i.pointer.x - ptr[0]!) * k;
          polar -= (i.pointer.y - ptr[1]!) * k;
          ptr[0] = i.pointer.x;
          ptr[1] = i.pointer.y;
        }
        if (!i.buttonDown("left")) o.dragging[0] = 0;
        if (i.wheel.y !== 0) distance *= clamp(1 + 0.001 * i.wheel.y * o.zoomSpeed[0]!, 0.5, 2);
      }
      if (o.autoRotate[0] && !o.dragging[0]) azimuth += o.autoRotateSpeed[0]! * time.dt;
      azimuth -= Math.floor(azimuth / TAU) * TAU;
      polar = clamp(polar, o.minPolarAngle[0]!, o.maxPolarAngle[0]!);
      distance = clamp(distance, o.minDistance[0]!, o.maxDistance[0]!);
      o.azimuth[0] = azimuth;
      o.polar[0] = polar;
      o.distance[0] = distance;
      const [sa, ca] = sincos(azimuth);
      const [sp, cp] = sincos(polar);
      const t = o.target;
      const point = { x: p[0]! + t[0]!, y: p[1]! + t[1]!, z: p[2]! + t[2]! };
      const eye = { x: point.x + distance * sp * sa, y: point.y + distance * cp, z: point.z + distance * sp * ca };
      const cam = camera(world);
      if (cam) aim(cam, eye, point);
      return;
    }
  },
);

/** og.followCamera: a smoothed chase camera. */
export const followCamera = defineSystem(
  "og.followCamera",
  { reads: [Transform, FollowCamera, Lockstep], writes: [Camera] },
  (q: QueryLike, time: { dt: number }, world: WorldImpl) => {
    const mine = deviceSeat(world);
    for (const a of q.archetypes()) {
      const f = a.column(FollowCamera);
      const i = driver(a, f.seat, mine);
      if (i < 0) continue;
      const tr = a.column(Transform);
      const p = tr.position;
      const r = tr.quaternion;
      const pos = { x: p[i * 3]!, y: p[i * 3 + 1]!, z: p[i * 3 + 2]! };
      let offset = { x: f.offset[i * 3]!, y: f.offset[i * 3 + 1]!, z: f.offset[i * 3 + 2]! };
      if (f.followRotation[i]) offset = rotate({ x: r[i * 4]!, y: r[i * 4 + 1]!, z: r[i * 4 + 2]!, w: r[i * 4 + 3]! }, offset);
      const cam = camera(world);
      if (!cam) return;
      const k = ease(time.dt, f.smoothTime[i]!);
      const c = cam.position;
      const eye = {
        x: c.x + (pos.x + offset.x - c.x) * k,
        y: c.y + (pos.y + offset.y - c.y) * k,
        z: c.z + (pos.z + offset.z - c.z) * k,
      };
      aim(cam, eye, { x: pos.x + f.lookOffset[i * 3]!, y: pos.y + f.lookOffset[i * 3 + 1]!, z: pos.z + f.lookOffset[i * 3 + 2]! });
      return;
    }
  },
);

interface PadValue {
  axis(axis: string, pad?: number): number;
  connected(pad?: number): boolean;
}

/** The right stick past a radial dead zone, y down as og.Gamepad's: the pad `gamepad - 1`, or the most deflected of all. */
function rightStick(world: WorldImpl, gamepad: number): { x: number; y: number } {
  const pads = world.resource(Gamepad);
  if (!pads.ok) return { x: 0, y: 0 };
  const g = pads.value as unknown as PadValue;
  let best = { x: 0, y: 0, m: 0 };
  for (let pad = gamepad > 0 ? gamepad - 1 : 0; pad < (gamepad > 0 ? gamepad : 8); pad++) {
    if (!g.connected(pad)) continue;
    const x = g.axis("rightX", pad);
    const y = g.axis("rightY", pad);
    const m = Math.sqrt(x * x + y * y);
    if (m > best.m) best = { x, y, m };
  }
  const dead = 0.15;
  if (best.m <= dead) return { x: 0, y: 0 };
  const k = Math.min(1, (best.m - dead) / (1 - dead)) / best.m;
  return { x: best.x * k, y: best.y * k };
}

/** og.firstPersonCamera: mouse-look while the pointer is locked and the right stick, or the game's own yaw and pitch. */
export const firstPersonCamera = defineSystem(
  "og.firstPersonCamera",
  { reads: [Transform, Pointers, PointerLock, Gamepad, Lockstep], writes: [FirstPersonCamera, Camera] },
  (q: QueryLike, time: { dt: number }, world: WorldImpl) => {
    const mine = deviceSeat(world);
    for (const a of q.archetypes()) {
      const f = a.column(FirstPersonCamera);
      const i = driver(a, f.seat, mine);
      if (i < 0) continue;
      // Every row clamps and wraps its own angles (the same on every peer); only the device's turns by this device's mouse.
      for (let r = 0; r < a.count; r++) {
        let yaw = f.yaw[r]!;
        let pitch = f.pitch[r]!;
        if (r === i && f.enabled[r]) {
          const flip = f.invertY[r] ? -1 : 1;
          const lock = world.resource(PointerLock);
          const ptr = world.resource(Pointers);
          if (lock.ok && ptr.ok && (lock.value as unknown as { locked: boolean }).locked) {
            const m = (ptr.value as unknown as { movement: { x: number; y: number } }).movement;
            yaw -= m.x * f.sensitivity[r]!;
            pitch -= m.y * f.sensitivity[r]! * flip;
          }
          const stick = rightStick(world, f.gamepad[r]!);
          const turn = f.stickSpeed[r]! * time.dt;
          yaw -= stick.x * turn;
          pitch -= stick.y * turn * flip;
        }
        yaw -= Math.floor((yaw + PI) / TAU) * TAU;
        pitch = clamp(pitch, f.minPitch[r]!, f.maxPitch[r]!);
        f.yaw[r] = yaw;
        f.pitch[r] = pitch;
      }
      const cam = camera(world);
      if (!cam) return;
      const p = a.column(Transform).position;
      // rotation = Ry(yaw) * Rx(pitch), from half angles.
      const [sy, cy] = sincos(f.yaw[i]! / 2);
      const [sp, cp] = sincos(f.pitch[i]! / 2);
      cam.position = { x: p[i * 3]! + f.eye[i * 3]!, y: p[i * 3 + 1]! + f.eye[i * 3 + 1]!, z: p[i * 3 + 2]! + f.eye[i * 3 + 2]! };
      cam.quaternion = { x: cy * sp, y: sy * cp, z: -sy * sp, w: cy * cp };
      return;
    }
  },
);

/** og.follow2D: a dead-zone follower in the xy plane, with look-ahead and per-axis smoothing. */
export const follow2D = defineSystem(
  "og.follow2D",
  { reads: [Transform, Lockstep], writes: [Follow2D, Camera] },
  (q: QueryLike, time: { dt: number }, world: WorldImpl) => {
    const mine = deviceSeat(world);
    for (const a of q.archetypes()) {
      const f = a.column(Follow2D);
      const i = driver(a, f.seat, mine);
      if (i < 0) continue;
      const p = a.column(Transform).position;
      // Every row keeps its own lead (world state, the same on every peer); the device's row moves the camera.
      const last = f.last;
      const lead = f.lead;
      const max = f.lookAheadMax;
      for (let r = 0; r < a.count; r++) {
        const px = p[r * 3]!;
        const py = p[r * 3 + 1]!;
        const la = f.lookAhead[r]!;
        if (la > 0 && time.dt > 0) {
          const fresh = last[r * 2] === 0 && last[r * 2 + 1] === 0;
          const vx = fresh ? 0 : (px - last[r * 2]!) / time.dt;
          const vy = fresh ? 0 : (py - last[r * 2 + 1]!) / time.dt;
          const k = ease(time.dt, f.lookAheadSmoothing[r]!);
          lead[r * 2] = lead[r * 2]! + (clamp(vx * la, -max[r * 2]!, max[r * 2]!) - lead[r * 2]!) * k;
          lead[r * 2 + 1] = lead[r * 2 + 1]! + (clamp(vy * la, -max[r * 2 + 1]!, max[r * 2 + 1]!) - lead[r * 2 + 1]!) * k;
        } else {
          lead[r * 2] = 0;
          lead[r * 2 + 1] = 0;
        }
        last[r * 2] = px;
        last[r * 2 + 1] = py;
      }
      const cam = camera(world);
      if (!cam) return;
      const c = cam.position;
      const goal = (cur: number, target: number, dead: number) => {
        const d = target - cur;
        return d > dead ? target - dead : d < -dead ? target + dead : cur;
      };
      const st = f.smoothTime[i]!;
      const sy = f.smoothTimeY[i]!;
      const kx = ease(time.dt, st);
      const ky = ease(time.dt, sy >= 0 ? sy : st);
      let x = c.x + (goal(c.x, p[i * 3]! + f.offset[i * 2]! + lead[i * 2]!, f.deadZone[i * 2]!) - c.x) * kx;
      let y = c.y + (goal(c.y, p[i * 3 + 1]! + f.offset[i * 2 + 1]! + lead[i * 2 + 1]!, f.deadZone[i * 2 + 1]!) - c.y) * ky;
      if (f.clamp[i]) {
        x = clamp(x, f.boundsMin[i * 2]!, Math.max(f.boundsMin[i * 2]!, f.boundsMax[i * 2]!));
        y = clamp(y, f.boundsMin[i * 2 + 1]!, Math.max(f.boundsMin[i * 2 + 1]!, f.boundsMax[i * 2 + 1]!));
      }
      if (x !== c.x || y !== c.y) cam.position = { x, y, z: c.z };
      return;
    }
  },
);

/** og.systems.cameras, in the IDL's order. */
export const cameras: readonly SystemToken[] = Object.freeze([orbitCamera, followCamera, firstPersonCamera, follow2D]);
