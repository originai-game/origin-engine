// Mass element timing on the native host (og.ui.defineMass). The world
// tags, ground marks and mass components are component documents now
// (src/ui/components), cooked and built by og-ui's component document test.
import { expect, test } from "bun:test";
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { engineFor } from "./support/ui.ts";

loadNativeHost();

const DESKTOP = { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" };
const engine = (name: string, build: (w: World) => void) => engineFor(name, build, DESKTOP, [og.Camera]);

test("mass timing accepts shared curves and refuses removed or invalid names", () => {
  const e = engine("MassTiming", () => {
    for (const ease of ["cubic-bezier(0.17,0.85,0.63,0.08)", "steps(4,start)", "power1.out"] as const) {
      expect(og.ui.defineMass(ease, { kind: "particle", keys: [{ t: 1, ease }] }).ok).toBe(true);
    }
    for (const ease of ["out", "step", "steps(0)", "cubic-bezier(2,0,0,1)"]) {
      // Runtime validation also protects untyped callers.
      const result = og.ui.defineMass(ease, JSON.parse(JSON.stringify({ keys: [{ t: 1, ease }] })));
      expect(result.ok).toBe(false);
      if (!result.ok) expect(result.reason).toContain(ease);
    }
  });
  e.advance(1);
  e.destroy();
});

// The world labels as a game uses them: each member mounted by name from a
// game page in every fixture its document declares, merged over its
// default, in the default style on a desktop and a phone at 100 % and
// 150 % text. Ways it could fail: a fixture state that no longer mounts, a
// token the style lacks, a part declared but never drawn, or a boss or
// casting plate whose priority drops under the prompt's (the prompt would
// then be laid over it).
import { cpSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterAll, beforeAll, setDefaultTimeout } from "bun:test";
import { native } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[], o: { cwd: string; env: Record<string, string | undefined>; stdout: "ignore"; stderr: "pipe" }): { exitCode: number; stderr: Uint8Array } };
setDefaultTimeout(60_000);

const GROUP = join(dirname(dirname(fileURLToPath(import.meta.url))), "src/ui/components/world");
const LABELS = ["nameplate", "overhead-bars", "loot-label", "interaction-prompt", "speech-bubble", "telegraph", "selection-ring", "placement-ghost", "waypoint", "path-line"];
const MEMBERS: Record<string, { component: string; parts: string[]; markup: string; fixtures: Record<string, Record<string, unknown>> }> = {};
let root = "";

beforeAll(() => {
  root = mkdtempSync(join(tmpdir(), "og-world-kit-"));
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "World kit"\nid "world-kit"\nentry "main.ts"\nprofile "3d"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  for (const name of LABELS) {
    const text = readFileSync(join(GROUP, `${name}.svelte`), "utf8");
    const component = /<meta name="component" content="([^"]+)">/.exec(text)![1]!;
    const parts = /<meta name="parts" content="([^"]+)">/.exec(text)![1]!.split(" ");
    const fixtures = Object.fromEntries([...text.matchAll(/<meta name="fixture:([^"]+)" content='([^']*)'>/g)].map((m) => [m[1]!, JSON.parse(m[2]!)]));
    MEMBERS[name] = { component, parts, markup: text.split("<style>")[0]!, fixtures };
    const props = Object.keys(fixtures.default!).map((k) => `${k}={$${k}}`).join(" ");
    writeFileSync(join(root, `assets/ui/host-${name}.svelte`), `<meta name="screen" content="Host_${component}" data-layer="hud">\n<div style="width: stretch; height: stretch"><${component} ${props} /></div>\n`);
  }
  const cook = Bun.spawnSync([ogBin(), "cook", root], { cwd: root, env: process.env, stdout: "ignore", stderr: "pipe" });
  if (cook.exitCode !== 0) throw new Error(`cooking ${root}: ${new TextDecoder().decode(cook.stderr)}`);
});
afterAll(() => {
  if (root) rmSync(root, { recursive: true, force: true });
});

const PHONE = { width: 390, height: 844, dpr: 2, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" };
const WIDE = { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" };

function mount(name: string, stores: Record<string, unknown>, view: object, textScale: number): Engine {
  const { component } = MEMBERS[name]!;
  const r = createEngine(og.game({
    start(world) {
      og.ui.configure(world, { style: null, textScale });
      const loaded = og.load.document(world, `ui/host-${name}.svelte`);
      if (!loaded.ok) throw new Error(loaded.reason);
      const cam = world.resource(og.Camera);
      if (cam.ok) { cam.value.position = { x: 0, y: 3, z: 8 }; cam.value.quaternion = og.transform.lookAt(cam.value.position, { x: 0, y: 1, z: 0 }); }
    },
    ui(world) {
      const scope = og.ui.scope(world, "t");
      const inputs = Object.fromEntries(Object.entries(stores).map(([k, v]) => [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
      const mounted = scope.mount("page", `Host_${component}`, inputs);
      if (!mounted.ok) throw new Error(mounted.reason);
    },
  }) as unknown as GameToken, { root });
  if (!r.ok) throw new Error(r.reason);
  og.ui.previewViewport(r.value.world, view as never);
  return r.value;
}

for (const [view, viewport] of [["desktop", WIDE], ["phone", PHONE]] as const) for (const textScale of [1, 1.5]) {
  test(`the world labels mount from every fixture on a ${view} at ${textScale * 100} % in the default style without errors`, () => {
    for (const name of LABELS) {
      for (const fixture of Object.keys(MEMBERS[name]!.fixtures)) {
        const e = mount(name, { ...MEMBERS[name]!.fixtures.default, ...MEMBERS[name]!.fixtures[fixture] }, viewport, textScale);
        try {
          e.advance(3);
          const errors = og.ui.scope(e.world, "t").takeErrors();
          expect(`${name}/${fixture}: ${errors.join("; ")}`).toBe(`${name}/${fixture}: `);
          const warnings = ((JSON.parse(native().og_node_ui_frame((e as unknown as { id: number }).id)) as { warnings?: string[] }).warnings ?? []).filter((w) => w.includes("computed-value"));
          expect(`${name}/${fixture}: ${warnings.join("; ")}`).toBe(`${name}/${fixture}: `);
        } finally {
          e.destroy();
        }
      }
    }
  }, 300_000);
}

test("every world label names only its own parts, each drawn by its markup", () => {
  for (const [name, m] of Object.entries(MEMBERS)) {
    const named = new Set([...m.markup.matchAll(/part="([^"]+)"/g)].flatMap((x) => x[1]!.split(/\s+/)));
    for (const p of m.parts) expect(`${name} ${p} ${p.startsWith(`${m.component}.`) && named.has(p)}`).toBe(`${name} ${p} true`);
    for (const used of named) expect(m.parts).toContain(used);
  }
});

test("a boss or casting nameplate outranks the interaction prompt, a target outranks both", () => {
  const prompt = Number(/priority="(\d+)"/.exec(MEMBERS["interaction-prompt"]!.markup)![1]);
  const plate = /priority=\{targeted \? (\d+) : \(boss \|\| castName != '' \? (\d+) : (\d+)\)\}/.exec(MEMBERS.nameplate!.markup)!;
  expect(prompt).toBe(300);
  expect(Number(plate[1])).toBe(1100);
  expect(Number(plate[2])).toBeGreaterThan(prompt);
  expect(Number(plate[3])).toBeLessThan(prompt);
});
