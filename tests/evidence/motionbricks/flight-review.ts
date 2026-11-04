// Diagnostic replay of exported FK, not runtime integration of the operator.
import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";

type Point = [number, number, number];
type Mode = "raw" | "aligned" | "corrected";
interface Case {
  group: string[];
  rig: string;
  frames: number;
  review: {
    nodes: { name: string; parent: number }[];
    raw: Point[][];
    aligned: Point[][];
    corrected: Point[][];
    contacts: { begin: number; end: number; anchor: Point }[];
  };
}
const cases: Case[] = /* EXPORTED_CASES */ [];
let selected = 0, sample = 0, playing = false;
let modes: Mode[] = ["raw", "aligned", "corrected"];
const colors = {
  raw: { r: 0.4, g: 0.6, b: 1 },
  aligned: { r: 1, g: 0.6, b: 0.2 },
  corrected: { r: 0.3, g: 1, b: 0.55 },
};
function camera(world: World): void {
  const c = cases[selected]!;
  const points = modes.flatMap(mode => c.review[mode].flat());
  const xs = points.map(p => p[0]), zs = points.map(p => p[2]);
  const mid = { x: (Math.min(...xs) + Math.max(...xs)) / 2, y: 1, z: (Math.min(...zs) + Math.max(...zs)) / 2 };
  const cam = world.resource(og.Camera);
  if (!cam.ok) throw new Error(cam.reason);
  cam.value.position = { x: mid.x + 1, y: 2.8, z: mid.z + 9 };
  cam.value.quaternion = og.transform.lookAt(cam.value.position, mid);
  cam.value.orthographic = true;
  cam.value.height = modes.length === 1 ? 3.4 : 7.2;
  cam.value.background = { r: 0.025, g: 0.03, b: 0.04 };
}
const Draw = defineSystem("Flight review", {}, (_q, _time, world) => {
  const c = cases[selected]!;
  og.debug.clear(world);
  for (let lane = 0; lane < modes.length; lane++) {
    const mode = modes[lane]!;
    const offset = (lane - (modes.length - 1) / 2) * 4;
    const at = (p: Point) => ({ x: p[0] + offset, y: p[1], z: p[2] });
    const pose = c.review[mode][sample]!;
    const desc = { color: colors[mode], width: 3 };
    for (let j = 0; j < pose.length; j++) {
      const parent = c.review.nodes[j]!.parent;
      if (parent < pose.length) og.debug.line(world, at(pose[parent]!), at(pose[j]!), desc);
    }
    for (const contact of c.review.contacts) {
      og.debug.sphere(world, at(contact.anchor), 0.025, {
        color: sample >= contact.begin && sample < contact.end ? { r: 1, g: 0.2, b: 0.2 } : { r: 0.5, g: 0.5, b: 0.5 },
      });
    }
    for (let k = -3; k <= 3; k++) {
      og.debug.line(world, at([k, 0, -3]), at([k, 0, 3]), { color: { r: 0.12, g: 0.12, b: 0.12 } });
      og.debug.line(world, at([-3, 0, k]), at([3, 0, k]), { color: { r: 0.12, g: 0.12, b: 0.12 } });
    }
    og.debug.text(world, { x: offset, y: 2.5, z: 0 }, `${c.group.join("/")} ${c.rig} ${mode} s${sample}`, { color: colors[mode], width: 16 });
  }
  if (playing) sample = Math.min(sample + 1, c.frames - 1);
});
export default og.game({
  tick: 30, seed: 1, systems: [Draw],
  start(world) {
    const font = og.load.font(world, "fonts/ui.ttf");
    if (!font.ok) throw new Error(font.reason);
    og.ui.configure(world, { font: font.value });
    camera(world);
  },
  commands: {
    case: (world, args) => {
      const index = Number(args[0]);
      if (!Number.isInteger(index) || !cases[index]) throw new Error("case:<index>");
      selected = index; sample = 0; playing = false; camera(world);
    },
    sample: (_world, args) => {
      const index = Number(args[0]);
      if (!Number.isInteger(index) || index < 0 || index >= cases[selected]!.frames) throw new Error("sample:<index>");
      sample = index; playing = false;
    },
    view: (world, args) => {
      const mode = args[0];
      if (mode !== "all" && mode !== "raw" && mode !== "aligned" && mode !== "corrected") throw new Error("view:all|raw|aligned|corrected");
      modes = mode === "all" ? ["raw", "aligned", "corrected"] : [mode]; camera(world);
    },
    play: () => { sample = 0; playing = true; },
  },
});
