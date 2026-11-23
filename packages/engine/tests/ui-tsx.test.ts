// The TSX path's contracts end to end (crates/og-ui-next/CONVENTIONS.md §JSX
// compile, §Ops): nsh-jsx compiles a component to the renderer's calls, the
// renderer writes the OGOP batch the tree applies, a changed signal sends only
// what changed, and an OGEV reply calls the node's handler with the world.

import { expect, test } from "bun:test";
import { defineSystem, og, type World } from "@origingame/engine";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { bundleGame, compileJsx } from "../src/node/build.ts";
import { cookProject } from "./support/og-bin.ts";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";
import { Text, View, mount, renderers, t, useStore } from "../src/ui/index.ts";
import { Renderer, strings } from "../src/ui/rt.ts";
import { createSignal } from "../src/ui/signals.ts";

const SOURCE = `import { View, Text, Show, Switch, Match, packClass } from "@origingame/engine/ui";
export function Hud(props: { hp: number; on: boolean; press: (e: unknown) => void }) {
  return (
    <View class="p-4 bg-surface" onPress={props.press}>
      <Text class="text-fg">HP {props.hp}</Text>
      <Show when={props.on}><Text>on</Text></Show>
      <Switch fallback={<Text>low</Text>}>
        <Match when={props.hp > 3}><Text class={packClass("Hud", "hp", "high")}>high</Text></Match>
      </Switch>
    </View>
  );
}`;

type Op = (string | number)[];

/** The ops of an OGOP batch, read independently of the renderer. */
function decode(bytes: Uint8Array): Op[] {
  const v = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  expect(new TextDecoder().decode(bytes.subarray(0, 4))).toBe("OGOP");
  let at = 4;
  const u8 = () => v.getUint8(at++);
  const u32 = () => { const n = v.getUint32(at, true); at += 4; return n; };
  const str = () => { const n = u32(); const s = new TextDecoder().decode(bytes.subarray(at, at + n)); at += n; return s; };
  const names = ["", "create", "createText", "setText", "setClass", "setStyle", "setProp", "listen", "insert", "remove", "release", "placeholder", "strings"];
  const ops: Op[] = [];
  for (let i = u32(); i > 0; i--) {
    const code = u8();
    const name = names[code]!;
    if (code === 1) ops.push([name, u32(), u8()]);
    else if (code === 2 || code === 3 || code === 4 || code === 5) ops.push([name, u32(), str()]);
    else if (code === 6) ops.push([name, u32(), str(), str()]);
    else if (code === 7) ops.push([name, u32(), u8(), u8()]);
    else if (code === 8) { const p = u32(), c = u32(), b = u32(); ops.push([name, p, c, b === 0xffffffff ? -1 : b]); }
    else if (code === 9) ops.push([name, u32(), u32()]);
    else if (code === 12) ops.push([name, str()]);
    else ops.push([name, u32()]);
  }
  expect(at).toBe(bytes.length);
  return ops;
}

function reply(events: [id: number, code: number][]): Uint8Array {
  const out = new Uint8Array(8 + events.length * 21);
  const v = new DataView(out.buffer);
  out.set([0x4f, 0x47, 0x45, 0x56]);
  v.setUint32(4, events.length, true);
  events.forEach(([id, code], i) => {
    const at = 8 + i * 21;
    v.setUint32(at, id, true);
    v.setUint8(at + 4, code);
    v.setFloat32(at + 5, 12, true);
    v.setFloat32(at + 9, 34, true);
  });
  return out;
}

test("a TSX component's ops, its updates and its events", async () => {
  const compiled = compileJsx("Hud.tsx", SOURCE);
  if (!compiled.ok) throw new Error(compiled.reason);
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-"));
  try {
    const rt = fileURLToPath(new URL("../src/ui/rt.ts", import.meta.url));
    const ui = fileURLToPath(new URL("../src/ui/index.ts", import.meta.url));
    const code = compiled.code.replace('"@origingame/engine/ui/rt"', JSON.stringify(rt)).replace('"@origingame/engine/ui"', JSON.stringify(ui));
    writeFileSync(join(dir, "Hud.js"), code);
    const { Hud } = await import(join(dir, "Hud.js"));

    const batches: Op[][] = [];
    let next: Uint8Array = reply([]);
    // The runtime's worlds carry their id; the public World does not name it.
    const world = { id: 9001 } as unknown as World;
    renderers.set(9001, new Renderer((bytes) => { batches.push(decode(bytes)); const r = next; next = reply([]); return r; }, world));
    const hp = createSignal(3), on = createSignal(false);
    const pressed: unknown[] = [];
    mount(world, Hud, { get hp() { return hp(); }, get on() { return on(); }, press: (e: unknown) => pressed.push(e) }, { layer: "hud" });

    // The mount's one batch builds the whole tree and puts its root in 0. A
    // string table another test's component registered in this process goes
    // first, alone, so t() reads it as the tree builds.
    while (batches[0]?.every((op) => op[0] === "strings")) batches.shift();
    expect(batches).toHaveLength(1);
    const first = batches[0]!;
    expect(first).toContainEqual(["create", 2, 0]);
    expect(first).toContainEqual(["setClass", 2, "p-4 bg-surface"]);
    expect(first).toContainEqual(["listen", 2, 0, 1]);
    expect(first).toContainEqual(["createText", 4, "HP "]);
    expect(first).toContainEqual(["createText", 5, "3"]);
    expect(first.at(-1)).toEqual(["insert", 0, 1, -1]);
    // The mount is one grid cell over the surface: a component's root is stretched to it.
    expect(first).toContainEqual(["setClass", 1, "absolute inset-0 grid"]);
    expect(first.some((op) => op[0] === "createText" && op[2] === "on")).toBe(false);
    expect(first.filter((op) => op[0] === "createText" && (op[2] === "low" || op[2] === "high")).map((op) => op[2])).toEqual(["low"]);

    // A changed signal sends only its text; the branch shows when `on` does.
    hp.set(4);
    on.set(true);
    renderers.get(9001)!.flush();
    const second = batches[1]!;
    expect(second).toContainEqual(["setText", 5, "4"]);
    // The Switch's case changes with hp: "high" is made, with its pack token, and "low" goes.
    expect(second.filter((op) => op[0] === "createText").map((op) => op[2]).sort()).toEqual(["high", "on"]);
    expect(second).toContainEqual(["setClass", expect.any(Number), "@Hud.hp.high"]);
    const made = (text: string) => second.find((op) => op[0] === "createText" && op[2] === text)!;
    const holder = (text: string) => second.find((op) => op[0] === "insert" && op[2] === made(text)[1])![1];
    expect(second).toContainEqual(["insert", 2, holder("high"), -1]);
    expect(second).toContainEqual(["insert", 2, holder("on"), -1]);
    expect(second.filter((op) => op[0] === "remove" && op[1] === 2)).toHaveLength(1);
    expect(second.filter((op) => op[0] === "release")).toHaveLength(1);
    expect(second.some((op) => op[0] === "create" && op[2] === 0)).toBe(false);

    // A press on the view (id 2) calls its handler with the world.
    next = reply([[2, 0]]);
    renderers.get(9001)!.flush();
    expect(pressed).toEqual([expect.objectContaining({ world, type: "press", x: 12, y: 34 })]);
  } finally {
    rmSync(dir, { recursive: true, force: true });
    renderers.delete(9001);
  }
});

