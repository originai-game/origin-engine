// The math layer (IDL `math`: vec3, vec2, quat, mat4, box3, sphere, plane,
// frustum, ray, color): three.js's methods as pure functions over the plain
// structs the IDL declares. No classes: a Vector3 is `{ x, y, z }`, so the
// values game code passes are the ones columns and reflection already hold,
// and every function returns a new value without changing its arguments.
//
// Determinism: a system may store a result in world state, so every number
// here comes from + - * / and sqrt, which JavaScript engines round
// identically. Trigonometry and powers are written out below (sincos is the
// camera systems' one); Math.sin, Math.atan2 and Math.pow never appear.

import type * as Api from "@origingame/engine";
import { sincos } from "./systems/cameras.ts";

type V2 = Api.Vector2;
type V3 = Api.Vector3;
type Q = Api.Quaternion;
type M4 = Api.Matrix4;
type Box = Api.Box3;
type Sph = Api.Sphere;
type Pl = Api.Plane;
type Fr = Api.Frustum;
type R = Api.Ray;
type C = Api.Color;

const PI = 3.141592653589793;

// ── scalar helpers (deterministic) ─────────────────────────────────────────

/** atan(x) for |x| <= tan(pi/8): its series to x^45 (below 1e-17 there). */
function atanSmall(x: number): number {
  const x2 = x * x;
  let s = 0;
  for (let n = 22; n >= 0; n--) s = s * -x2 + 1 / (2 * n + 1);
  return x * s;
}

/** atan(x) on [0, 1]: reduced by atan(x) = pi/4 + atan((x - 1) / (x + 1)). */
function atanUnit(x: number): number {
  return x > 0.41421356237309503 ? PI / 4 + atanSmall((x - 1) / (x + 1)) : atanSmall(x);
}

/** atan2(y, x) with basic arithmetic only. */
export function atan2(y: number, x: number): number {
  if (x === 0 && y === 0) return 0;
  const ax = Math.abs(x);
  const ay = Math.abs(y);
  const a = ay <= ax ? atanUnit(ay / ax) : PI / 2 - atanUnit(ax / ay);
  const r = x < 0 ? PI - a : a;
  return y < 0 ? -r : r;
}

/** The n-th root of x >= 0 by Newton's method from above (x^(1/n)). */
function root(x: number, n: number): number {
  if (x <= 0) return 0;
  let y = x > 1 ? x : 1;
  for (let i = 0; i < 64; i++) {
    let p = 1;
    for (let k = 0; k < n - 1; k++) p *= y;
    const next = ((n - 1) * y + x / p) / n;
    if (next >= y) break;
    y = next;
  }
  return y;
}

/** sRGB to linear (three.js SRGBToLinear): c^2.4 as c^2 * (c^2)^(1/5). */
function srgbToLinear(c: number): number {
  if (c < 0.04045) return c * 0.0773993808;
  const b = c * 0.9478672986 + 0.0521327014;
  const b2 = b * b;
  return b2 * root(b2, 5);
}

/** Linear to sRGB (three.js LinearToSRGB): c^(1/2.4) as the 12th root of c^5. */
function linearToSrgb(c: number): number {
  if (c < 0.0031308) return c * 12.92;
  const c2 = c * c;
  return 1.055 * root(c2 * c2 * c, 12) - 0.055;
}

const clamp = (v: number, lo: number, hi: number) => (v < lo ? lo : v > hi ? hi : v);

// ── vec3 ───────────────────────────────────────────────────────────────────

const v3 = (x: number, y: number, z: number): V3 => ({ x, y, z });

export const vec3: typeof Api.vec3 = Object.freeze({
  add: (a: V3, b: V3) => v3(a.x + b.x, a.y + b.y, a.z + b.z),
  sub: (a: V3, b: V3) => v3(a.x - b.x, a.y - b.y, a.z - b.z),
  scale: (v: V3, s: number) => v3(v.x * s, v.y * s, v.z * s),
  dot: (a: V3, b: V3) => a.x * b.x + a.y * b.y + a.z * b.z,
  cross: (a: V3, b: V3) => v3(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x),
  length: (v: V3) => Math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z),
  normalize: (v: V3) => {
    const l = Math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z);
    // three.js divides by (length || 1): the zero vector stays zero.
    const d = l || 1;
    return v3(v.x / d, v.y / d, v.z / d);
  },
  lerp: (a: V3, b: V3, t: number) => v3(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t),
  distance: (a: V3, b: V3) => {
    const dx = a.x - b.x;
    const dy = a.y - b.y;
    const dz = a.z - b.z;
    return Math.sqrt(dx * dx + dy * dy + dz * dz);
  },
  applyQuaternion: (v: V3, q: Q) => rotateV(q, v),
});

