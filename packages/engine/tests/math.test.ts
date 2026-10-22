// The math layer (IDL `math`) against three.js's documented results: one
// table of cases per namespace, each expected value worked out by hand from
// three.js's documentation (a rotation's effect, a box's corners, an sRGB
// value's linear form), not copied from the code under test. Inputs are
// asymmetric so a swapped argument, a transposed matrix or a wrong
// rotation order gives a different answer.
import { describe, expect, test } from "bun:test";
import { box3, color, frustum, mat4, plane, quat, ray, sphere, vec2, vec3 } from "../src/index.ts";
import type { Camera, Quaternion, Vector3 } from "@origingame/engine";

const S = Math.SQRT1_2;
const v = (x: number, y: number, z: number): Vector3 => ({ x, y, z });
const q = (x: number, y: number, z: number, w: number): Quaternion => ({ x, y, z, w });
/** A turn of `deg` about the unit axis (x, y, z): three.js Quaternion.setFromAxisAngle. */
const axis = (x: number, y: number, z: number, deg: number) => {
  const h = (deg * Math.PI) / 360;
  return q(x * Math.sin(h), y * Math.sin(h), z * Math.sin(h), Math.cos(h));
};
const near = (got: Record<string, number>, want: Record<string, number>, eps = 1e-9) => {
  for (const k of Object.keys(want)) {
    if (Math.abs(got[k]! - want[k]!) > eps) throw new Error(`${k}: got ${JSON.stringify(got)}, want ${JSON.stringify(want)}`);
  }
};
const sameRotation = (a: Quaternion, b: Quaternion, eps = 1e-9) => {
  // q and -q are one rotation.
  const s = Math.sign(a.w * b.w + a.x * b.x + a.y * b.y + a.z * b.z) || 1;
  near({ x: a.x * s, y: a.y * s, z: a.z * s, w: a.w * s }, b as never, eps);
};

describe("vec3", () => {
  const a = v(1, 2, 3);
  const b = v(4, -5, 6);
  test.each([
    ["add", () => vec3.add(a, b), v(5, -3, 9)],
    ["sub (a - b, not b - a)", () => vec3.sub(a, b), v(-3, 7, -3)],
    ["scale", () => vec3.scale(a, -2), v(-2, -4, -6)],
    ["cross x * y = z (right-handed)", () => vec3.cross(v(1, 0, 0), v(0, 1, 0)), v(0, 0, 1)],
    ["cross is a x b", () => vec3.cross(a, b), v(27, 6, -13)],
    ["normalize", () => vec3.normalize(v(3, 0, -4)), v(0.6, 0, -0.8)],
    ["normalize keeps the zero vector", () => vec3.normalize(v(0, 0, 0)), v(0, 0, 0)],
    ["lerp", () => vec3.lerp(v(0, 0, 0), v(10, -10, 4), 0.25), v(2.5, -2.5, 1)],
    ["lerp extrapolates (t not clamped)", () => vec3.lerp(v(1, 1, 1), v(2, 3, 4), 2), v(3, 5, 7)],
    ["applyQuaternion: 90 degrees about z turns +x to +y", () => vec3.applyQuaternion(v(1, 0, 0), axis(0, 0, 1, 90)), v(0, 1, 0)],
    ["applyQuaternion: 90 degrees about x turns +y to +z", () => vec3.applyQuaternion(v(2, 1, 0), axis(1, 0, 0, 90)), v(2, 0, 1)],
  ] as const)("%s", (_name, got, want) => near(got() as never, want as never));
  test("dot, length and distance", () => {
    expect(vec3.dot(a, b)).toBe(12);
    expect(vec3.length(v(2, -3, 6))).toBe(7);
    expect(vec3.distance(v(1, 1, 1), v(4, 5, 1))).toBe(5);
  });
  test("arguments are never changed", () => {
    const x = v(1, 2, 3);
    const out = vec3.normalize(x);
    expect(x).toEqual(v(1, 2, 3));
    expect(out).not.toBe(x);
  });
});

