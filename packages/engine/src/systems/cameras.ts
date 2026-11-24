// The camera controllers of og.systems.cameras (IDL `camera`: og.OrbitCamera,
// og.FollowCamera, og.Follow2D, og.FramingCamera). They are game-level systems the engine ships
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
// angles go through `sincos` (math.ts), never Math.sin, and rotations through
// og.transform.lookAt, which the engine computes.

import type { World } from "@origingame/engine";
import { sincos } from "../math.ts";
import { calls, defineSystem, ui, BUILTINS, type SystemToken, type Token } from "../runtime.ts";

const T = BUILTINS as unknown as Record<string, Token>;
const Transform = T.Transform!;
const Input = T.Input!;
const Camera = T.Camera!;
const OrbitCamera = T.OrbitCamera!;
const FollowCamera = T.FollowCamera!;
const Follow2D = T.Follow2D!;
const FramingCamera = T.FramingCamera!;

const PI = 3.141592653589793;
const TAU = 2 * PI;


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

function camera(world: World): CameraValue | null {
  const c = world.resource(Camera as never);
  return c.ok ? (c.value as unknown as CameraValue) : null;
}

function aim(cam: CameraValue, eye: V3, target: V3): void {
  cam.position = eye;
  cam.quaternion = calls.transform.lookAt(eye, target) as Q;
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
  buttonJustPressed(b: string): boolean;
}