/** v turned by q, as three.js Vector3.applyQuaternion (t = 2 q x v; v + w t + q x t). */
function rotateV(q: Q, v: V3): V3 {
  const tx = 2 * (q.y * v.z - q.z * v.y);
  const ty = 2 * (q.z * v.x - q.x * v.z);
  const tz = 2 * (q.x * v.y - q.y * v.x);
  return v3(
    v.x + q.w * tx + q.y * tz - q.z * ty,
    v.y + q.w * ty + q.z * tx - q.x * tz,
    v.z + q.w * tz + q.x * ty - q.y * tx,
  );
}

// ── vec2 ───────────────────────────────────────────────────────────────────

const v2 = (x: number, y: number): V2 => ({ x, y });

export const vec2: typeof Api.vec2 = Object.freeze({
  add: (a: V2, b: V2) => v2(a.x + b.x, a.y + b.y),
  sub: (a: V2, b: V2) => v2(a.x - b.x, a.y - b.y),
  scale: (v: V2, s: number) => v2(v.x * s, v.y * s),
  dot: (a: V2, b: V2) => a.x * b.x + a.y * b.y,
  cross: (a: V2, b: V2) => a.x * b.y - a.y * b.x,
  length: (v: V2) => Math.sqrt(v.x * v.x + v.y * v.y),
  normalize: (v: V2) => {
    const d = Math.sqrt(v.x * v.x + v.y * v.y) || 1;
    return v2(v.x / d, v.y / d);
  },
  lerp: (a: V2, b: V2, t: number) => v2(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t),
  distance: (a: V2, b: V2) => {
    const dx = a.x - b.x;
    const dy = a.y - b.y;
    return Math.sqrt(dx * dx + dy * dy);
  },
  rotateAround: (v: V2, center: V2, angle: number) => {
    const [s, c] = sincos(angle);
    const x = v.x - center.x;
    const y = v.y - center.y;
    return v2(x * c - y * s + center.x, x * s + y * c + center.y);
  },
});

// ── quat ───────────────────────────────────────────────────────────────────

const q4 = (x: number, y: number, z: number, w: number): Q => ({ x, y, z, w });

function multiplyQ(a: Q, b: Q): Q {
  return q4(
    a.x * b.w + a.w * b.x + a.y * b.z - a.z * b.y,
    a.y * b.w + a.w * b.y + a.z * b.x - a.x * b.z,
    a.z * b.w + a.w * b.z + a.x * b.y - a.y * b.x,
    a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
  );
}