describe("vec2", () => {
  test.each([
    ["add", () => vec2.add({ x: 1, y: 2 }, { x: 3, y: -7 }), { x: 4, y: -5 }],
    ["sub", () => vec2.sub({ x: 1, y: 2 }, { x: 3, y: -7 }), { x: -2, y: 9 }],
    ["scale", () => vec2.scale({ x: 1, y: -2 }, 3), { x: 3, y: -6 }],
    ["normalize", () => vec2.normalize({ x: -5, y: 12 }), { x: -5 / 13, y: 12 / 13 }],
    ["lerp", () => vec2.lerp({ x: 2, y: 0 }, { x: 4, y: -8 }, 0.75), { x: 3.5, y: -6 }],
    ["rotateAround: a quarter turn is counter-clockwise", () => vec2.rotateAround({ x: 1, y: 0 }, { x: 0, y: 0 }, Math.PI / 2), { x: 0, y: 1 }],
    ["rotateAround a pivot", () => vec2.rotateAround({ x: 3, y: 1 }, { x: 1, y: 1 }, Math.PI), { x: -1, y: 1 }],
    ["rotateAround by -30 degrees", () => vec2.rotateAround({ x: 2, y: 0 }, { x: 0, y: 0 }, -Math.PI / 6), { x: Math.sqrt(3), y: -1 }],
  ] as const)("%s", (_name, got, want) => near(got() as never, want as never));
  test("dot, cross (b counter-clockwise of a is positive), length, distance", () => {
    expect(vec2.dot({ x: 1, y: 2 }, { x: 3, y: -4 })).toBe(-5);
    expect(vec2.cross({ x: 1, y: 0 }, { x: 0, y: 1 })).toBe(1);
    expect(vec2.cross({ x: 2, y: 3 }, { x: 4, y: 5 })).toBe(-2);
    expect(vec2.length({ x: -3, y: 4 })).toBe(5);
    expect(vec2.distance({ x: 1, y: 1 }, { x: 4, y: -3 })).toBe(5);
  });
});

describe("quat", () => {
  const z90 = axis(0, 0, 1, 90);
  const x90 = axis(1, 0, 0, 90);
  test("multiply(a, b) turns by b first, then a", () => {
    // x90 takes +z to -y, then z90 takes -y to +x; the other order leaves -y.
    near(quat.rotate(quat.multiply(z90, x90), v(0, 0, 1)) as never, v(1, 0, 0) as never);
    near(quat.rotate(quat.multiply(x90, z90), v(0, 0, 1)) as never, v(0, -1, 0) as never);
    sameRotation(quat.multiply(z90, z90), q(0, 0, 1, 0));
  });
  test.each([
    ["half way from identity to 90 degrees about z is 45", () => quat.slerp(q(0, 0, 0, 1), z90, 0.5), axis(0, 0, 1, 45)],
    ["a quarter of the way", () => quat.slerp(q(0, 0, 0, 1), z90, 0.25), axis(0, 0, 1, 22.5)],
    ["the shorter arc when b is negated", () => quat.slerp(q(0, 0, 0, 1), q(-z90.x, -z90.y, -z90.z, -z90.w), 0.5), axis(0, 0, 1, 45)],
    ["between two turns about y", () => quat.slerp(axis(0, 1, 0, 20), axis(0, 1, 0, 140), 0.75), axis(0, 1, 0, 110)],
    ["t 0 and 1 are the ends", () => quat.slerp(x90, z90, 1), z90],
  ] as const)("slerp: %s", (_name, got, want) => sameRotation(got(), want, 1e-12));
  test("fromEuler matches the intrinsic product of its order, as three.js", () => {
    const e = v(0.3, -1.1, 2.4);
    const qx = axis(1, 0, 0, (e.x * 180) / Math.PI);
    const qy = axis(0, 1, 0, (e.y * 180) / Math.PI);
    const qz = axis(0, 0, 1, (e.z * 180) / Math.PI);
    const by: Record<string, Quaternion[]> = {
      XYZ: [qx, qy, qz],
      YXZ: [qy, qx, qz],
      ZXY: [qz, qx, qy],
      ZYX: [qz, qy, qx],
      YZX: [qy, qz, qx],
      XZY: [qx, qz, qy],
    };
    for (const [order, [a, b, c]] of Object.entries(by)) {
      sameRotation(quat.fromEuler(e, order), quat.multiply(quat.multiply(a!, b!), c!), 1e-12);
    }
    // The default order is XYZ; a quarter turn about x alone.
    sameRotation(quat.fromEuler(v(0.3, -1.1, 2.4)), quat.fromEuler(e, "XYZ"));
    near(quat.fromEuler(v(Math.PI / 2, 0, 0)) as never, q(S, 0, 0, S) as never, 1e-15);
    expect(() => quat.fromEuler(e, "xyz")).toThrow(/order/);
  });
  test("invert undoes the rotation; rotate is applyQuaternion", () => {
    const r = quat.multiply(axis(0, 1, 0, 33), x90);
    sameRotation(quat.multiply(r, quat.invert(r)), q(0, 0, 0, 1));
    near(quat.rotate(quat.invert(z90), v(0, 1, 0)) as never, v(1, 0, 0) as never);
    expect(quat.rotate(r, v(1, 2, 3))).toEqual(vec3.applyQuaternion(v(1, 2, 3), r));
  });
});

