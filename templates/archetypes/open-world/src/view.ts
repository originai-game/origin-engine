// The view: how a 3D starter's camera follows its seats. The same file in
// every 3D starter (packages/create's test keeps the copies equal); what
// differs is src/creation.ts's `view`, `camera` and `rig`, from the create
// block's view choice (packages/create/bin/starters.ts VIEW_PRESETS):
//   first-person-3d  og.FirstPersonCamera at each hero's eye;
//   third-person-3d  og.FollowCamera behind each hero, turned by its look;
//   isometric-2.5d   og.FollowCamera at the look's fixed ArtCamera angle
//                    (true isometric: 35.264° down, turned 45°) on an
//                    orthographic camera; the look's aim turns the hero only.
// Each seat's look (yaw, pitch) is world state in its SeatLook, turned only
// by its og.PlayerInput's aim (src/seats.ts: degrees this tick), so it
// replays and plays over a room. Alone and online the main camera follows
// this device's seat (the controllers' `seat`); two seats on one device
// split the screen with an og.CameraView per seat, left and right.

import { og, defineComponent, defineSystem, f32, math, u32 } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { creation } from "./creation.ts";
import { LOCAL, localSeats } from "./seats.ts";

const view: string = creation.view;
export const FIRST_PERSON = view === "first-person-3d";
export const ISOMETRIC = view === "isometric-2.5d";
const DEG = Math.PI / 180;
/** The look's fixed angle (isometric): from creation.camera, the look's ArtCamera. */
const camera = creation.camera as { projection: string; pitch?: number; yaw?: number; fovMin?: number; fovMax?: number };
const ISO_PITCH = (camera.pitch ?? 35.264) * DEG;
const ISO_YAW = (camera.yaw ?? 45) * DEG;

/** A seat's look: its heading and pitch, turned by its aim. */
export const SeatLook = defineComponent("SeatLook", {
  yaw: f32(0, { unit: "rad", doc: "The heading about +Y: 0 looks along -Z, growing to the left." }),
  pitch: f32(0, { unit: "rad", doc: "Above the horizon (below 0 looks down)." }),
  seat: u32(0),
});

/** The rig's distances: third person behind and above, isometric's distance and view height. */
export interface ViewDesc {
  /** Eye height above the hero's position (first person, and the point the others look at). */
  readonly eye: number;
  readonly behind: number;
  readonly above: number;
  /** Isometric: the orthographic view's height in world units. */
  readonly isoHeight: number;
  readonly minPitch?: number;
  readonly maxPitch?: number;
}
let desc: ViewDesc = { eye: 1.6, behind: 5, above: 1.8, isoHeight: 16 };

/** One split-screen view per seat (local play), in seat order. */
const views: Entity[] = [];

/**
 * First person: the layer a seat's body is drawn on (set its og.Model or
 * og.Mesh `layers` to it). Each seat's eye draws the world (layer 0) and
 * the other seats' bodies, never its own head; its shadow still falls.
 */
export function bodyLayer(seat: number): number {
  return 2 << seat;
}

/** The layers a seat's camera draws: everything but its own body in first person. */
function eyeLayers(seat: number, seats: number): number {
  if (!FIRST_PERSON) return 1;
  let mask = 1;
  for (let s = 0; s < seats; s++) if (s !== seat) mask |= bodyLayer(s);
  return mask;
}

/**
 * Call once in start, after spawning one hero per seat (each with
 * og.PlayerInput): gives each hero its SeatLook and controller, and the
 * main camera its projection.
 */
export function setupView(world: World, heroes: readonly Entity[], d: Partial<ViewDesc> = {}): void {
  desc = { ...desc, ...d };
  const main = must(world.resource(og.Camera));
  main.orthographic = ISOMETRIC;
  if (ISOMETRIC) main.height = desc.isoHeight;
  if (camera.fovMin || camera.fovMax) main.fov = Math.min(camera.fovMax ?? 179, Math.max(camera.fovMin ?? 1, FIRST_PERSON ? 75 : 55));
  heroes.forEach((hero, seat) => {
    const yaw = ISOMETRIC ? ISO_YAW : 0;
    must(world.insert(hero, SeatLook, { yaw, pitch: ISOMETRIC ? 0 : FIRST_PERSON ? 0 : -0.25, seat }));
    // Local play draws through the views below; the controller is still there (the main frame draws nothing).
    if (FIRST_PERSON) must(world.insert(hero, og.FirstPersonCamera, { seat, enabled: false, eye: { x: 0, y: desc.eye, z: 0 } }));
    else must(world.insert(hero, og.FollowCamera, { seat, smoothTime: ISOMETRIC ? 0.15 : 0.08, lookOffset: { x: 0, y: ISOMETRIC ? 0 : desc.eye, z: 0 }, offset: offsetOf(yaw, -0.25) }));
  });
  if (LOCAL) {
    // Two halves: the main frame draws only the background behind them.
    main.layers = 0;
    heroes.forEach((_, seat) => {
      const v = must(world.spawn({ name: `view-${seat}` }));
      must(world.insert(v, og.CameraView, { viewportX: seat * 0.5, viewportWidth: 0.5, viewportHeight: 1, orthographic: ISOMETRIC, height: desc.isoHeight, fov: main.fov, order: seat, layers: eyeLayers(seat, heroes.length) }));
      views.push(v);
    });
  } else {
    main.layers = eyeLayers(localSeats(world)[0] ?? 0, heroes.length);
  }
}