export const quat: typeof Api.quat = Object.freeze({
  multiply: multiplyQ,
  slerp: (a: Q, b: Q, t: number) => {
    // three.js Quaternion.slerp, step for step.
    if (t === 0) return q4(a.x, a.y, a.z, a.w);
    if (t === 1) return q4(b.x, b.y, b.z, b.w);
    let { x, y, z, w } = b;
    let cosHalf = a.w * w + a.x * x + a.y * y + a.z * z;
    if (cosHalf < 0) {
      x = -x;
      y = -y;
      z = -z;
      w = -w;
      cosHalf = -cosHalf;
    }
    if (cosHalf >= 1) return q4(a.x, a.y, a.z, a.w);
    const sqrSin = 1 - cosHalf * cosHalf;
    if (sqrSin <= Number.EPSILON) {
      const s = 1 - t;
      const r = q4(s * a.x + t * x, s * a.y + t * y, s * a.z + t * z, s * a.w + t * w);
      const l = Math.sqrt(r.x * r.x + r.y * r.y + r.z * r.z + r.w * r.w) || 1;
      return q4(r.x / l, r.y / l, r.z / l, r.w / l);
    }
    const sinHalf = Math.sqrt(sqrSin);
    const half = atan2(sinHalf, cosHalf);
    const ra = sincos((1 - t) * half)[0] / sinHalf;
    const rb = sincos(t * half)[0] / sinHalf;
    return q4(a.x * ra + x * rb, a.y * ra + y * rb, a.z * ra + z * rb, a.w * ra + w * rb);
  },
  fromEuler: (angles: V3, order: string = "XYZ") => {
    // three.js Quaternion.setFromEuler.
    const [s1, c1] = sincos(angles.x / 2);
    const [s2, c2] = sincos(angles.y / 2);
    const [s3, c3] = sincos(angles.z / 2);
    switch (order) {
      case "XYZ":
        return q4(s1 * c2 * c3 + c1 * s2 * s3, c1 * s2 * c3 - s1 * c2 * s3, c1 * c2 * s3 + s1 * s2 * c3, c1 * c2 * c3 - s1 * s2 * s3);
      case "YXZ":
        return q4(s1 * c2 * c3 + c1 * s2 * s3, c1 * s2 * c3 - s1 * c2 * s3, c1 * c2 * s3 - s1 * s2 * c3, c1 * c2 * c3 + s1 * s2 * s3);
      case "ZXY":
        return q4(s1 * c2 * c3 - c1 * s2 * s3, c1 * s2 * c3 + s1 * c2 * s3, c1 * c2 * s3 + s1 * s2 * c3, c1 * c2 * c3 - s1 * s2 * s3);
      case "ZYX":
        return q4(s1 * c2 * c3 - c1 * s2 * s3, c1 * s2 * c3 + s1 * c2 * s3, c1 * c2 * s3 - s1 * s2 * c3, c1 * c2 * c3 + s1 * s2 * s3);
      case "YZX":
        return q4(s1 * c2 * c3 + c1 * s2 * s3, c1 * s2 * c3 + s1 * c2 * s3, c1 * c2 * s3 - s1 * s2 * c3, c1 * c2 * c3 - s1 * s2 * s3);
      case "XZY":
        return q4(s1 * c2 * c3 - c1 * s2 * s3, c1 * s2 * c3 - s1 * c2 * s3, c1 * c2 * s3 + s1 * s2 * c3, c1 * c2 * c3 + s1 * s2 * s3);
      default:
        throw new Error(`quat.fromEuler: order "${order}" is not XYZ, YXZ, ZXY, ZYX, YZX or XZY`);
    }
  },
  invert: (q: Q) => q4(-q.x, -q.y, -q.z, q.w),
  rotate: (q: Q, v: V3) => rotateV(q, v),
});

// ── mat4 ───────────────────────────────────────────────────────────────────

type E = Api.Matrix4["elements"];
const m4 = (e: number[]): M4 => ({ elements: e as unknown as E });

function multiplyM(a: M4, b: M4): M4 {
  const ae = a.elements;
  const be = b.elements;
  const out = new Array<number>(16);
  for (let c = 0; c < 4; c++) {
    for (let r = 0; r < 4; r++) {
      out[c * 4 + r] =
        ae[r]! * be[c * 4]! + ae[4 + r]! * be[c * 4 + 1]! + ae[8 + r]! * be[c * 4 + 2]! + ae[12 + r]! * be[c * 4 + 3]!;
    }
  }
  return m4(out);
}