describe("mat4", () => {
  // Scale by 2, turn 90 degrees about z, move by (1, 2, 3).
  const m = mat4.compose(v(1, 2, 3), axis(0, 0, 1, 90), v(2, 2, 2));
  test("compose is column-major with the translation in elements 12 to 14", () => {
    const e = m.elements;
    near({ a: e[0], b: e[1], c: e[4], d: e[5], tx: e[12], ty: e[13], tz: e[14], w: e[15] } as never, { a: 0, b: 2, c: -2, d: 0, tx: 1, ty: 2, tz: 3, w: 1 }, 1e-15);
  });
  test.each([
    ["transformPoint: scale, turn, then move", () => mat4.transformPoint(m, v(1, 0, 0)), v(1, 4, 3)],
    ["transformDirection: turned, normalised, not moved", () => mat4.transformDirection(m, v(1, 0, 0)), v(0, 1, 0)],
    ["the inverse takes the point back", () => mat4.transformPoint(mat4.invert(m), v(1, 4, 3)), v(1, 0, 0)],
    ["multiply(a, b) applies b first", () => mat4.transformPoint(mat4.multiply(mat4.compose(v(10, 0, 0), q(0, 0, 0, 1), v(1, 1, 1)), m), v(1, 0, 0)), v(11, 4, 3)],
    [
      "transformPoint divides by w (three.js applyMatrix4)",
      () => mat4.transformPoint({ elements: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 4] } as never, v(2, 4, 8)),
      v(0.5, 1, 2),
    ],
  ] as const)("%s", (_name, got, want) => near(got() as never, want as never, 1e-12));
  test("invert(m) * m is the identity; a singular matrix inverts to zeros", () => {
    const r = mat4.compose(v(-3, 0.5, 7), quat.fromEuler(v(0.4, 1.2, -0.7)), v(1, 3, 0.5));
    const id = mat4.multiply(mat4.invert(r), r).elements;
    [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1].forEach((x, i) => expect(Math.abs(id[i]! - x)).toBeLessThan(1e-12));
    expect([...mat4.invert(mat4.compose(v(1, 1, 1), q(0, 0, 0, 1), v(1, 0, 1))).elements]).toEqual(new Array(16).fill(0));
  });
});

