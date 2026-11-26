// The player: walks the wanderer along the cliffs (WASD or the left stick)
// and turns the camera round them
// (the arrow keys or the right stick). The wanderer turns to where it walks;
// the camera trails over its right shoulder. Everything is world state, so a
// script of key presses replays to the same frames.
import { defineResource, defineSystem, f64, math, og, quat } from "@origingame/engine";
import type { ClipAsset, Vector3, World } from "@origingame/engine";
import { CAMERA, PATH } from "./layout.ts";
import { LAST, Quest } from "./quest.ts";
import { must } from "./must.ts";
import { CARRY, Clips, LANTERN, START_HEADING } from "./wanderer.ts";

const WALK = 1.4;
/** How fast the wanderer turns to where it walks, and how fast the speed changes. */
const TURN = 8, ACCEL = 6;
/** How fast the camera turns on the look keys (radians a second), and how quickly it catches up. */
const LOOK = 1.6, FOLLOW = 5;
/** The camera's place behind the wanderer: back, over the right shoulder, up. */
const BACK = 4.2, SIDE = 1.0, EYE = 2.7;

export const Player = defineResource("Player", {
  /** Where the wanderer faces, and the camera's heading (radians about +Y; 0 looks down +Z). */
  heading: f64(START_HEADING), yaw: f64(START_HEADING), speed: f64(),
  /** Metres walked, and where the wanderer stands, for the HUD and the playtest. */
  walked: f64(), x: f64(), z: f64(),
  /** The lantern's swing on its cord: forwards (pitch) and sideways (roll), radians, and their rates. */
  pitch: f64(), roll: f64(), pitchRate: f64(), rollRate: f64(), lastSpeed: f64(), lastHeading: f64(START_HEADING),

});

export function buildPlayer(world: World): void {
  must(og.input.defineAction(world, { name: "move", kind: "axis2", label: "Walk", bindings: [{ up: "KeyW", down: "KeyS", left: "KeyA", right: "KeyD" }, { control: "pad.leftXY" }] }));
  must(og.input.defineAction(world, { name: "look", kind: "axis2", label: "Look", bindings: [{ up: "ArrowUp", down: "ArrowDown", left: "ArrowLeft", right: "ArrowRight" }, { control: "pad.rightXY" }] }));
  frame(world, 1);
}

const forward = (yaw: number): Vector3 => ({ x: math.sin(yaw), y: 0, z: math.cos(yaw) });
/** The shortest signed turn from a to b. */
const turnTo = (a: number, b: number): number => math.atan2(math.sin(b - a), math.cos(b - a));

/** Puts the camera behind the wanderer, `k` of the way from where it is (1: there). */
function frame(world: World, k: number): void {
  const body = world.find("wanderer");
  if (body === null) return;
  const p = must(world.get(body, og.Transform)).position, yaw = must(world.resource(Player)).yaw;
  const f = forward(yaw), right = { x: -f.z, y: 0, z: f.x };
  const want = { x: p.x - f.x * BACK + right.x * SIDE, y: p.y + EYE, z: p.z - f.z * BACK + right.z * SIDE };
  const cam = must(world.resource(og.Camera));
  const now = k >= 1 ? want : { x: cam.position.x + (want.x - cam.position.x) * k, y: cam.position.y + (want.y - cam.position.y) * k, z: cam.position.z + (want.z - cam.position.z) * k };
  cam.position = now;
  // Down the path and a little below level, as the cover frame looks.
  const look = { x: now.x + f.x * 40, y: now.y - (CAMERA.drop * 40) / 78, z: now.z + f.z * 40 };
  cam.quaternion = og.transform.lookAt(now, look);
}

/**
 * walk:path: the stick a player would hold to walk the path, towards its
 * next point (null when the player walks). Each point passed, the next.
 */
function autopilot(world: World, f: Vector3, right: { x: number; z: number }): { x: number; y: number } | null {
  const q = must(world.resource(Quest));
  const body = world.find("wanderer");
  if (q.auto === 0 || body === null) return null;
  const p = must(world.get(body, og.Transform)).position;
  while (q.leg <= LAST) {
    const [x, z] = PATH[q.leg]!;
    if ((x - p.x) ** 2 + (z - p.z) ** 2 > 1.5 ** 2) break;
    q.leg += 1;
  }
  if (q.leg > LAST) return { x: 0, y: 0 };
  const [x, z] = PATH[q.leg]!;
  const dx = x - p.x, dz = z - p.z, d = math.sqrt(dx * dx + dz * dz);
  return { x: (dx * right.x + dz * right.z) / d, y: (dx * f.x + dz * f.z) / d };
}

/** How far the ground falls away `reach` metres ahead of the feet. */
function drop(world: World, at: Vector3, ahead: Vector3, reach: number): number {
  const land = world.find("coast");
  if (land === null) return 0;
  const data = must(world.get(land, og.Terrain)).data;
  const there = must(og.terrain.sample(world, { data, origin: { x: 0, y: 0, z: 0 }, x: at.x + ahead.x * reach, z: at.z + ahead.z * reach })).height;
  return at.y - there;
}