function invertM(m: M4): M4 {
  // three.js Matrix4.invert (cofactors), a singular matrix to zeros.
  const te = m.elements;
  const [n11, n21, n31, n41, n12, n22, n32, n42, n13, n23, n33, n43, n14, n24, n34, n44] = te as unknown as number[] as [
    number, number, number, number, number, number, number, number,
    number, number, number, number, number, number, number, number,
  ];
  const t11 = n23 * n34 * n42 - n24 * n33 * n42 + n24 * n32 * n43 - n22 * n34 * n43 - n23 * n32 * n44 + n22 * n33 * n44;
  const t12 = n14 * n33 * n42 - n13 * n34 * n42 - n14 * n32 * n43 + n12 * n34 * n43 + n13 * n32 * n44 - n12 * n33 * n44;
  const t13 = n13 * n24 * n42 - n14 * n23 * n42 + n14 * n22 * n43 - n12 * n24 * n43 - n13 * n22 * n44 + n12 * n23 * n44;
  const t14 = n14 * n23 * n32 - n13 * n24 * n32 - n14 * n22 * n33 + n12 * n24 * n33 + n13 * n22 * n34 - n12 * n23 * n34;
  const det = n11 * t11 + n21 * t12 + n31 * t13 + n41 * t14;
  if (det === 0) return m4(new Array<number>(16).fill(0));
  const d = 1 / det;
  return m4([
    t11 * d,
    (n24 * n33 * n41 - n23 * n34 * n41 - n24 * n31 * n43 + n21 * n34 * n43 + n23 * n31 * n44 - n21 * n33 * n44) * d,
    (n22 * n34 * n41 - n24 * n32 * n41 + n24 * n31 * n42 - n21 * n34 * n42 - n22 * n31 * n44 + n21 * n32 * n44) * d,
    (n23 * n32 * n41 - n22 * n33 * n41 - n23 * n31 * n42 + n21 * n33 * n42 + n22 * n31 * n43 - n21 * n32 * n43) * d,
    t12 * d,
    (n13 * n34 * n41 - n14 * n33 * n41 + n14 * n31 * n43 - n11 * n34 * n43 - n13 * n31 * n44 + n11 * n33 * n44) * d,
    (n14 * n32 * n41 - n12 * n34 * n41 - n14 * n31 * n42 + n11 * n34 * n42 + n12 * n31 * n44 - n11 * n32 * n44) * d,
    (n12 * n33 * n41 - n13 * n32 * n41 + n13 * n31 * n42 - n11 * n33 * n42 - n12 * n31 * n43 + n11 * n32 * n43) * d,
    t13 * d,
    (n14 * n23 * n41 - n13 * n24 * n41 - n14 * n21 * n43 + n11 * n24 * n43 + n13 * n21 * n44 - n11 * n23 * n44) * d,
    (n12 * n24 * n41 - n14 * n22 * n41 + n14 * n21 * n42 - n11 * n24 * n42 - n12 * n21 * n44 + n11 * n22 * n44) * d,
    (n13 * n22 * n41 - n12 * n23 * n41 - n13 * n21 * n42 + n11 * n23 * n42 + n12 * n21 * n43 - n11 * n22 * n43) * d,
    t14 * d,
    (n13 * n24 * n31 - n14 * n23 * n31 + n14 * n21 * n33 - n11 * n24 * n33 - n13 * n21 * n34 + n11 * n23 * n34) * d,
    (n14 * n22 * n31 - n12 * n24 * n31 - n14 * n21 * n32 + n11 * n24 * n32 + n12 * n21 * n34 - n11 * n22 * n34) * d,
    (n12 * n23 * n31 - n13 * n22 * n31 + n13 * n21 * n32 - n11 * n23 * n32 - n12 * n21 * n33 + n11 * n22 * n33) * d,
  ]);
}

function transformPointM(m: M4, p: V3): V3 {
  const e = m.elements;
  const w = 1 / (e[3]! * p.x + e[7]! * p.y + e[11]! * p.z + e[15]!);
  return v3(
    (e[0]! * p.x + e[4]! * p.y + e[8]! * p.z + e[12]!) * w,
    (e[1]! * p.x + e[5]! * p.y + e[9]! * p.z + e[13]!) * w,
    (e[2]! * p.x + e[6]! * p.y + e[10]! * p.z + e[14]!) * w,
  );
}

function composeM(p: V3, q: Q, s: V3): M4 {
  // three.js Matrix4.compose.
  const { x, y, z, w } = q;
  const x2 = x + x, y2 = y + y, z2 = z + z;
  const xx = x * x2, xy = x * y2, xz = x * z2;
  const yy = y * y2, yz = y * z2, zz = z * z2;
  const wx = w * x2, wy = w * y2, wz = w * z2;
  return m4([
    (1 - (yy + zz)) * s.x, (xy + wz) * s.x, (xz - wy) * s.x, 0,
    (xy - wz) * s.y, (1 - (xx + zz)) * s.y, (yz + wx) * s.y, 0,
    (xz + wy) * s.z, (yz - wx) * s.z, (1 - (xx + yy)) * s.z, 0,
    p.x, p.y, p.z, 1,
  ]);
}