describe("box3", () => {
  const b = box3.fromPoints([v(1, 2, 3), v(-1, 5, 0), v(0, 3, 1)]);
  test("fromPoints, center and size", () => {
    expect(b).toEqual({ min: v(-1, 2, 0), max: v(1, 5, 3) });
    expect(box3.getCenter(b)).toEqual(v(0, 3.5, 1.5));
    expect(box3.getSize(b)).toEqual(v(2, 3, 3));
  });
  test("the empty box", () => {
    const e = box3.empty();
    expect(box3.isEmpty(e)).toBe(true);
    expect(box3.getSize(e)).toEqual(v(0, 0, 0));
    expect(box3.getCenter(e)).toEqual(v(0, 0, 0));
    expect(box3.fromPoints([])).toEqual(e);
    expect(box3.expandByPoint(e, v(2, -1, 4))).toEqual({ min: v(2, -1, 4), max: v(2, -1, 4) });
    expect(box3.union(e, b)).toEqual(b);
  });
  test.each([
    ["a point on the face is inside", () => box3.containsPoint(b, v(1, 3, 1)), true],
    ["just outside one axis", () => box3.containsPoint(b, v(0, 5.0001, 1)), false],
    ["touching boxes intersect", () => box3.intersectsBox(b, { min: v(1, 5, 3), max: v(2, 6, 4) }), true],
    ["apart on z only", () => box3.intersectsBox(b, { min: v(0, 3, 3.01), max: v(1, 4, 4) }), false],
    ["containsBox", () => box3.containsBox(b, { min: v(-1, 2, 0), max: v(0, 3, 1) }), true],
    ["containsBox, sticking out", () => box3.containsBox(b, { min: v(-1, 2, 0), max: v(0, 3, 3.5) }), false],
    ["a sphere touching a face", () => box3.intersectsSphere(b, { center: v(2, 3, 1), radius: 1 }), true],
    ["a sphere off a corner", () => box3.intersectsSphere(b, { center: v(2, 6, 4), radius: 1.7 }), false],
    ["a sphere reaching a corner", () => box3.intersectsSphere(b, { center: v(2, 6, 4), radius: 1.75 }), true],
  ] as const)("%s", (_name, got, want) => expect(got()).toBe(want));
  test("applyMatrix4 boxes the turned corners", () => {
    const r = box3.applyMatrix4({ min: v(0, 0, 0), max: v(2, 1, 1) }, mat4.compose(v(0, 0, 5), axis(0, 0, 1, 90), v(1, 1, 1)));
    near(r.min as never, v(-1, 0, 5) as never, 1e-12);
    near(r.max as never, v(0, 2, 6) as never, 1e-12);
  });
});

describe("sphere", () => {
  const s = { center: v(1, 0, 0), radius: 2 };
  test("fromPoints centres on the points' box", () => {
    const f = sphere.fromPoints([v(0, 0, 0), v(4, 0, 0), v(1, 1, 0)]);
    expect(f.center).toEqual(v(2, 0.5, 0));
    near({ r: f.radius }, { r: Math.sqrt(4.25) });
  });
  test.each([
    ["containsPoint on the surface", () => sphere.containsPoint(s, v(1, 2, 0)), true],
    ["containsPoint outside", () => sphere.containsPoint(s, v(3, 0.1, 0)), false],
    ["touching spheres", () => sphere.intersectsSphere(s, { center: v(4, 0, 0), radius: 1 }), true],
    ["apart", () => sphere.intersectsSphere(s, { center: v(4.01, 0, 0), radius: 1 }), false],
    ["a box", () => sphere.intersectsBox(s, { min: v(3, -1, -1), max: v(4, 1, 1) }), true],
  ] as const)("%s", (_name, got, want) => expect(got()).toBe(want));
  test("distanceToPoint, getBoundingBox and applyMatrix4", () => {
    expect(sphere.distanceToPoint(s, v(1, 0, 5))).toBe(3);
    expect(sphere.distanceToPoint(s, v(1.5, 0, 0))).toBe(-1.5);
    expect(sphere.getBoundingBox(s)).toEqual({ min: v(-1, -2, -2), max: v(3, 2, 2) });
    const t = sphere.applyMatrix4(s, mat4.compose(v(0, 10, 0), q(0, 0, 0, 1), v(1, 3, 2)));
    expect(t).toEqual({ center: v(1, 10, 0), radius: 6 });
  });
});