test("a spread's children go in its element, and bare text in a View is refused", async () => {
  const source = `import { View, Text } from "@origingame/engine/ui";
export function Plate(props: { children?: unknown; class?: string }) { return <View {...props} role="button" />; }
export function Own(props: { children?: unknown }) { return <View {...props}><Text>own</Text></View>; }
export function Labelled() { return <Plate class="p-2"><Text>Go</Text></Plate>; }
export function Shadowed() { return <Own><Text>lost</Text></Own>; }
export function Bare(props: { label: string }) { return <View>{props.label}</View>; }`;
  const compiled = compileJsx("Plate.tsx", source);
  if (!compiled.ok) throw new Error(compiled.reason);
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-spread-"));
  try {
    const rt = fileURLToPath(new URL("../src/ui/rt.ts", import.meta.url));
    const ui = fileURLToPath(new URL("../src/ui/index.ts", import.meta.url));
    writeFileSync(join(dir, "Plate.js"), compiled.code.replace('"@origingame/engine/ui/rt"', JSON.stringify(rt)).replace('"@origingame/engine/ui"', JSON.stringify(ui)));
    const { Labelled, Shadowed, Bare } = await import(join(dir, "Plate.js"));
    const world = { id: 9002 } as unknown as World;
    const texts = (C: unknown, props: object = {}) => {
      const batches: Op[][] = [];
      renderers.set(9002, new Renderer((bytes) => { batches.push(decode(bytes)); return reply([]); }, world));
      try {
        mount(world, C as never, props, { layer: "hud" });
        return batches.flat().filter((op) => op[0] === "createText").map((op) => op[2]);
      } finally {
        renderers.delete(9002);
      }
    };
    // A wrapper spreading its props into a View still draws what it was given.
    expect(texts(Labelled)).toEqual(["Go"]);
    // The element's own children win over the spread's.
    expect(texts(Shadowed)).toEqual(["own"]);
    // The function form (View, Text called directly) puts its children in once.
    expect(texts(() => View({ children: [Text({ children: "Go" })] }))).toEqual(["Go"]);
    // Dynamic text has no <Text>: an error naming the text, never a wrap.
    expect(() => texts(Bare, { label: "Continue" })).toThrow('og.ui: the text "Continue" is a bare child of <view>: put it in a <Text>');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("a native bundle compiles the engine's TSX components with the game's", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-native-"));
  try {
    writeFileSync(join(dir, "project.ogt"), "type project\nversion 1\n---\n");
    writeFileSync(join(dir, "main.ts"), `export { Celebration, Spinner } from "@origingame/engine/ui/components";\n`);
    const out = join(dir, "game.js");
    // As og run builds it too: with a cache plan, whose catch-all resolver
    // once dropped the table's import, and its strings() call, as unused.
    for (const plan of [undefined, new Set([join(dir, "main.ts")])]) {
      const b = await bundleGame(join(dir, "main.ts"), out, "native", false, {}, plan);
      if (!b.ok) throw new Error(b.reason);
      const code = readFileSync(out, "utf8");
      // Loaded as an external module, Bun's own JSX transform would have kept
      // the compile-time Show/For imports, which the engine does not export.
      expect(code).not.toContain("@origingame/engine/ui/components");
      expect(code).toMatch(/from ["'][^"']*ui\/rt(\.ts)?["']/);
      // The component's `import "./spinner.en.ogt"` records the table's bundle id.
      expect(code).toMatch(/strings\d*\("@ui\/components\/base\/spinner\.en\.ogt"\)/);
      // An emitter's import (Celebration's `import sparks from "./fx/sparks.ogt"`)
      // is its bundle id for og.load.emitter, and registers no string table.
      expect(code).toContain(`"@ui/components/hud/fx/sparks.ogt"`);
      expect(code).not.toMatch(/strings\d*\("@ui\/components\/hud\/fx\//);
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

// A display frame flushes the TSX UI at each tick boundary, as an advance
// does: what a system wrote reaches the tree while the game plays, not only
// at the mount (the platform pump that flushes it ran in advance alone).
test("a display frame sends the UI's changes at each tick boundary", () => {
  loadNativeHost();
  const Count = defineSystem("UiCount", { reads: [] }, (_q, _time, world) => {
    const n = og.ui.scope(world, "count").writable("n", 0);
    n.update((v) => v + 1);
  });
  const game = og.game({
    tick: 60,
    systems: [Count],
    ui(world) {
      const n = useStore<number>(og.ui.scope(world, "count").writable("n", 0));
      // A box only while the count is 10: in that tick's paint.
      mount(world, () => View({ children: [Text({ children: () => String(n()) }), () => (n() === 10 ? View({ class: "w-4 h-4 bg-surface" }) : null)] }), {});
    },
  });
  const created = createEngine(game as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  const id = (engine.world as unknown as { id: number }).id;
  const ui = renderers.get(id);
  if (!ui) throw new Error("the mount made no renderer");
  let flushes = 0;
  const flush = ui.flush.bind(ui);
  ui.flush = () => { flushes++; flush(); };
  const before = ui.clock().tick;
  let ticks = 0;
  for (let i = 0; i < 6; i++) ticks += engine.frame(1000 / 60).ticks ?? 0;
  expect(ticks).toBeGreaterThan(2);
  expect(flushes).toBeGreaterThanOrEqual(ticks);
  // The UI's clock is the world's: it moves by the ticks run, at the tick's length.
  expect(ui.clock().tick - before).toBe(ticks);
  expect(ui.clock().dt).toBeCloseTo(1 / 60, 6);
  // og.ui.end lays the tree out inside the tick, after the game's systems:
  // what a system wrote in tick t is in tick t's paint, not tick t+1's.
  // The tenth write shows the box in the tenth tick's paint.
  expect(ticks).toBeLessThan(10);
  const quads = () => (engine.probe.dump({ what: "ui" }) as unknown as { next: { quads: number } }).next.quads;
  engine.advance(9 - ticks);
  expect(quads()).toBe(0);
  engine.advance(1);
  expect(quads()).toBeGreaterThan(0);
  renderers.delete(id);
});

// FlowChoices (the 底牌 playtest, flow guide §4) takes og.flow.view's value and
// derives its rows itself. Ways this could fail: a pressed row or hotspot
// reporting its place in the list instead of ChoiceView.index (choice 3 is
// listed before 1 here); a disabled choice, or any after the lock, still
// taking presses; the pending choice not marked; the screen showing with no
// window.
test("FlowChoices shows a flow's window from its view and reports the ChoiceView.index pressed", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-flow-"));
  try {
    writeFileSync(join(dir, "project.ogt"), "type project\nversion 1\n---\n");
    writeFileSync(join(dir, "main.ts"), `export { FlowChoices } from "@origingame/engine/ui/components";\n`);
    const out = join(dir, "game.js");
    const b = await bundleGame(join(dir, "main.ts"), out, "native");
    if (!b.ok) throw new Error(b.reason);
    const rt = fileURLToPath(new URL("../src/ui/rt.ts", import.meta.url));
    const ui = fileURLToPath(new URL("../src/ui/index.ts", import.meta.url));
    writeFileSync(out, readFileSync(out, "utf8").replace(/(["'])[^"']*\/ui\/rt(\.ts)?\1/g, JSON.stringify(rt)).replace(/(["'])@origingame\/engine\/ui\1/g, JSON.stringify(ui)));
    const { FlowChoices } = await import(out);
    const { loadNativeHost } = await import("../src/node/index.ts");
    loadNativeHost();

    const batches: Op[][] = [];
    let next: Uint8Array = reply([]);
    const world = { id: 9003 } as unknown as World;
    renderers.set(9003, new Renderer((bytes) => { batches.push(decode(bytes)); const r = next; next = reply([]); return r; }, world));
    const choice = (index: number, text: string, enabled = true) => ({ index, text, enabled });
    const view = (window: object | null) => ({
      choices: window ? [choice(0, "Stay"), choice(3, "Run"), choice(1, "Lie", false)] : [],
      window,
      hotspots: window ? [{ choice: 3, id: "door", x: 0.5, y: 0.2, width: 0.1, height: 0.3 }] : [],
      qtes: [],
    });
    const v = createSignal<unknown>(view(null));
    const chosen: number[] = [];
    mount(world, FlowChoices, { get view() { return v(); }, onChoose: (_e: unknown, index: number) => chosen.push(index) });
    const ops = () => batches.flat();
    expect(ops().some((op) => op[0] === "createText")).toBe(false);

    v.set(view({ ticksLeft: 60, ticksTotal: 120, defaultChoice: 0, pending: 3, locked: false }));
    renderers.get(9003)!.flush();
    const texts = ops().filter((op) => op[0] === "createText").map((op) => op[2]);
    for (const t of ["Stay", "Run", "Lie"]) expect(texts).toContain(t);
    // The row holding a text: the pressable view around it.
    const parent = (id: number) => ops().find((op) => op[0] === "insert" && op[2] === id)![1] as number;
    const pressable = new Set(ops().filter((op) => op[0] === "listen" && op[2] === 0 && op[3] === 1).map((op) => op[1] as number));
    const row = (text: string) => {
      let id = ops().find((op) => op[0] === "createText" && op[2] === text)![1] as number;
      while (!pressable.has(id)) id = parent(id);
      return id;
    };
    expect(ops()).toContainEqual(["setProp", row("Run"), "data-chosen", "true"]);
    const hotspot = [...pressable].find((id) => ![row("Stay"), row("Run"), row("Lie")].includes(id))!;

    next = reply([[row("Stay"), 0], [row("Run"), 0], [row("Lie"), 0], [hotspot, 0]]);
    renderers.get(9003)!.flush();
    renderers.get(9003)!.flush();
    expect(chosen).toEqual([0, 3, 3]);

    v.set(view({ ticksLeft: 0, ticksTotal: 120, defaultChoice: 0, pending: 3, locked: true }));
    renderers.get(9003)!.flush();
    next = reply([[row("Stay"), 0], [hotspot, 0]]);
    renderers.get(9003)!.flush();
    renderers.get(9003)!.flush();
    expect(chosen).toEqual([0, 3, 3]);
  } finally {
    rmSync(dir, { recursive: true, force: true });
    renderers.delete(9003);
  }
});

// A long press and a hold listen on the tree's codes 11 and 12 (ops.rs
// Event), after onAction's 10; `hold` and `autofocus` reach it as props, and
// a hold's event calls its handler.
test("onLongPress and onHold listen on the tree's codes, with hold and autofocus", () => {
  const batches: Op[][] = [];
  let next: Uint8Array = reply([]);
  const world = { id: 9002 } as unknown as World;
  renderers.set(9002, new Renderer((bytes) => { batches.push(decode(bytes)); const r = next; next = reply([]); return r; }, world));
  try {
    const held: string[] = [];
    mount(world, () => View({ hold: 0.5, autofocus: true, onLongPress: () => held.push("long"), onHold: (e) => held.push(e.type) }), {});
    const ops = batches.flat();
    expect(ops).toContainEqual(["listen", 2, 11, 1]);
    expect(ops).toContainEqual(["listen", 2, 12, 1]);
    expect(ops).toContainEqual(["setProp", 2, "hold", "0.5"]);
    expect(ops).toContainEqual(["setProp", 2, "autofocus", "true"]);
    next = reply([[2, 12], [2, 11]]);
    renderers.get(9002)!.flush();
    expect(held).toEqual(["hold", "long"]);
  } finally {
    renderers.delete(9002);
  }
});

// A built-in component's string table, cooked into a game that uses the
// built-in components and recorded by its import, reaches the world before
// the component's t() reads it: without it the text is the raw key.
test("a component's imported string table is registered before it is built", () => {
  loadNativeHost();
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-strings-"));
  try {
    writeFileSync(join(dir, "project.ogt"), "type project\nversion 1\n---\nname \"p\"\n");
    mkdirSync(join(dir, "src"));
    mkdirSync(join(dir, "assets"));
    // A module imported by its path bundles its tables as the barrel does.
    writeFileSync(join(dir, "src/main.ts"), `export { ErrorState } from "@origingame/engine/ui/components/base/error-state.tsx";\n`);
    cookProject(dir, dir);
    const shown: string[] = [];
    const game = og.game({
      ui(world) {
        // What the build makes of the component's `import "./states.en.ogt"`.
        strings("@ui/components/base/states.en.ogt");
        mount(world, () => { shown.push(t("base.error.retry")); return Text({ children: shown[0] }); }, {});
      },
    });
    const created = createEngine(game as unknown as GameToken, { root: dir });
    if (!created.ok) throw new Error(created.reason);
    const engine = created.value;
    engine.frame(1000 / 60);
    expect(shown).toEqual(["Try again"]);
    renderers.delete((engine.world as unknown as { id: number }).id);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

// A node with `anchor` sits where the tick's camera sees its entity, lifted
// by `lift`, and one whose entity is behind the camera is hidden: it leaves
// the probe's elements, which autoplay presses from. Ways this could fail:
// the lift applied after projecting (in px), the node left in flow at the
// root's corner, or a target behind the eye mirrored onto the screen.
test("an anchored node sits on its entity's projection and hides behind the camera", () => {
  loadNativeHost();
  const game = og.game({
    start(world) {
      world.spawn({ name: "front", position: { x: 0, y: 0, z: 0 } });
      world.spawn({ name: "behind", position: { x: 0, y: 0, z: 20 } });
      const cam = world.resource(og.Camera);
      if (!cam.ok) throw new Error(cam.reason);
      cam.value.position = { x: 0, y: 0, z: 10 };
      cam.value.quaternion = og.transform.lookAt(cam.value.position, { x: 0, y: 0, z: 0 });
    },
    ui(world) {
      const plate = (name: string) =>
        View({ anchor: world.find(name)!, lift: [0, 1, 0], role: "button", label: name, onPress: () => {}, class: "w-[40px] h-[20px]" });
      mount(world, () => View({ children: [plate("front"), plate("behind")] }), {});
    },
  });
  const created = createEngine(game as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  const id = (engine.world as unknown as { id: number }).id;
  try {
    og.ui.previewViewport(engine.world, { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" } as never);
    engine.advance(3);
    const frame = JSON.parse(native().og_node_ui_frame(id)) as { elements: { name: string; role: string; x: number; y: number; width: number; height: number }[] };
    const shown = frame.elements.filter((e) => e.role === "button");
    expect(shown.map((e) => e.name)).toEqual(["front"]);
    const [front] = shown;
    // The centre column, above the middle by one world unit at distance 10.
    expect(front!.x).toBeCloseTo(640, 0);
    expect(front!.y).toBeLessThan(350);
    expect(front!.y).toBeGreaterThan(200);
    expect([front!.width, front!.height]).toEqual([40, 20]);
  } finally {
    renderers.delete(id);
    engine.destroy();
  }
});

test("og.ui's unitScale lays the tree out in its units, and the probe and the virtual cursor answer in the surface's px", () => {
  loadNativeHost();
  const Cursor = defineSystem("UiCursor", { reads: [] }, () => og.ui.setVirtualCursor({} as never));
  const game = og.game({
    systems: [Cursor],
    start(world) {
      og.ui.configure(world, { unitScale: 2 });
    },
    ui(world) {
      mount(world, () => View({ role: "button", label: "Box", onPress: () => {}, class: "absolute left-[10px] top-[30px] w-[40px] h-[20px]" }), {});
    },
  });
  const created = createEngine(game as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  const id = (engine.world as unknown as { id: number }).id;
  try {
    og.ui.previewViewport(engine.world, { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" } as never);
    engine.advance(3);
    const frame = JSON.parse(native().og_node_ui_frame(id)) as { elements: { name: string; role: string; x: number; y: number; width: number; height: number }[] };
    const box = frame.elements.find((e) => e.role === "button")!;
    // Two surface px to the unit: twice the authored lengths.
    expect([box.x, box.y, box.width, box.height]).toEqual([20, 60, 80, 40]);
    // The cursor starts in the surface's middle.
    const cursor = og.ui.virtualCursor()!;
    expect([cursor.x, cursor.y]).toEqual([640, 360]);
  } finally {
    renderers.delete(id);
    engine.destroy();
  }
});

// og.ui's frame answers come from og-runtime's UI frame (og.ui.begin's
// record): the surface in units, the clock by the world's tick. Ways it
// could fail: an answer read from a frame nothing fills (the default
// 1280x720 at unit 1, tick 0), the unit not applied, or a clock that a
// snapshot restore leaves running on (the UI's own count, not the tick's).
test("og.ui's frame answers are the tick's surface in units and its clock, restored with the world", () => {
  loadNativeHost();
  const seen: { width: number; height: number; dpr: number; tick: number; seconds: number; dir: string }[] = [];
  const Watch = defineSystem("UiFrameWatch", { reads: [] }, () => {
    const v = og.ui.frameViewport();
    seen.push({ width: v.width, height: v.height, dpr: v.dpr, tick: og.ui.tick(), seconds: og.ui.seconds(), dir: og.ui.frameDirection() });
  });
  const game = og.game({
    systems: [Watch],
    start(world) {
      og.ui.configure(world, { unitScale: 2 });
    },
  });
  const created = createEngine(game as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  try {
    og.ui.previewViewport(engine.world, { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" } as never);
    engine.advance(2);
    const snap = engine.probe.snapshot() as { id: string };
    engine.advance(1);
    const third = seen.at(-1)!;
    // Two CSS px to the unit: half the surface, twice the density.
    expect([third.width, third.height, third.dpr]).toEqual([640, 360, 2]);
    expect([third.tick, third.seconds, third.dir]).toEqual([3, 3 / 60, "ltr"]);
    expect(engine.probe.restore(snap.id)).toBeTrue();
    engine.advance(1);
    // The restored world's next tick is the third again.
    expect(seen.at(-1)!.tick).toBe(3);
  } finally {
    engine.destroy();
  }
});

// og.ui.feedback plays its event's haptic pattern into the frame answer,
// at the player's strength. Ways it could fail: feedback reaching only
// og-ui's queue, which a TSX game's frame never drains (no `haptics`);
// setHaptics not scaling it; a pattern answered twice.
test("og.ui.feedback's haptic reaches the frame answer at setHaptics' strength", () => {
  loadNativeHost();
  let tick = 0;
  const Buzz = defineSystem("UiFeedbackBuzz", { reads: [] }, () => {
    tick += 1;
    if (tick === 2) og.ui.setHaptics(0.5);
    if (tick === 3) og.ui.feedback("confirm", -1);
  });
  const created = createEngine(og.game({ systems: [Buzz] }) as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  try {
    engine.advance(3);
    // confirm is a tap: 18 ms at 0.2 low and 0.6 high, here at half.
    const [[pulse]] = engine.frame(0).haptics as number[][][];
    expect(pulse![0]).toBe(0);
    expect(pulse![1]).toBe(18);
    expect(pulse![2]).toBeCloseTo(0.1, 5);
    expect(pulse![3]).toBeCloseTo(0.3, 5);
    expect(engine.frame(0).haptics).toBe(undefined);
  } finally {
    engine.destroy();
  }
});

// The UI's own actions are og-input's (the `ui` action map): a rebind of
// confirm moves its prompt as a game action's does. Ways it could fail:
// the glyph re-asked on another revision than og-input's bindings (the
// rebind shows a frame late or never), or confirm not in og-input at all.
test("a glyph prompt shows the player's rebinding on the next frame", () => {
  loadNativeHost();
  const game = og.game({
    start(world) {
      og.input.defineAction(world, { name: "use", bindings: [{ control: "KeyE" }] });
    },
    ui(world) {
      mount(world, () => View({ children: [Text({ glyph: "use" }), Text({ glyph: "confirm" })] }), {});
    },
  });
  const created = createEngine(game as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  const id = (engine.world as unknown as { id: number }).id;
  try {
    og.ui.previewViewport(engine.world, { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" } as never);
    const shown = () => {
      // The one element with text: the glyph.
      const frame = JSON.parse(native().og_node_ui_frame(id)) as { elements: { text: string }[] };
      return frame.elements.filter((e) => e.text).map((e) => e.text);
    };
    engine.advance(3);
    // No pack art in this project: the binding's label in its place.
    expect(shown()).toEqual(["E", "Enter"]);
    const r = og.input.rebind(engine.world, { action: "use", slot: 0, binding: { control: "KeyQ" } } as never);
    expect(r.ok).toBe(true);
    const c = og.input.rebind(engine.world, { action: "confirm", slot: 0, binding: { control: "KeyF" } } as never);
    expect(c.ok).toBe(true);
    engine.advance(1);
    expect(shown()).toEqual(["Q", "F"]);
  } finally {
    renderers.delete(id);
    engine.destroy();
  }
});

// FlowChartScreen draws og.flow.chart through NodeGraph. Ways this could
// fail: a pressed node reporting nothing or another node's id (the graph's
// pick not passed through); Play from here firing for a node the player
// cannot play from; the chart's nodes not reaching the graph at all.
test("FlowChartScreen passes a picked node's id through and plays only where it can", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-flowchart-"));
  try {
    writeFileSync(join(dir, "project.ogt"), "type project\nversion 1\n---\n");
    const screen = fileURLToPath(new URL("../src/ui/screens/flow-chart.tsx", import.meta.url));
    writeFileSync(join(dir, "main.ts"), `export { FlowChartScreen, fixtures } from ${JSON.stringify(screen)};\n`);
    const out = join(dir, "game.js");
    const b = await bundleGame(join(dir, "main.ts"), out, "native");
    if (!b.ok) throw new Error(b.reason);
    const rt = fileURLToPath(new URL("../src/ui/rt.ts", import.meta.url));
    const ui = fileURLToPath(new URL("../src/ui/index.ts", import.meta.url));
    writeFileSync(out, readFileSync(out, "utf8").replace(/(["'])[^"']*\/ui\/rt(\.ts)?\1/g, JSON.stringify(rt)).replace(/(["'])@origingame\/engine\/ui\1/g, JSON.stringify(ui)));
    const { FlowChartScreen, fixtures } = await import(out);
    const { loadNativeHost } = await import("../src/node/index.ts");
    loadNativeHost();

    const batches: Op[][] = [];
    let next: Uint8Array = reply([]);
    const world = { id: 9004 } as unknown as World;
    renderers.set(9004, new Renderer((bytes) => { batches.push(decode(bytes)); const r = next; next = reply([]); return r; }, world));
    const canPlay = createSignal(false);
    const picked: string[] = [];
    let played = 0;
    mount(world, FlowChartScreen, { ...fixtures.default, get canPlay() { return canPlay(); }, onPick: (_e: unknown, id: string) => picked.push(id), onPlay: () => played++ });
    const ops = () => batches.flat();
    const texts = ops().filter((op) => op[0] === "createText").map((op) => op[2]);
    for (const label of ["Kitchen", "Cellar", "Dawn"]) expect(texts).toContain(label);
    const parent = (id: number) => ops().find((op) => op[0] === "insert" && op[2] === id)![1] as number;
    const pressable = new Set(ops().filter((op) => op[0] === "listen" && op[2] === 0 && op[3] === 1).map((op) => op[1] as number));
    const around = (text: string) => {
      let id = ops().find((op) => op[0] === "createText" && op[2] === text)![1] as number;
      while (!pressable.has(id)) id = parent(id);
      return id;
    };
    const play = around(texts.find((t) => /play/i.test(String(t))) as string);

    // NodeGraph captions a node beside its plate: the plates are the pressable views
    // other than Back and Play, in the chart's node order.
    const back = around("screen.common.back");
    const plates = [...pressable].filter((id) => id !== back && id !== play).sort((x, y) => x - y);
    const node = (id: string) => plates[(fixtures.default.nodes as { id: string }[]).findIndex((n) => n.id === id)]!;
    expect(plates).toHaveLength(6);
    next = reply([[node("shy"), 0], [node("hall"), 0], [play, 0]]);
    renderers.get(9004)!.flush();
    renderers.get(9004)!.flush();
    expect(picked).toEqual(["shy", "hall"]);
    expect(played).toBe(0);

    canPlay.set(true);
    renderers.get(9004)!.flush();
    next = reply([[play, 0]]);
    renderers.get(9004)!.flush();
    renderers.get(9004)!.flush();
    expect(played).toBe(1);
  } finally {
    rmSync(dir, { recursive: true, force: true });
    renderers.delete(9004);
  }
});

// A built-in screen draws a row only where the game passes its handler
// (screens README, Rows): a row it draws without one is dead, a press that
// does nothing. Every screen with action rows is mounted once per handler
// its source names, with that one left out and every other one a spy; each
// button drawn is pressed, and a press that reaches no spy and redraws
// nothing is a dead row.
// Its own rows (PauseScreenProps.rows) take their place and report onRow.
test("a built-in screen draws no row the game passes no handler for", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-rows-"));
  const screens = ["pause", "main-menu", "results", "game-over", "title", "settings", "mode-select", "save-load"];
  try {
    writeFileSync(join(dir, "project.ogt"), "type project\nversion 1\n---\n");
    const src = (n: string) => fileURLToPath(new URL(`../src/ui/screens/${n}.tsx`, import.meta.url));
    writeFileSync(join(dir, "main.ts"), screens.map((n, i) => `export * as S${i} from ${JSON.stringify(src(n))};`).join("\n") + "\n");
    const out = join(dir, "game.js");
    const b = await bundleGame(join(dir, "main.ts"), out, "native");
    if (!b.ok) throw new Error(b.reason);
    const rt = fileURLToPath(new URL("../src/ui/rt.ts", import.meta.url));
    const ui = fileURLToPath(new URL("../src/ui/index.ts", import.meta.url));
    writeFileSync(out, readFileSync(out, "utf8").replace(/(["'])[^"']*\/ui\/rt(\.ts)?\1/g, JSON.stringify(rt)).replace(/(["'])@origingame\/engine\/ui\1/g, JSON.stringify(ui)));
    const mods = (await import(out)) as Record<string, Record<string, unknown>>;
    const { loadNativeHost } = await import("../src/node/index.ts");
    loadNativeHost();

    let id = 9100;
    /** Mounts a screen; answers its buttons' labels and which spies a press of each reached. */
    const presses = (Screen: unknown, props: Record<string, unknown>, spies: string[]) => {
      const world = { id: ++id } as unknown as World;
      const batches: Op[][] = [];
      let next: Uint8Array = reply([]);
      renderers.set(id, new Renderer((bytes) => { batches.push(decode(bytes)); const r = next; next = reply([]); return r; }, world));
      const called: string[] = [];
      const p: Record<string, unknown> = { ...props };
      for (const s of spies) p[s] = (_e: unknown, key?: string) => called.push(key === undefined ? s : `${s}:${key}`);
      mount(world, Screen as never, p);
      const ops = () => batches.flat();
      const buttons = ops().filter((op) => op[0] === "setProp" && op[2] === "role" && op[3] === "button").map((op) => op[1] as number);
      const label = (b: number): string => {
        const under = (x: number): number[] => ops().filter((op) => op[0] === "insert" && op[1] === x).map((op) => op[2] as number);
        const queue = [b];
        while (queue.length) {
          const x = queue.shift()!;
          const text = ops().find((op) => op[0] === "createText" && op[1] === x);
          if (text) return String(text[2]);
          queue.push(...under(x));
        }
        return "";
      };
      const out = buttons.map((b) => {
        const before = called.length;
        const drawn = ops().length;
        next = reply([[b, 0]]);
        renderers.get(id)!.flush();
        renderers.get(id)!.flush();
        // A press that only changes the screen's own state (a binding row listening) redraws it.
        return { label: label(b), reached: called.slice(before), redrew: ops().length > drawn };
      });
      renderers.delete(id);
      return out;
    };

    screens.forEach((name, i) => {
      const mod = mods[`S${i}`]!;
      const Screen = Object.values(mod).find((v) => typeof v === "function")!;
      const base = (mod.fixtures as Record<string, Record<string, unknown>>).default!;
      const handlers = [...new Set([...readFileSync(src(name), "utf8").matchAll(/props\.(on[A-Z]\w*)/g)].map((m) => m[1]!))];
      for (const left of handlers) {
        const props = Object.fromEntries(Object.entries(base).filter(([k]) => !k.startsWith("on")));
        const dead = presses(Screen, props, handlers.filter((h) => h !== left)).filter((p) => p.reached.length === 0 && !p.redrew);
        expect({ screen: name, without: left, dead: dead.map((d) => d.label) }).toEqual({ screen: name, without: left, dead: [] });
      }
    });

    // Star Focus's pause menu: three built-in rows and one of its own after Resume.
    const pause = mods.S0!;
    const rows = presses(pause.PauseScreen, { objective: "", played: 0, canSave: false, rows: [{ key: "restart", label: "Restart stage", after: "resume" }] }, ["onResume", "onSettings", "onQuitToMenu", "onRow"]);
    expect(rows.map((r) => r.label)).toEqual(["screen.pause.resume", "Restart stage", "screen.pause.settings", "screen.pause.quitToMenu"]);
    expect(rows.map((r) => r.reached)).toEqual([["onResume"], ["onRow:restart"], ["onSettings"], ["onQuitToMenu"]]);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

// An attribute written after a spread wins on every update, not only the
// first: Button (`<View {...props} class={merged}>`) keeps its merged class
// when a reactive prop of the caller's changes, rather than taking the
// caller's bare `class` and losing min-h-11.
test("a caller's class and a reactive prop keep a component's merged class", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-spread-"));
  try {
    writeFileSync(join(dir, "project.ogt"), "type project\nversion 1\n---\n");
    writeFileSync(join(dir, "main.tsx"), `import { Text } from "@origingame/engine/ui";
import { Button } from "@origingame/engine/ui/components";
export function Probe(props: { off: boolean }) {
  return <Button class="text-fg" disabled={props.off}><Text>a</Text></Button>;
}
`);
    const out = join(dir, "game.js");
    const b = await bundleGame(join(dir, "main.tsx"), out, "native");
    if (!b.ok) throw new Error(b.reason);
    const rt = fileURLToPath(new URL("../src/ui/rt.ts", import.meta.url));
    const ui = fileURLToPath(new URL("../src/ui/index.ts", import.meta.url));
    writeFileSync(out, readFileSync(out, "utf8").replace(/(["'])[^"']*\/ui\/rt(\.ts)?\1/g, JSON.stringify(rt)).replace(/(["'])@origingame\/engine\/ui\1/g, JSON.stringify(ui)));
    const { Probe } = await import(out);
    loadNativeHost();
    const batches: Op[][] = [];
    const world = { id: 9200 } as unknown as World;
    renderers.set(9200, new Renderer((bytes) => { batches.push(decode(bytes)); return reply([]); }, world));
    const off = createSignal(false);
    mount(world, Probe, { get off() { return off(); } });
    const ops = () => batches.flat();
    const button = ops().find((op) => op[0] === "setProp" && op[2] === "role" && op[3] === "button")![1] as number;
    const cls = () => String(ops().filter((op) => op[0] === "setClass" && op[1] === button).at(-1)![2]);
    expect(cls()).toContain("min-h-11");
    for (const v of [true, false, true]) {
      off.set(v);
      renderers.get(9200)!.flush();
      expect(ops()).toContainEqual(["setProp", button, "disabled", String(v)]);
      expect(cls()).toContain("min-h-11");
      expect(cls()).toContain("text-fg");
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
    renderers.delete(9200);
  }
});

// A built-in full-screen screen fills the layer it is mounted in and takes
// the caller's `class` after its own (screens README, Placement). Ways this
// could fail: a root sized to its content (Star Focus's title, a 198 px band
// in an 844 px column), a root that fills only along a flex parent's main
// axis, or a caller's class dropped or applied before the screen's own.
const FULL_SCREENS = ["calibration", "creator-tools", "credits", "flow-chapters", "flow-chart", "flow-gallery", "game-over", "hud-editor", "leaderboard", "loading", "main-menu", "mode-select", "pause", "results", "save-load", "settings", "splash", "title"];
test("every built-in full-screen screen fills a game's 390x844 column and takes its class", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-fill-"));
  try {
    writeFileSync(join(dir, "project.ogt"), "type project\nversion 1\n---\n");
    const src = (n: string) => fileURLToPath(new URL(`../src/ui/screens/${n}.tsx`, import.meta.url));
    writeFileSync(join(dir, "main.ts"), FULL_SCREENS.map((n, i) => `export * as S${i} from ${JSON.stringify(src(n))};`).join("\n") + "\n");
    const out = join(dir, "game.js");
    const b = await bundleGame(join(dir, "main.ts"), out, "native");
    if (!b.ok) throw new Error(b.reason);
    const rt = fileURLToPath(new URL("../src/ui/rt.ts", import.meta.url));
    const ui = fileURLToPath(new URL("../src/ui/index.ts", import.meta.url));
    writeFileSync(out, readFileSync(out, "utf8").replace(/(["'])[^"']*\/ui\/rt(\.ts)?\1/g, JSON.stringify(rt)).replace(/(["'])@origingame\/engine\/ui\1/g, JSON.stringify(ui)));
    const mods = (await import(out)) as Record<string, Record<string, unknown>>;
    loadNativeHost();

    type El = { id: string; parent: string; name: string; role: string; x: number; y: number; width: number; height: number };
    const rootOf = (name: string, extra?: string, height = 844) => {
      const mod = mods[`S${FULL_SCREENS.indexOf(name)}`]!;
      const Screen = Object.entries(mod).find(([k, v]) => /Screen$/.test(k) && typeof v === "function")![1] as (p: unknown) => never;
      const props = { ...(mod.fixtures as Record<string, Record<string, unknown>>).default, ...(extra ? { class: extra } : {}) };
      const game = og.game({
        ui(world) {
          // A game's own column, as Star Focus composes its screens: a flex column, not the mount's grid cell.
          mount(world, () => View({ role: "group", label: "column", class: `absolute left-0 top-0 w-[390px] h-[${height}px] flex-col`, children: Screen(props) }), {});
        },
      });
      const created = createEngine(game as unknown as GameToken);
      if (!created.ok) throw new Error(created.reason);
      const engine = created.value;
      const id = (engine.world as unknown as { id: number }).id;
      try {
        for (let i = 0; i < 3; i++) engine.frame(1000 / 60);
        const els = (JSON.parse(native().og_node_ui_frame(id)) as { elements: El[] }).elements;
        // Under the surface and the mount's layer (`1/<layer>`): the game's column, then the screen's root.
        // Paths run mount/column/root: the screen's root is the column's one child.
        const root = els.find((e) => (e as El & { path: string }).path.split("/").length === 3)!;
        const shown = (text: string) => els.some((e) => (e as El & { text: string }).text === text && e.y >= 0 && e.y < height);
        return { name, box: [root.x, root.y, root.width, root.height], shown };
      } finally {
        renderers.delete(id);
        engine.destroy();
      }
    };
    expect(FULL_SCREENS.map((name) => { const r = rootOf(name); return { name: r.name, box: r.box }; })).toEqual(FULL_SCREENS.map((name) => ({ name, box: [0, 0, 390, 844] })));
    // Results never clips its grade, stats or rewards: all on a phone, and on a short column the page scrolls from its top.
    const results = rootOf("results");
    expect(["A", "Score", "Kills", "screen.results.rewards"].map((t) => [t, results.shown(t)])).toEqual([["A", true], ["Score", true], ["Kills", true], ["screen.results.rewards", true]]);
    const short = rootOf("results", undefined, 360);
    expect({ box: short.box, title: short.shown("Victory") }).toEqual({ box: [0, 0, 390, 360], title: true });
    // The caller's class comes after the screen's own: an inset it gives wins.
    expect(rootOf("title", "top-[100px]").box).toEqual([0, 100, 390, 744]);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