export const mat4: typeof Api.mat4 = Object.freeze({
  multiply: multiplyM,
  invert: invertM,
  transformPoint: transformPointM,
  transformDirection: (m: M4, d: V3) => {
    const e = m.elements;
    return vec3.normalize(
      v3(e[0]! * d.x + e[4]! * d.y + e[8]! * d.z, e[1]! * d.x + e[5]! * d.y + e[9]! * d.z, e[2]! * d.x + e[6]! * d.y + e[10]! * d.z),
    );
  },
  compose: composeM,
});

// ── box3 ───────────────────────────────────────────────────────────────────

const box = (min: V3, max: V3): Box => ({ min, max });
const emptyBox = (): Box => box(v3(Infinity, Infinity, Infinity), v3(-Infinity, -Infinity, -Infinity));
const isEmptyBox = (b: Box) => b.max.x < b.min.x || b.max.y < b.min.y || b.max.z < b.min.z;

function expand(b: Box, p: V3): Box {
  return box(
    v3(Math.min(b.min.x, p.x), Math.min(b.min.y, p.y), Math.min(b.min.z, p.z)),
    v3(Math.max(b.max.x, p.x), Math.max(b.max.y, p.y), Math.max(b.max.z, p.z)),
  );
}

function boxFromPoints(points: readonly V3[]): Box {
  let b = emptyBox();
  for (const p of points) b = expand(b, p);
  return b;
}

const clampToBox = (b: Box, p: V3) =>
  v3(clamp(p.x, b.min.x, b.max.x), clamp(p.y, b.min.y, b.max.y), clamp(p.z, b.min.z, b.max.z));

function boxCenter(b: Box): V3 {
  return isEmptyBox(b) ? v3(0, 0, 0) : v3((b.min.x + b.max.x) / 2, (b.min.y + b.max.y) / 2, (b.min.z + b.max.z) / 2);
}

function boxIntersectsSphere(b: Box, s: Sph): boolean {
  const c = clampToBox(b, s.center);
  return vec3.dot(vec3.sub(c, s.center), vec3.sub(c, s.center)) <= s.radius * s.radius;
}

export const box3: typeof Api.box3 = Object.freeze({
  empty: emptyBox,
  fromPoints: boxFromPoints,
  isEmpty: isEmptyBox,
  containsPoint: (b: Box, p: V3) =>
    !(p.x < b.min.x || p.x > b.max.x || p.y < b.min.y || p.y > b.max.y || p.z < b.min.z || p.z > b.max.z),
  containsBox: (b: Box, o: Box) =>
    b.min.x <= o.min.x && o.max.x <= b.max.x && b.min.y <= o.min.y && o.max.y <= b.max.y && b.min.z <= o.min.z && o.max.z <= b.max.z,
  intersectsBox: (a: Box, b: Box) =>
    !(b.max.x < a.min.x || b.min.x > a.max.x || b.max.y < a.min.y || b.min.y > a.max.y || b.max.z < a.min.z || b.min.z > a.max.z),
  intersectsSphere: boxIntersectsSphere,
  expandByPoint: expand,
  union: (a: Box, b: Box) =>
    box(
      v3(Math.min(a.min.x, b.min.x), Math.min(a.min.y, b.min.y), Math.min(a.min.z, b.min.z)),
      v3(Math.max(a.max.x, b.max.x), Math.max(a.max.y, b.max.y), Math.max(a.max.z, b.max.z)),
    ),
  getCenter: boxCenter,
  getSize: (b: Box) => (isEmptyBox(b) ? v3(0, 0, 0) : v3(b.max.x - b.min.x, b.max.y - b.min.y, b.max.z - b.min.z)),
  applyMatrix4: (b: Box, m: M4) => {
    if (isEmptyBox(b)) return b;
    const corners: V3[] = [];
    for (const x of [b.min.x, b.max.x]) for (const y of [b.min.y, b.max.y]) for (const z of [b.min.z, b.max.z]) corners.push(transformPointM(m, v3(x, y, z)));
    return boxFromPoints(corners);
  },
});

// ── sphere ─────────────────────────────────────────────────────────────────