describe("plane", () => {
  const p = plane.fromNormalAndCoplanarPoint(v(0, 2, 0), v(4, 5, -1));
  test("fromNormalAndCoplanarPoint normalises and sets the constant", () => {
    expect(p).toEqual({ normal: v(0, 1, 0), constant: -5 });
  });
  test("fromCoplanarPoints faces you when they are counter-clockwise", () => {
    const f = plane.fromCoplanarPoints(v(0, 0, 2), v(1, 0, 2), v(0, 1, 2));
    near(f.normal as never, v(0, 0, 1) as never);
    near({ c: f.constant }, { c: -2 });
    near(plane.fromCoplanarPoints(v(0, 0, 2), v(0, 1, 2), v(1, 0, 2)).normal as never, v(0, 0, -1) as never);
  });
  test.each([
    ["distanceToPoint above", () => plane.distanceToPoint(p, v(9, 7, 3)), 2],
    ["distanceToPoint below", () => plane.distanceToPoint(p, v(0, 1, 0)), -4],
    ["normalize scales the constant too", () => plane.normalize({ normal: v(0, 0, 4), constant: 8 }).constant, 2],
  ] as const)("%s", (_name, got, want) => expect(got()).toBe(want));
  test("projectPoint, intersectsBox and intersectsSphere", () => {
    expect(plane.projectPoint(p, v(3, 7, 1))).toEqual(v(3, 5, 1));
    expect(plane.intersectsBox(p, { min: v(0, 4, 0), max: v(1, 5, 1) })).toBe(true);
    expect(plane.intersectsBox(p, { min: v(0, 5.1, 0), max: v(1, 6, 1) })).toBe(false);
    expect(plane.intersectsSphere(p, { center: v(0, 6, 0), radius: 1 })).toBe(true);
    expect(plane.intersectsSphere(p, { center: v(0, 6, 0), radius: 0.9 })).toBe(false);
  });
});

