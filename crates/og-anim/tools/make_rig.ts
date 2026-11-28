// CC0 walker fixture: blended boxes, two morphs, LINEAR/STEP/CUBICSPLINE
// channels and event markers. bun crates/og-anim/tools/make_rig.ts [out.glb]
import { Gltf, FACES, cross, bounds } from "../../../scripts/gltf.ts";
const OUT = process.argv[2] ?? "tests/fixtures/cook-assets/assets/models/walker.glb";
const NODES: [string, number | null, number[]][] = [
  ["Rig", null, [0, 0, 0]],
  ["hips", 0, [0, 1, 0]],
  ["spine", 1, [0, 0.3, 0]],
  ["head", 2, [0, 0.4, 0]],
  ["thigh.L", 1, [0.15, 0, 0]],
  ["shin.L", 4, [0, -0.45, 0]],
  ["foot.L", 5, [0, -0.45, 0]],
  ["thigh.R", 1, [-0.15, 0, 0]],
  ["shin.R", 7, [0, -0.45, 0]],
  ["foot.R", 8, [0, -0.45, 0]],
  ["arm.L", 2, [0.25, 0.3, 0]],
  ["forearm.L", 10, [0.3, 0, 0]],
  ["hand.L", 11, [0.3, 0, 0]],
];
function world(i: number): number[] {
  const [, p, t] = NODES[i]!;
  return p === null ? t : t.map((v, k) => v + world(p)[k]!);
}
const JOINTS = Array.from({ length: 12 }, (_, i) => i + 1),
  g = new Gltf();
const pos: number[] = [],
  nrm: number[] = [],
  jnt: number[] = [],
  wgt: number[] = [],
  idx: number[] = [],
  morph1: number[] = [],
  morph2: number[] = [];
function box(center: number[], half: number[], joint: number, blendTo?: number, blendBelow = 0, head = false): void {
  for (const [n, u, v] of FACES) {
    const base = pos.length / 3;
    for (const [su, sv] of [
      [-1, -1],
      [1, -1],
      [1, 1],
      [-1, 1],
    ]) {
      const p = center.map((c, k) => c + half[k]! * (n![k]! + su! * u![k]! + sv! * v![k]!));
      pos.push(...p);
      nrm.push(...n!);
      if (blendTo !== undefined && p[1]! < blendBelow) {
        jnt.push(joint, blendTo, 0, 0);
        wgt.push(0.5, 0.5, 0, 0);
      } else {
        jnt.push(joint, 0, 0, 0);
        wgt.push(1, 0, 0, 0);
      }
      morph1.push(0, head ? -0.5 * (p[1]! - center[1]!) : 0, 0);
      morph2.push(head ? 0.3 * (p[0]! - center[0]!) : 0, 0, head ? 0.3 * (p[2]! - center[2]!) : 0);
    }
    const [a, b, c, d] = [base, base + 1, base + 2, base + 3];
    idx.push(...(cross(u!, v!).reduce((s, x, k) => s + x * n![k]!, 0) > 0 ? [a!, b!, c!, a!, c!, d!] : [a!, c!, b!, a!, d!, c!]));
  }
}
function seg(node: number, child: number, r: number, blend = false): void {
  const a = world(node),
    b = world(child),
    c = a.map((x, k) => (x + b[k]!) / 2),
    h = a.map((x, k) => Math.max(Math.abs(b[k]! - x) / 2, r));
  box(c, h, node - 1, blend ? child - 1 : undefined, c[1]! - h[1]! * 0.5);
}
seg(1, 2, 0.12);
seg(2, 3, 0.1);
box(world(3), [0.12, 0.12, 0.12], 2, undefined, 0, true);
for (const [t, s, f] of [
  [4, 5, 6],
  [7, 8, 9],
]) {
  seg(t!, s!, 0.06, true);
  seg(s!, f!, 0.05);
}
seg(10, 11, 0.05);
seg(11, 12, 0.04);
const nv = pos.length / 3,
  aPos = g.add(pos, "f", nv, 5126, "VEC3", bounds(pos), 34962),
  aNrm = g.add(nrm, "f", nv, 5126, "VEC3", null, 34962),
  aJnt = g.add(jnt, "H", nv, 5123, "VEC4", null, 34962),
  aWgt = g.add(wgt, "f", nv, 5126, "VEC4", null, 34962),
  aIdx = g.add(idx, "I", idx.length, 5125, "SCALAR", null, 34963);
const aM1 = g.add(morph1, "f", nv, 5126, "VEC3", bounds(morph1)),
  aM2 = g.add(morph2, "f", nv, 5126, "VEC3", bounds(morph2));
const aIbm = g.add(
  JOINTS.flatMap((n) => {
    const [x, y, z] = world(n);
    return [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, -x!, -y!, -z!, 1];
  }),
  "f",
  JOINTS.length,
  5126,
  "MAT4",
);
const quat = (axis: number[], angle: number) => [...axis.map((x) => x * Math.sin(angle / 2)), Math.cos(angle / 2)],
  TAU = 2 * Math.PI;