export const sphere: typeof Api.sphere = Object.freeze({
  fromPoints: (points: readonly V3[]) => {
    const center = boxCenter(boxFromPoints(points));
    let r2 = 0;
    for (const p of points) {
      const d = vec3.sub(p, center);
      r2 = Math.max(r2, vec3.dot(d, d));
    }
    return { center, radius: Math.sqrt(r2) };
  },
  containsPoint: (s: Sph, p: V3) => {
    const d = vec3.sub(p, s.center);
    return vec3.dot(d, d) <= s.radius * s.radius;
  },
  distanceToPoint: (s: Sph, p: V3) => vec3.distance(p, s.center) - s.radius,
  intersectsSphere: (a: Sph, b: Sph) => {
    const r = a.radius + b.radius;
    const d = vec3.sub(a.center, b.center);
    return vec3.dot(d, d) <= r * r;
  },
  intersectsBox: (s: Sph, b: Box) => boxIntersectsSphere(b, s),
  getBoundingBox: (s: Sph) =>
    s.radius < 0
      ? emptyBox()
      : box(vec3.sub(s.center, v3(s.radius, s.radius, s.radius)), vec3.add(s.center, v3(s.radius, s.radius, s.radius))),
  applyMatrix4: (s: Sph, m: M4) => {
    const e = m.elements;
    // three.js Matrix4.getMaxScaleOnAxis.
    const sx = e[0]! * e[0]! + e[1]! * e[1]! + e[2]! * e[2]!;
    const sy = e[4]! * e[4]! + e[5]! * e[5]! + e[6]! * e[6]!;
    const sz = e[8]! * e[8]! + e[9]! * e[9]! + e[10]! * e[10]!;
    return { center: transformPointM(m, s.center), radius: s.radius * Math.sqrt(Math.max(sx, sy, sz)) };
  },
});

// ── plane ──────────────────────────────────────────────────────────────────

const distanceToPlane = (pl: Pl, p: V3) => vec3.dot(pl.normal, p) + pl.constant;

function normalizePlane(pl: Pl): Pl {
  const inv = 1 / vec3.length(pl.normal);
  return { normal: vec3.scale(pl.normal, inv), constant: pl.constant * inv };
}

function planeFromNormalAndPoint(n: V3, p: V3): Pl {
  const normal = vec3.normalize(n);
  return { normal, constant: -vec3.dot(p, normal) };
}

export const plane: typeof Api.plane = Object.freeze({
  fromNormalAndCoplanarPoint: planeFromNormalAndPoint,
  fromCoplanarPoints: (a: V3, b: V3, c: V3) =>
    planeFromNormalAndPoint(vec3.cross(vec3.sub(c, b), vec3.sub(a, b)), a),
  normalize: normalizePlane,
  distanceToPoint: distanceToPlane,
  projectPoint: (pl: Pl, p: V3) => vec3.sub(p, vec3.scale(pl.normal, distanceToPlane(pl, p))),
  intersectsBox: (pl: Pl, b: Box) => {
    // three.js Plane.intersectsBox: the box's corners on both sides.
    let lo = Infinity;
    let hi = -Infinity;
    for (const x of [b.min.x, b.max.x]) for (const y of [b.min.y, b.max.y]) for (const z of [b.min.z, b.max.z]) {
      const d = distanceToPlane(pl, v3(x, y, z));
      lo = Math.min(lo, d);
      hi = Math.max(hi, d);
    }
    return lo <= 0 && hi >= 0;
  },
  intersectsSphere: (pl: Pl, s: Sph) => Math.abs(distanceToPlane(pl, s.center)) <= s.radius,
});

// ── frustum ────────────────────────────────────────────────────────────────

function frustumFromMatrix(m: M4): Fr {
  // three.js Frustum.setFromProjectionMatrix (WebGL clip space).
  const e = m.elements as unknown as number[];
  const pl = (a: number, b: number, c: number, d: number) => normalizePlane({ normal: v3(a, b, c), constant: d });
  return {
    planes: [
      pl(e[3]! - e[0]!, e[7]! - e[4]!, e[11]! - e[8]!, e[15]! - e[12]!),
      pl(e[3]! + e[0]!, e[7]! + e[4]!, e[11]! + e[8]!, e[15]! + e[12]!),
      pl(e[3]! + e[1]!, e[7]! + e[5]!, e[11]! + e[9]!, e[15]! + e[13]!),
      pl(e[3]! - e[1]!, e[7]! - e[5]!, e[11]! - e[9]!, e[15]! - e[13]!),
      pl(e[3]! - e[2]!, e[7]! - e[6]!, e[11]! - e[10]!, e[15]! - e[14]!),
      pl(e[3]! + e[2]!, e[7]! + e[6]!, e[11]! + e[10]!, e[15]! + e[14]!),
    ],
  };
}