describe("frustum", () => {
  // og.Camera's defaults (fov 50, near 0.1, far 2000), at z 10 looking down -z.
  const camera = {
    position: v(0, 0, 10),
    quaternion: q(0, 0, 0, 1),
    fov: 50,
    near: 0.1,
    far: 2000,
    zoom: 1,
    orthographic: false,
    height: 10,
  } as unknown as Camera;
  const f = frustum.fromCamera(camera, 2);
  // At 10 m the view is 10 tan 25 deg = 4.663 m high each side, twice that wide.
  const half = 10 * Math.tan((25 * Math.PI) / 180);
  test.each([
    ["the origin, straight ahead", v(0, 0, 0), true],
    ["behind the camera", v(0, 0, 11), false],
    ["closer than near", v(0, 0, 9.95), false],
    ["past far", v(0, 0, -1990.5), false],
    ["inside the top edge", v(0, half - 0.01, 0), true],
    ["above the top edge", v(0, half + 0.01, 0), false],
    ["inside the right edge (aspect 2)", v(2 * half - 0.01, 0, 0), true],
    ["right of it", v(2 * half + 0.01, 0, 0), false],
  ] as const)("containsPoint: %s", (_name, p, want) => expect(frustum.containsPoint(f, p)).toBe(want));
  test("planes in three.js's order with normals inwards: near (5) faces away from the camera", () => {
    expect(f.planes.length).toBe(6);
    near(f.planes[5]!.normal as never, v(0, 0, -1) as never, 1e-9);
    near({ d: plane.distanceToPoint(f.planes[5]!, v(0, 0, 9.9)) }, { d: 0 }, 1e-6);
    near(f.planes[4]!.normal as never, v(0, 0, 1) as never, 1e-9);
  });
  test("intersectsBox and intersectsSphere", () => {
    expect(frustum.intersectsBox(f, { min: v(2 * half - 0.5, -0.1, -0.1), max: v(30, 0.1, 0.1) })).toBe(true);
    expect(frustum.intersectsBox(f, { min: v(2 * half + 0.5, -0.1, -0.1), max: v(30, 0.1, 0.1) })).toBe(false);
    expect(frustum.intersectsBox(f, { min: v(-1, -1, 12), max: v(1, 1, 14) })).toBe(false);
    expect(frustum.intersectsSphere(f, { center: v(0, half + 0.5, 0), radius: 0.6 })).toBe(true);
    expect(frustum.intersectsSphere(f, { center: v(0, half + 1, 0), radius: 0.6 })).toBe(false);
  });
  test("an orthographic camera sees height by height times aspect, divided by zoom", () => {
    const o = frustum.fromCamera({ ...camera, orthographic: true, height: 10, zoom: 2 } as Camera, 2);
    expect(frustum.containsPoint(o, v(4.9, 2.4, -100))).toBe(true);
    expect(frustum.containsPoint(o, v(5.1, 0, 0))).toBe(false);
    expect(frustum.containsPoint(o, v(0, 2.6, 0))).toBe(false);
  });
  test("a turned camera: a quarter turn about y looks down -x", () => {
    const t = frustum.fromCamera({ ...camera, position: v(0, 0, 0), quaternion: axis(0, 1, 0, 90) } as Camera, 1);
    expect(frustum.containsPoint(t, v(-5, 0, 0))).toBe(true);
    expect(frustum.containsPoint(t, v(5, 0, 0))).toBe(false);
    expect(frustum.containsPoint(t, v(0, 0, -5))).toBe(false);
  });
});

describe("ray", () => {
  const r = { origin: v(0, 0, 0), direction: v(0, 0, 1) };
  const b = { min: v(-1, -1, 2), max: v(1, 1, 4) };
  test.each([
    ["at", () => ray.at(r, 2.5), v(0, 0, 2.5)],
    ["intersectBox enters the near face", () => ray.intersectBox(r, b), v(0, 0, 2)],
    ["intersectBox from inside answers where it leaves", () => ray.intersectBox({ origin: v(0, 0, 3), direction: v(0, 0, 1) }, b), v(0, 0, 4)],
    ["intersectBox along a diagonal", () => ray.intersectBox({ origin: v(-3, -3, 3), direction: vec3.normalize(v(1, 1, 0)) }, b), v(-1, -1, 3)],
    ["intersectSphere hits the near side", () => ray.intersectSphere(r, { center: v(0, 0, 5), radius: 1 }), v(0, 0, 4)],
    ["intersectSphere from inside answers where it leaves", () => ray.intersectSphere(r, { center: v(0, 0, 0.5), radius: 1 }), v(0, 0, 1.5)],
    ["intersectPlane", () => ray.intersectPlane(r, { normal: v(0, 0, 1), constant: -3 }), v(0, 0, 3)],
    ["intersectTriangle, front or back without culling", () => ray.intersectTriangle(r, v(-1, -1, 5), v(1, -1, 5), v(0, 1, 5)), v(0, 0, 5)],
    ["intersectTriangle facing the ray, culled", () => ray.intersectTriangle({ origin: v(0, 0, 10), direction: v(0, 0, -1) }, v(-1, -1, 5), v(1, -1, 5), v(0, 1, 5), true), v(0, 0, 5)],
  ] as const)("%s", (_name, got, want) => near(got() as never, want as never, 1e-12));
  test.each([
    ["a box beside the ray", () => ray.intersectBox(r, { min: v(1.5, -1, 2), max: v(2, 1, 4) })],
    ["a box behind", () => ray.intersectBox(r, { min: v(-1, -1, -4), max: v(1, 1, -2) })],
    ["a sphere it misses", () => ray.intersectSphere(r, { center: v(1.5, 0, 5), radius: 1 })],
    ["a sphere behind", () => ray.intersectSphere(r, { center: v(0, 0, -5), radius: 1 })],
    ["a parallel plane", () => ray.intersectPlane(r, { normal: v(1, 0, 0), constant: -3 })],
    ["a plane behind", () => ray.intersectPlane(r, { normal: v(0, 0, 1), constant: 3 })],
    ["a triangle facing away, culled", () => ray.intersectTriangle(r, v(-1, -1, 5), v(1, -1, 5), v(0, 1, 5), true)],
    ["a triangle beside the ray", () => ray.intersectTriangle(r, v(2, -1, 5), v(4, -1, 5), v(3, 1, 5))],
  ] as const)("misses %s: null", (_name, got) => expect(got()).toBeNull());
  test("a ray in its plane meets it at the origin; distanceToPoint", () => {
    expect(ray.intersectPlane(r, { normal: v(1, 0, 0), constant: 0 })).toEqual(v(0, 0, 0));
    expect(ray.distanceToPoint({ origin: v(0, 0, 0), direction: v(1, 0, 0) }, v(3, 4, 0))).toBe(4);
    expect(ray.distanceToPoint({ origin: v(0, 0, 0), direction: v(1, 0, 0) }, v(-3, 4, 0))).toBe(5);
  });
});

