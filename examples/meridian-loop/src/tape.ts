// The tapes as text (src/components.ts's Tape): str fields, since a
// component has no array field (FINDINGS #4).
//
// - Inputs: two characters a tick, the driver's steering (33 steps),
//   throttle (17) and brake (5) packed in 12 bits. The car is always driven
//   by the quantised values, so replaying the tape from the same world
//   gives the same run (the replay, src/replay.ts).
// - Poses: the ghost's, below.

import { MAP } from "./track.ts";

const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
const INDEX = new Map([...ALPHABET].map((c, i) => [c.charCodeAt(0), i]));

export interface Intent {
  steer: number;
  throttle: number;
  brake: number;
}

const clamp = (v: number, a: number, b: number) => Math.max(a, Math.min(b, v));

/** The intent as its 12-bit code. */
export function encode(i: Intent): number {
  const s = Math.round((clamp(i.steer, -1, 1) + 1) * 16);
  const t = Math.round((clamp(i.throttle, -1, 1) + 1) * 8);
  const b = Math.round(clamp(i.brake, 0, 1) * 4);
  return (s * 17 + t) * 5 + b;
}

export function decode(code: number): Intent {
  const b = code % 5;
  const t = Math.floor(code / 5) % 17;
  const s = Math.floor(code / 85);
  return { steer: s / 16 - 1, throttle: t / 8 - 1, brake: b / 4 };
}

/** What the car is driven by: the intent as the tape keeps it. */
export function quantise(i: Intent): Intent {
  return decode(encode(i));
}

/** The two bytes of a code (ASCII), for a str column. */
export function bytesOf(code: number): [number, number] {
  return [ALPHABET.charCodeAt(code >> 6), ALPHABET.charCodeAt(code & 63)];
}

/** The code at two bytes of a str column. */
export function codeAt(hi: number, lo: number): number {
  return ((INDEX.get(hi) ?? 0) << 6) | (INDEX.get(lo) ?? 0);
}

/** Whether a text is a well-formed tape of n ticks (a stored record read back). */
export function validTape(text: string, n: number): boolean {
  if (text.length !== n * 2) return false;
  for (let i = 0; i < text.length; i++) if (!INDEX.has(text.charCodeAt(i))) return false;
  return true;
}

// ---------------------------------------------------------------- poses
//
// The ghost is drawn from the best lap's poses, one every POSE_EVERY ticks:
// x and z in centimetres from the mini-map's corner and y in centimetres
// above -10 m (three characters each, 18 bits), yaw, pitch and roll in
// 4096ths of a turn (two characters each): 15 characters a pose, 40.5 KB
// for the longest lap kept. (Why poses and not the inputs replayed on a
// second car: FINDINGS #3.)


export const POSE_EVERY = 2;
export const POSE_BYTES = 15;

export interface Pose {
  x: number;
  y: number;
  z: number;
  /** Euler angles, radians: yaw about y, then pitch about x, then roll about z. */
  yaw: number;
  pitch: number;
  roll: number;
}

const TURN = 4096;

function put(out: number[], v: number, chars: number): void {
  for (let k = chars - 1; k >= 0; k--) out.push(ALPHABET.charCodeAt((v >> (6 * k)) & 63));
}

function get(bytes: ArrayLike<number>, at: number, chars: number): number {
  let v = 0;
  for (let k = 0; k < chars; k++) v = (v << 6) | (INDEX.get(bytes[at + k]!) ?? 0);
  return v;
}

const angle = (a: number) => (((Math.round((a / (2 * Math.PI)) * TURN) % TURN) + TURN) % TURN);
const unangle = (n: number) => ((n >= TURN / 2 ? n - TURN : n) / TURN) * 2 * Math.PI;

/** A pose as its 15 bytes (ASCII). */
export function poseBytes(p: Pose): number[] {
  const out: number[] = [];
  put(out, Math.max(0, Math.min(262143, Math.round((p.x - MAP.minX) * 100))), 3);
  put(out, Math.max(0, Math.min(262143, Math.round((p.y + 10) * 100))), 3);
  put(out, Math.max(0, Math.min(262143, Math.round((p.z - MAP.minZ) * 100))), 3);
  put(out, angle(p.yaw), 2);
  put(out, angle(p.pitch), 2);
  put(out, angle(p.roll), 2);
  return out;
}

/** The pose at byte `at` of a pose tape. */
export function poseAt(bytes: ArrayLike<number>, at: number): Pose {
  return {
    x: get(bytes, at, 3) / 100 + MAP.minX,
    y: get(bytes, at + 3, 3) / 100 - 10,
    z: get(bytes, at + 6, 3) / 100 + MAP.minZ,
    yaw: unangle(get(bytes, at + 9, 2)),
    pitch: unangle(get(bytes, at + 11, 2)),
    roll: unangle(get(bytes, at + 13, 2)),
  };
}

type Quat = { x: number; y: number; z: number; w: number };

/** Yaw, pitch, roll (YXZ, three.js's order for a vehicle) of a quaternion. */
export function euler(q: Quat): { yaw: number; pitch: number; roll: number } {
  // The rotation matrix's entries needed for YXZ.
  const m13 = 2 * (q.x * q.z + q.w * q.y);
  const m23 = 2 * (q.y * q.z - q.w * q.x);
  const m33 = 1 - 2 * (q.x * q.x + q.y * q.y);
  const m21 = 2 * (q.x * q.y + q.w * q.z);
  const m22 = 1 - 2 * (q.x * q.x + q.z * q.z);
  const pitch = Math.asin(Math.max(-1, Math.min(1, -m23)));
  return { yaw: Math.atan2(m13, m33), pitch, roll: Math.atan2(m21, m22) };
}

/** The quaternion of yaw, pitch, roll (YXZ). */
export function quat(yaw: number, pitch: number, roll: number): Quat {
  const cy = Math.cos(yaw / 2), sy = Math.sin(yaw / 2);
  const cp = Math.cos(pitch / 2), sp = Math.sin(pitch / 2);
  const cr = Math.cos(roll / 2), sr = Math.sin(roll / 2);
  // q = qy * qx * qz
  return {
    x: cy * sp * cr + sy * cp * sr,
    y: sy * cp * cr - cy * sp * sr,
    z: cy * cp * sr - sy * sp * cr,
    w: cy * cp * cr + sy * sp * sr,
  };
}

/** Between two angles the short way round. */
export function lerpAngle(a: number, b: number, t: number): number {
  let d = b - a;
  while (d > Math.PI) d -= 2 * Math.PI;
  while (d < -Math.PI) d += 2 * Math.PI;
  return a + d * t;
}