/** The projection of og.Camera (three.js PerspectiveCamera / OrthographicCamera.updateProjectionMatrix). */
function projection(c: Api.Camera, aspect: number): M4 {
  const zoom = c.zoom || 1;
  const near = c.near;
  const far = c.far;
  let left: number, right: number, top: number, bottom: number;
  if (c.orthographic) {
    top = c.height / 2 / zoom;
    bottom = -top;
    right = top * aspect;
    left = -right;
    const w = 1 / (right - left), h = 1 / (top - bottom), p = 1 / (far - near);
    return m4([2 * w, 0, 0, 0, 0, 2 * h, 0, 0, 0, 0, -2 * p, 0, -(right + left) * w, -(top + bottom) * h, -(far + near) * p, 1]);
  }
  const [s, co] = sincos((c.fov * PI) / 360);
  top = (near * s) / co / zoom;
  const height = 2 * top;
  const width = aspect * height;
  left = -width / 2;
  right = left + width;
  bottom = top - height;
  const x = (2 * near) / (right - left);
  const y = (2 * near) / (top - bottom);
  const a = (right + left) / (right - left);
  const b = (top + bottom) / (top - bottom);
  const cc = -(far + near) / (far - near);
  const d = (-2 * far * near) / (far - near);
  return m4([x, 0, 0, 0, 0, y, 0, 0, a, b, cc, -1, 0, 0, d, 0]);
}

const frustumContains = (f: Fr, p: V3) => f.planes.every((pl) => distanceToPlane(pl, p) >= 0);

export const frustum: typeof Api.frustum = Object.freeze({
  fromProjectionMatrix: frustumFromMatrix,
  fromCamera: (camera: Api.Camera, aspect: number) => {
    const world = composeM(camera.position, camera.quaternion, v3(1, 1, 1));
    return frustumFromMatrix(multiplyM(projection(camera, aspect), invertM(world)));
  },
  containsPoint: frustumContains,
  intersectsBox: (f: Fr, b: Box) =>
    f.planes.every((pl) => {
      // three.js: the corner furthest along the normal.
      const p = v3(pl.normal.x > 0 ? b.max.x : b.min.x, pl.normal.y > 0 ? b.max.y : b.min.y, pl.normal.z > 0 ? b.max.z : b.min.z);
      return distanceToPlane(pl, p) >= 0;
    }),
  intersectsSphere: (f: Fr, s: Sph) => f.planes.every((pl) => distanceToPlane(pl, s.center) >= -s.radius),
});

// ── ray ────────────────────────────────────────────────────────────────────

const at = (r: R, t: number) => vec3.add(r.origin, vec3.scale(r.direction, t));

function distanceSqToPoint(r: R, p: V3): number {
  const t = vec3.dot(vec3.sub(p, r.origin), r.direction);
  const q = t < 0 ? r.origin : at(r, t);
  const d = vec3.sub(q, p);
  return vec3.dot(d, d);
}