describe("color", () => {
  test.each([
    ["pure red", 0xff0000, { r: 1, g: 0, b: 0 }],
    ["sRGB 0x80 is linear 0.2158605", 0x808080, { r: 0.21586050010324417, g: 0.21586050010324417, b: 0.21586050010324417 }],
    ["the brand ink #14161b (og.Camera.background's default)", 0x14161b, { r: 0.0069954101845983935, g: 0.008023192982520563, b: 0.010960094003125918 }],
    ["a dark channel on the linear segment", 0x0a0000, { r: (10 / 255) * 0.0773993808, g: 0, b: 0 }],
  ] as const)("fromHex: %s", (_name, hex, want) => near(color.fromHex(hex) as never, want, 1e-12));
  test("toHex rounds back what fromHex made, and clamps HDR", () => {
    for (const hex of [0x000000, 0xffffff, 0x123456, 0xff8800, 0x14161b, 0x7f7f80, 0x010203]) expect(color.toHex(color.fromHex(hex))).toBe(hex);
    expect(color.toHex({ r: 2.5, g: -1, b: 0.21586050010324417 })).toBe(0xff0080);
  });
  test.each([
    ["red", [0, 1, 0.5], { r: 1, g: 0, b: 0 }],
    ["green at a third", [1 / 3, 1, 0.5], { r: 0, g: 1, b: 0 }],
    ["dark blue: sRGB 0.5 is linear 0.2140411", [2 / 3, 1, 0.25], { r: 0, g: 0, b: 0.2140411404715882 }],
    ["the hue wraps: 1.5 is cyan", [1.5, 1, 0.5], { r: 0, g: 1, b: 1 }],
    ["no saturation is grey", [0.7, 0, 0.5], { r: 0.2140411404715882, g: 0.2140411404715882, b: 0.2140411404715882 }],
    ["lightness clamps to white", [0.2, 1, 3], { r: 1, g: 1, b: 1 }],
  ] as const)("fromHsl: %s", (_name, [h, s, l], want) => near(color.fromHsl(h, s, l) as never, want, 1e-9));
  test("lerp in linear RGB", () => {
    near(color.lerp({ r: 0, g: 1, b: 0.5 }, { r: 1, g: 0, b: 0.5 }, 0.25) as never, { r: 0.25, g: 0.75, b: 0.5 });
  });
});