/** The lantern as a damped pendulum from its hook: the wanderer's starts and stops swing it fore and aft, its turns to the side. */
function swing(world: World, player: { pitch: number; roll: number; pitchRate: number; rollRate: number; speed: number; lastSpeed: number; heading: number; lastHeading: number }, dt: number): void {
  const lantern = world.find("lantern");
  if (lantern === null || dt <= 0) return;
  const g = 9.81 / LANTERN.drop, damp = 1.6;
  const along = (player.speed - player.lastSpeed) / dt, across = player.speed * turnTo(player.lastHeading, player.heading) / dt;
  player.pitchRate += (-g * math.sin(player.pitch) - damp * player.pitchRate + (along / LANTERN.drop) * math.cos(player.pitch)) * dt;
  player.rollRate += (-g * math.sin(player.roll) - damp * player.rollRate - (across / LANTERN.drop) * math.cos(player.roll)) * dt;
  player.pitch += player.pitchRate * dt;
  player.roll += player.rollRate * dt;
  player.lastSpeed = player.speed;
  player.lastHeading = player.heading;
  // Hung from the hook: the lantern's origin LANTERN.drop below it, turned with the cord.
  const q = quat.multiply(og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, player.pitch), og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, player.roll));
  const d = LANTERN.drop, sp = math.sin(player.pitch), cp = math.cos(player.pitch), sr = math.sin(player.roll), cr = math.cos(player.roll);
  // (0, -d, 0) turned by the roll about z, then the pitch about x.
  const x = d * sr, y0 = -d * cr;
  must(world.set(lantern, og.Transform, { position: { x: CARRY.hook.x + x, y: CARRY.hook.y + y0 * cp, z: CARRY.hook.z + y0 * sp }, quaternion: q }));
}

/** Carries the hand's grip with the wanderer (the staff and the hook are its children). */
function carry(world: World, at: Vector3, heading: number): void {
  const grip = world.find("staff-grip");
  if (grip === null) return;
  const c = math.cos(heading), s = math.sin(heading);
  const put = (q: { x: number; y: number; z: number }) => ({ x: at.x + q.x * c + q.z * s, y: at.y + q.y, z: at.z - q.x * s + q.z * c });
  must(world.set(grip, og.IkChain, { target: put(CARRY.grip), pole: put(CARRY.pole) }));
}

export const Walk = defineSystem("Walk", { reads: [og.Input, og.Terrain, Clips], writes: [Player, Quest, og.AnimationPlayer, og.CharacterController, og.Transform, og.Camera, og.IkChain] }, (_q, time, world) => {
  const body = world.find("wanderer");
  if (body === null) return;
  const player = must(world.resource(Player));
  const dt = time.dt;

  const look = og.input.actionVector(world, "look");
  player.yaw += look.x * LOOK * dt;

  // Where the stick points, seen from the camera's heading.
  const f = forward(player.yaw), right = { x: -f.z, z: f.x };
  const stick = autopilot(world, f, right) ?? og.input.actionVector(world, "move");
  const dir = { x: f.x * stick.y + right.x * stick.x, z: f.z * stick.y + right.z * stick.x };
  const tilt = Math.min(1, math.sqrt(dir.x * dir.x + dir.z * dir.z));
  const want = tilt * WALK;
  player.speed += Math.max(-ACCEL * dt, Math.min(ACCEL * dt, want - player.speed));
  if (tilt > 0.05) player.heading += turnTo(player.heading, math.atan2(dir.x, dir.z)) * Math.min(1, TURN * dt);
  const ahead = forward(player.heading);
  const at = must(world.get(body, og.Transform)).position;
  // The cliffs' edge stops the wanderer: it slows as the ground ahead falls
  // away and never steps down more than a metre and a half (a cliff, the shore).
  if (player.speed > 0) {
    const near = drop(world, at, ahead, 0.8), far = drop(world, at, ahead, 2.5);
    player.speed = Math.min(player.speed, near > 1.5 ? 0 : far > 1.5 ? 0.5 : WALK);
  }
  must(world.set(body, og.CharacterController, { move: { x: ahead.x * player.speed, y: 0, z: ahead.z * player.speed } }));
  must(world.set(body, og.Transform, { quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, player.heading) }));
  player.walked += player.speed * dt;
  if (player.walked > 5) og.assert.reachable("the wanderer walked along the path", {});

  // The camera eases round behind the wanderer as it turns, unless the player looks about.
  if (tilt > 0.05 && look.x === 0) player.yaw += turnTo(player.yaw, player.heading) * Math.min(1, 1.2 * dt) * (player.speed / WALK);
  // Walk while it moves, its stride matched to the speed; stand otherwise.
  const clips = must(world.resource(Clips));
  const moving = player.speed > 0.15;
  must(world.set(body, og.AnimationPlayer, { clip: (moving ? clips.walk : clips.idle) as unknown as ClipAsset, speed: moving ? Math.max(0.6, player.speed / WALK) : 1 }));
  player.x = at.x;
  player.z = at.z;
  carry(world, at, player.heading);
  swing(world, player, dt);
  frame(world, Math.min(1, FOLLOW * dt));
});