const animations: any[] = [];
function channel(
  samplers: any[],
  channels: any[],
  node: number,
  path: string,
  times: number[],
  values: number[],
  interp: string,
  lanes: number,
): void {
  const ti = g.add(times, "f", times.length, 5126, "SCALAR", [[Math.min(...times)], [Math.max(...times)]]),
    vi = g.add(
      values,
      "f",
      [1, 3, 4].includes(lanes) ? values.length / lanes : values.length,
      5126,
      ({ 1: "SCALAR", 3: "VEC3", 4: "VEC4" } as Record<number, string>)[lanes] ?? "SCALAR",
    );
  samplers.push({ input: ti, output: vi, interpolation: interp });
  channels.push({ sampler: samplers.length - 1, target: { node, path } });
}
type Chan = [number, string, (t: number) => number[]];
function clip(name: string, duration: number, rate: number, interp: string, chans: Chan[], events?: [string, number][]): void {
  const samplers: any[] = [],
    channels: any[] = [],
    times = Array.from({ length: Math.round(duration * rate) + 1 }, (_, i) => i / rate);
  for (const [node, path, f] of chans) {
    const vals: number[] = [];
    for (const t of times) {
      const v = f(t);
      if (interp === "CUBICSPLINE") {
        const a = f(t + 0.001),
          b = f(t - 0.001),
          d = a.map((v, i) => (v - b[i]!) / 0.002);
        vals.push(...d, ...v, ...d);
      } else vals.push(...v);
    }
    channel(samplers, channels, node, path, times, vals, interp, f(0).length);
  }
  animations.push({
    name,
    samplers,
    channels,
    ...(events ? { extras: { "og.events": events.map(([name, time]) => ({ name, time })) } } : {}),
  });
}
function gait(
  name: string,
  duration: number,
  rate: number,
  interp: string,
  swing: number,
  stride: number,
  events: [string, number][],
): void {
  const ph = (t: number) => (t / duration) * TAU;
  clip(
    name,
    duration,
    rate,
    interp,
    [
      [1, "translation", (t) => [0, 1 + 0.03 * Math.cos(2 * ph(t)), (stride * t) / duration]],
      [1, "rotation", (t) => quat([0, 1, 0], 0.1 * Math.sin(ph(t)))],
      [4, "rotation", (t) => quat([1, 0, 0], swing * Math.sin(ph(t)))],
      [5, "rotation", (t) => quat([1, 0, 0], 0.6 * swing * (1 + Math.sin(ph(t) - 1.2)))],
      [7, "rotation", (t) => quat([1, 0, 0], -swing * Math.sin(ph(t)))],
      [8, "rotation", (t) => quat([1, 0, 0], 0.6 * swing * (1 - Math.sin(ph(t) - 1.2)))],
    ],
    events,
  );
}
clip("idle", 2, 30, "LINEAR", [
  [2, "rotation", (t) => quat([0, 0, 1], 0.05 * Math.sin((t * TAU) / 2))],
  [1, "translation", (t) => [0, 1 - 0.01 * Math.sin((t * TAU) / 2), 0]],
]);
gait("walk", 1, 24, "LINEAR", 0.5, 1.2, [
  ["footL", 0.26],
  ["footR", 0.76],
]);
gait("run", 0.6, 10, "CUBICSPLINE", 0.9, 1.5, [
  ["footL", 0.16],
  ["footR", 0.46],
]);
clip("wave", 1, 4, "STEP", [[10, "rotation", (t) => quat([0, 0, 1], 1.2 * Math.sin(t * TAU))]], [["wave", 0.5]]);
const samplers: any[] = [],
  channels: any[] = [];
channel(samplers, channels, 1, "rotation", [0, 1], [...quat([0, 1, 0], 0), ...quat([0, 1, 0], (2 * Math.PI) / 3)], "LINEAR", 4);
animations.push({ name: "turn", samplers, channels });
clip("blink", 0.5, 30, "LINEAR", [[13, "weights", (t) => [Math.sin((t / 0.5) * Math.PI), (0.5 * t) / 0.5]]]);
const nodes: any[] = NODES.map(([name, , t], i) => {
  const children = NODES.flatMap(([, p], j) => (p === i ? [j] : []));
  if (i === 0) children.push(NODES.length);
  return { name, ...(t.some(Boolean) ? { translation: t } : {}), ...(children.length ? { children } : {}) };
});
nodes.push({ name: "body", mesh: 0, skin: 0 });
const size = g.write(OUT, {
  asset: { version: "2.0", generator: "og-anim make_rig.ts", copyright: "CC0 1.0, Origin Engine authors" },
  scene: 0,
  scenes: [{ name: "walker", nodes: [0] }],
  nodes,
  meshes: [
    {
      name: "walker",
      primitives: [
        {
          attributes: { POSITION: aPos, NORMAL: aNrm, JOINTS_0: aJnt, WEIGHTS_0: aWgt },
          indices: aIdx,
          material: 0,
          targets: [{ POSITION: aM1 }, { POSITION: aM2 }],
        },
      ],
      weights: [0, 0],
      extras: { targetNames: ["blink", "squash"] },
    },
  ],
  materials: [{ name: "walker", pbrMetallicRoughness: { baseColorFactor: [0.8, 0.45, 0.2, 1], metallicFactor: 0, roughnessFactor: 0.7 } }],
  skins: [{ name: "walker", joints: JOINTS, inverseBindMatrices: aIbm, skeleton: 1 }],
  animations,
});
console.log(`${OUT}: ${size} bytes, ${nv} vertices, ${idx.length / 3} triangles, ${animations.length} clips`);