/** og.orbitCamera: OrbitControls over og.Input. */
export const orbitCamera = defineSystem(
  "og.orbitCamera",
  { reads: [Transform, Input], writes: [OrbitCamera, Camera] },
  (q: QueryLike, time: { dt: number }, world: World) => {
    for (const a of q.archetypes()) {
      if (a.count === 0) continue;
      const o = a.column(OrbitCamera);
      const p = a.column(Transform).position;
      let azimuth = o.azimuth[0]!;
      let polar = o.polar[0]!;
      let distance = o.distance[0]!;
      const input = world.resource(Input as never);
      if (o.enabled[0] && input.ok) {
        const i = input.value as unknown as InputValue;
        const ptr = o.pointer;
        if (i.buttonJustPressed("left") && i.pointerInside) {
          o.dragging[0] = 1;
          ptr[0] = i.pointer.x;
          ptr[1] = i.pointer.y;
        }
        if (o.dragging[0] && i.buttonDown("left")) {
          const height = Math.max(1, (ui.viewport(world as never) as { height: number }).height);
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
  { reads: [Transform, FollowCamera], writes: [Camera] },
  (q: QueryLike, time: { dt: number }, world: World) => {
    for (const a of q.archetypes()) {
      if (a.count === 0) continue;
      const f = a.column(FollowCamera);
      const tr = a.column(Transform);
      const p = tr.position;
      const r = tr.quaternion;
      const pos = { x: p[0]!, y: p[1]!, z: p[2]! };
      let offset = { x: f.offset[0]!, y: f.offset[1]!, z: f.offset[2]! };
      // With followRotation the offset is right, up and behind the entity: an entity faces its +Z with its
      // right at -X, so the offset turns with it after a half turn about +Y.
      if (f.followRotation[0]) offset = rotate({ x: r[0]!, y: r[1]!, z: r[2]!, w: r[3]! }, { x: -offset.x, y: offset.y, z: -offset.z });
      const cam = camera(world);
      if (!cam) return;
      const k = ease(time.dt, f.smoothTime[0]!);
      const c = cam.position;
      const eye = {
        x: c.x + (pos.x + offset.x - c.x) * k,
        y: c.y + (pos.y + offset.y - c.y) * k,
        z: c.z + (pos.z + offset.z - c.z) * k,
      };
      aim(cam, eye, { x: pos.x + f.lookOffset[0]!, y: pos.y + f.lookOffset[1]!, z: pos.z + f.lookOffset[2]! });
      return;
    }
  },
);

/** og.follow2D: a dead-zone follower in the xy plane, with look-ahead and per-axis smoothing. */
export const follow2D = defineSystem(
  "og.follow2D",
  { reads: [Transform], writes: [Follow2D, Camera] },
  (q: QueryLike, time: { dt: number }, world: World) => {
    for (const a of q.archetypes()) {
      if (a.count === 0) continue;
      const f = a.column(Follow2D);
      const p = a.column(Transform).position;
      const cam = camera(world);
      if (!cam) return;
      const c = cam.position;
      const goal = (cur: number, target: number, dead: number) => {
        const d = target - cur;
        return d > dead ? target - dead : d < -dead ? target + dead : cur;
      };
      // Look-ahead: lead by the entity's velocity times lookAhead, eased.
      const last = f.last;
      const lead = f.lead;
      const px = p[0]!;
      const py = p[1]!;
      const la = f.lookAhead[0]!;
      if (la > 0 && time.dt > 0) {
        const fresh = last[0] === 0 && last[1] === 0;
        const vx = fresh ? 0 : (px - last[0]!) / time.dt;
        const vy = fresh ? 0 : (py - last[1]!) / time.dt;
        const max = f.lookAheadMax;
        const k = ease(time.dt, f.lookAheadSmoothing[0]!);
        lead[0] = lead[0]! + (clamp(vx * la, -max[0]!, max[0]!) - lead[0]!) * k;
        lead[1] = lead[1]! + (clamp(vy * la, -max[1]!, max[1]!) - lead[1]!) * k;
      } else {
        lead[0] = 0;
        lead[1] = 0;
      }
      last[0] = px;
      last[1] = py;
      const st = f.smoothTime[0]!;
      const sy = f.smoothTimeY[0]!;
      const kx = ease(time.dt, st);
      const ky = ease(time.dt, sy >= 0 ? sy : st);
      let x = c.x + (goal(c.x, px + f.offset[0]! + lead[0]!, f.deadZone[0]!) - c.x) * kx;
      let y = c.y + (goal(c.y, py + f.offset[1]! + lead[1]!, f.deadZone[1]!) - c.y) * ky;
      if (f.clamp[0]) {
        x = clamp(x, f.boundsMin[0]!, Math.max(f.boundsMin[0]!, f.boundsMax[0]!));
        y = clamp(y, f.boundsMin[1]!, Math.max(f.boundsMin[1]!, f.boundsMax[1]!));
      }
      if (x !== c.x || y !== c.y) cam.position = { x, y, z: c.z };
      return;
    }
  },
);

interface FramingValue {
  height: number;
  centre: V3;
  beats: readonly number[];
  area: { x: number; y: number };
  ticks: boolean;
  time: number;
  speed: number;
}
interface ProjectionValue {
  fov: number;
  orthographic: boolean;
}

const DEG = PI / 180;
/** Numbers per beat in og.FramingCamera.beats: time, screen height, x, y, yaw, pitch. */
const BEAT = 6;

/** The beats' numbers at clock `s`: eased from the beat at or before it to the next. */
function framing(beats: readonly number[], s: number): { h: number; x: number; y: number; yaw: number; pitch: number } {
  const n = Math.floor(beats.length / BEAT);
  let i = 0;
  while (i < n - 2 && s >= beats[(i + 1) * BEAT]!) i++;
  const a = i * BEAT;
  const b = Math.min(i + 1, n - 1) * BEAT;
  const span = beats[b]! - beats[a]!;
  const u = b === a || !(span > 0) ? (s >= beats[b]! ? 1 : 0) : clamp((s - beats[a]!) / span, 0, 1);
  const e = u * u * (3 - 2 * u);
  const mix = (k: number) => beats[a + k]! + (beats[b + k]! - beats[a + k]!) * e;
  // The distance is proportional to 1/h: easing 1/h moves the camera evenly.
  const inv = 1 / beats[a + 1]! + (1 / beats[b + 1]! - 1 / beats[a + 1]!) * e;
  return { h: 1 / inv, x: mix(2), y: mix(3), yaw: mix(4) * DEG, pitch: mix(5) * DEG };
}

const dot = (a: V3, b: V3) => a.x * b.x + a.y * b.y + a.z * b.z;

/**
 * The eye that frames a level rectangle (width across the yawed view, depth along the ground
 * ahead, centred on c): its on-screen bounds fill h of the frame in their tighter direction and
 * their centre sits at (nx, ny), in -1..1 with +y up. Under perspective the near edge projects
 * wider than the far one, so no closed form: the distance along the view is bisected on the fill,
 * and at each distance the sideways and upward shift (which leave every corner's depth alone) is
 * solved for the bounds' centre. Fixed iteration counts keep it deterministic.
 */
function frameArea(c: V3, width: number, depth: number, sy: number, cy: number, forward: V3, side: V3, up: V3, tx: number, ty: number, h: number, nx: number, ny: number): V3 {
  // The corners relative to c, in the view's axes: k across, m up, z along the view.
  const k: number[] = [];
  const m: number[] = [];
  const z: number[] = [];
  for (const [i, j] of [[-1, -1], [1, -1], [-1, 1], [1, 1]] as const) {
    // Across the view is the screen's right (-cy, 0, sy); ahead on the ground is (sy, 0, cy).
    const v = { x: (-cy * i * width + sy * j * depth) / 2, y: 0, z: (sy * i * width + cy * j * depth) / 2 };
    k.push(dot(v, side));
    m.push(dot(v, up));
    z.push(dot(v, forward));
  }
  const near = Math.max(...z.map((v) => -v)) + 1e-3 * (width + depth);
  let shiftX = 0;
  let shiftY = 0;
  // The bounds' fill at distance l, after solving the shift that centres them where the beat says.
  const fill = (l: number): number => {
    const mean = l + (z[0]! + z[1]! + z[2]! + z[3]!) / 4;
    let w = 0;
    let t = 0;
    for (let n = 0; n < 16; n++) {
      let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity;
      for (let q = 0; q < 4; q++) {
        const d = l + z[q]!;
        const x = (k[q]! - shiftX) / (d * tx);
        const y = (m[q]! - shiftY) / (d * ty);
        x0 = Math.min(x0, x); x1 = Math.max(x1, x);
        y0 = Math.min(y0, y); y1 = Math.max(y1, y);
      }
      w = (x1 - x0) / 2;
      t = (y1 - y0) / 2;
      // Moving the eye right moves every corner left by about 1/(depth tx).
      shiftX += ((x0 + x1) / 2 - nx) * mean * tx;
      shiftY += ((y0 + y1) / 2 - ny) * mean * ty;
    }
    return Math.max(w, t);
  };
  let lo = near;
  let hi = near + (width + depth) / (ty < tx ? ty : tx);
  for (let n = 0; n < 40 && fill(hi) > h; n++) hi = near + (hi - near) * 2;
  for (let n = 0; n < 48; n++) {
    const mid = (lo + hi) / 2;
    if (fill(mid) > h) lo = mid;
    else hi = mid;
  }
  fill(hi);
  return {
    x: c.x - hi * forward.x + shiftX * side.x + shiftY * up.x,
    y: c.y - hi * forward.y + shiftX * side.y + shiftY * up.y,
    z: c.z - hi * forward.z + shiftX * side.z + shiftY * up.z,
  };
}

/** og.framingCamera: the subject's height and place in the frame, and the view's yaw and pitch, per beat. */
export const framingCamera = defineSystem(
  "og.framingCamera",
  { reads: [Transform], writes: [FramingCamera, Camera] },
  (q: { archetypes(): readonly { count: number; column(t: Token): Record<string, ArrayLike<number>>; entity(i: number): number }[] }, time: { dt: number }, world: World) => {
    for (const a of q.archetypes()) {
      if (a.count === 0) continue;
      // This tick frames the clock as it stands; then the clock moves on.
      const e = a.entity(0);
      const got = (world.get as (e: number, t: Token) => { ok: true; value: FramingValue } | { ok: false })(e, FramingCamera);
      if (!got.ok) return;
      const f = got.value;
      const now = f.time;
      const moved = (world.set as (e: number, t: Token, v: object) => { ok: boolean; reason?: string })(e, FramingCamera, { ...f, time: now + f.speed * (f.ticks ? 1 : time.dt) });
      if (!moved.ok) throw new Error(`og.framingCamera: ${moved.reason}`);
      const beats = f.beats;
      const cam = camera(world);
      const projection = cam as unknown as ProjectionValue | null;
      const area = f.area.x > 0 && f.area.y > 0;
      if (!cam || projection!.orthographic || !(area || f.height > 0) || beats.length < BEAT) return;
      for (let k = 1; k < beats.length; k += BEAT) if (!(beats[k]! > 0)) return;
      const p = a.column(Transform).position;
      const b = framing(beats, now);
      const [sy, cy] = sincos(b.yaw);
      const [sp, cp] = sincos(b.pitch);
      const forward = { x: sy * cp, y: sp, z: cy * cp };
      // The screen's right and up: looking along +Z, the right is -X.
      const side = { x: -cy, y: 0, z: sy };
      const up = {
        x: side.y * forward.z - side.z * forward.y,
        y: side.z * forward.x - side.x * forward.z,
        z: side.x * forward.y - side.y * forward.x,
      };
      const [sf, cf] = sincos((projection!.fov * DEG) / 2);
      const ty = sf / cf;
      const view = ui.viewport(world as never) as { width: number; height: number };
      const tx = ty * (view.height > 0 ? view.width / view.height : 1);
      const nx = 2 * b.x - 1;
      const ny = 1 - 2 * b.y;
      const c = { x: p[0]! + f.centre.x, y: p[1]! + f.centre.y, z: p[2]! + f.centre.z };
      let eye: V3;
      if (area) {
        eye = frameArea(c, f.area.x, f.area.y, sy, cy, forward, side, up, tx, ty, b.h, nx, ny);
      } else {
        // At depth d along the view the subject's height fills h of the frame.
        const d = f.height / (2 * ty * b.h);
        const ray = {
          x: forward.x + nx * tx * side.x + ny * ty * up.x,
          y: forward.y + nx * tx * side.y + ny * ty * up.y,
          z: forward.z + nx * tx * side.z + ny * ty * up.z,
        };
        eye = { x: c.x - d * ray.x, y: c.y - d * ray.y, z: c.z - d * ray.z };
      }
      aim(cam, eye, { x: eye.x + forward.x, y: eye.y + forward.y, z: eye.z + forward.z });
      return;
    }
  },
);

/** og.systems.cameras, in the IDL's order. */
export const cameras: readonly SystemToken[] = Object.freeze([orbitCamera, followCamera, follow2D, framingCamera]);