export const ray: typeof Api.ray = Object.freeze({
  at,
  distanceToPoint: (r: R, p: V3) => Math.sqrt(distanceSqToPoint(r, p)),
  intersectBox: (r: R, b: Box) => {
    // three.js Ray.intersectBox (slabs; a NaN-safe order of the tests).
    const o = r.origin;
    const d = r.direction;
    const slab = (lo: number, hi: number, origin: number, dir: number): [number, number] => {
      const inv = 1 / dir;
      return inv >= 0 ? [(lo - origin) * inv, (hi - origin) * inv] : [(hi - origin) * inv, (lo - origin) * inv];
    };
    let [tmin, tmax] = slab(b.min.x, b.max.x, o.x, d.x);
    const [tymin, tymax] = slab(b.min.y, b.max.y, o.y, d.y);
    if (tmin > tymax || tymin > tmax) return null;
    if (tymin > tmin || Number.isNaN(tmin)) tmin = tymin;
    if (tymax < tmax || Number.isNaN(tmax)) tmax = tymax;
    const [tzmin, tzmax] = slab(b.min.z, b.max.z, o.z, d.z);
    if (tmin > tzmax || tzmin > tmax) return null;
    if (tzmin > tmin || Number.isNaN(tmin)) tmin = tzmin;
    if (tzmax < tmax || Number.isNaN(tmax)) tmax = tzmax;
    if (tmax < 0) return null;
    return at(r, tmin >= 0 ? tmin : tmax);
  },
  intersectSphere: (r: R, s: Sph) => {
    // three.js Ray.intersectSphere.
    const l = vec3.sub(s.center, r.origin);
    const tca = vec3.dot(l, r.direction);
    const d2 = vec3.dot(l, l) - tca * tca;
    const r2 = s.radius * s.radius;
    if (d2 > r2) return null;
    const thc = Math.sqrt(r2 - d2);
    const t0 = tca - thc;
    const t1 = tca + thc;
    if (t1 < 0) return null;
    return at(r, t0 < 0 ? t1 : t0);
  },
  intersectPlane: (r: R, pl: Pl) => {
    // three.js Ray.distanceToPlane, then at().
    const den = vec3.dot(pl.normal, r.direction);
    if (den === 0) return distanceToPlane(pl, r.origin) === 0 ? at(r, 0) : null;
    const t = -(vec3.dot(r.origin, pl.normal) + pl.constant) / den;
    return t >= 0 ? at(r, t) : null;
  },
  intersectTriangle: (r: R, a: V3, b: V3, c: V3, backfaceCulling = false) => {
    // three.js Ray.intersectTriangle (Möller-Trumbore in its form).
    const edge1 = vec3.sub(b, a);
    const edge2 = vec3.sub(c, a);
    const normal = vec3.cross(edge1, edge2);
    let ddn = vec3.dot(r.direction, normal);
    let sign: number;
    if (ddn > 0) {
      if (backfaceCulling) return null;
      sign = 1;
    } else if (ddn < 0) {
      sign = -1;
      ddn = -ddn;
    } else return null;
    const diff = vec3.sub(r.origin, a);
    const ddqxe2 = sign * vec3.dot(r.direction, vec3.cross(diff, edge2));
    if (ddqxe2 < 0) return null;
    const dde1xq = sign * vec3.dot(r.direction, vec3.cross(edge1, diff));
    if (dde1xq < 0) return null;
    if (ddqxe2 + dde1xq > ddn) return null;
    const qdn = -sign * vec3.dot(diff, normal);
    if (qdn < 0) return null;
    return at(r, qdn / ddn);
  },
});

// ── color ──────────────────────────────────────────────────────────────────

const rgb = (r: number, g: number, b: number): C => ({ r, g, b });

function hue2rgb(p: number, q: number, t: number): number {
  if (t < 0) t += 1;
  if (t > 1) t -= 1;
  if (t < 1 / 6) return p + (q - p) * 6 * t;
  if (t < 1 / 2) return q;
  if (t < 2 / 3) return p + (q - p) * 6 * (2 / 3 - t);
  return p;
}

export const color: typeof Api.color = Object.freeze({
  fromHex: (hex: number) => {
    const h = Math.floor(hex);
    return rgb(srgbToLinear(((h >> 16) & 255) / 255), srgbToLinear(((h >> 8) & 255) / 255), srgbToLinear((h & 255) / 255));
  },
  toHex: (c: C) => {
    const ch = (v: number) => Math.round(clamp(linearToSrgb(v), 0, 1) * 255);
    return ((ch(c.r) << 16) | (ch(c.g) << 8) | ch(c.b)) >>> 0;
  },
  lerp: (a: C, b: C, t: number) => rgb(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t),
  fromHsl: (h: number, s: number, l: number) => {
    // three.js Color.setHSL: euclidean modulo of the hue, clamped s and l, then sRGB to linear.
    h = ((h % 1) + 1) % 1;
    s = clamp(s, 0, 1);
    l = clamp(l, 0, 1);
    if (s === 0) {
      const v = srgbToLinear(l);
      return rgb(v, v, v);
    }
    const p = l <= 0.5 ? l * (1 + s) : l + s - l * s;
    const q = 2 * l - p;
    return rgb(srgbToLinear(hue2rgb(q, p, h + 1 / 3)), srgbToLinear(hue2rgb(q, p, h)), srgbToLinear(hue2rgb(q, p, h - 1 / 3)));
  },
});