/** Where the camera sits from the point it looks at, for a heading and pitch (third person, isometric). */
function offsetOf(yaw: number, pitch: number): { x: number; y: number; z: number } {
  if (ISOMETRIC) {
    const r = 60;
    return { x: r * math.cos(ISO_PITCH) * math.sin(ISO_YAW), y: r * math.sin(ISO_PITCH), z: r * math.cos(ISO_PITCH) * math.cos(ISO_YAW) };
  }
  // Behind the heading (+Z when yaw is 0), raised as the look tips down.
  const back = desc.behind * math.cos(pitch);
  return { x: back * math.sin(yaw), y: desc.above - desc.behind * math.sin(pitch), z: back * math.cos(yaw) };
}

/** The ground direction a stick (x right, y forward) walks for a seat's heading: unit-free, length of the stick. */
export function walkDirection(yaw: number, x: number, y: number): { x: number; z: number } {
  const s = math.sin(yaw), c = math.cos(yaw);
  // forward (-sin, -cos), right (cos, -sin)
  return { x: -s * y + c * x, z: -c * y - s * x };
}

/** The unit vector a seat looks along (its aim in 3D). */
export function lookDirection(look: { yaw: number; pitch: number }): { x: number; y: number; z: number } {
  const cp = math.cos(look.pitch);
  return { x: -math.sin(look.yaw) * cp, y: math.sin(look.pitch), z: -math.cos(look.yaw) * cp };
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The controller every hero carries for this view. */
const RIG = FIRST_PERSON ? og.FirstPersonCamera : og.FollowCamera;

/**
 * Turns each seat's look by its aim and points its camera: the
 * controllers (alone, online) or its split-screen view (local). List it
 * after Seats and before ...og.systems.cameras.
 */
export const ViewRig = defineSystem(
  "ViewRig",
  { reads: [og.PlayerInput], writes: [SeatLook, og.Transform, RIG] },
  (q, _time, world) => {
    const lo = desc.minPitch ?? (FIRST_PERSON ? -1.45 : -1.1);
    const hi = desc.maxPitch ?? (FIRST_PERSON ? 1.45 : 0.35);
    for (const a of q.archetypes()) {
      const input = a.column(og.PlayerInput);
      const look = a.column(SeatLook);
      const p = a.column(og.Transform).position;
      const rig = a.column(RIG) as unknown as { yaw: Float32Array; pitch: Float32Array; offset: Float32Array };
      for (let i = 0; i < a.count; i++) {
        let yaw = look.yaw[i]! - input.aimX[i]! * DEG;
        let pitch = look.pitch[i]! + input.aimY[i]! * DEG;
        yaw -= Math.floor((yaw + Math.PI) / (2 * Math.PI)) * 2 * Math.PI;
        pitch = Math.max(lo, Math.min(hi, pitch));
        look.yaw[i] = yaw;
        look.pitch[i] = ISOMETRIC ? 0 : pitch;
        if (FIRST_PERSON) {
          rig.yaw[i] = yaw;
          rig.pitch[i] = pitch;
        } else if (!ISOMETRIC) {
          const o = offsetOf(yaw, pitch);
          rig.offset[i * 3] = o.x;
          rig.offset[i * 3 + 1] = o.y;
          rig.offset[i * 3 + 2] = o.z;
        }
        if (!LOCAL) continue;
        const v = views[look.seat[i]!];
        if (v === undefined) continue;
        const at = { x: p[i * 3]!, y: p[i * 3 + 1]! + (ISOMETRIC ? 0 : desc.eye), z: p[i * 3 + 2]! };
        // As the controllers place it: the eye itself in first person, else the offset from the hero's position.
        const off = FIRST_PERSON ? { x: 0, y: 0, z: 0 } : offsetOf(yaw, pitch);
        const from = FIRST_PERSON ? at : { x: p[i * 3]!, y: p[i * 3 + 1]!, z: p[i * 3 + 2]! };
        const eye = { x: from.x + off.x, y: from.y + off.y, z: from.z + off.z };
        const quaternion = FIRST_PERSON ? yawPitch(yaw, pitch) : og.transform.lookAt(eye, at);
        must(world.set(v, og.Transform, { position: eye, quaternion }));
      }
    }
  },
);

/** Ry(yaw) * Rx(pitch). */
function yawPitch(yaw: number, pitch: number) {
  const sy = math.sin(yaw / 2), cy = math.cos(yaw / 2), sp = math.sin(pitch / 2), cp = math.cos(pitch / 2);
  return { x: cy * sp, y: sy * cp, z: -sy * sp, w: cy * cp };
}
