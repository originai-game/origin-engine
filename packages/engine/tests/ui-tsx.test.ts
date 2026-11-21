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

    // The mount's one batch builds the whole tree and puts its root in 0.
    expect(batches).toHaveLength(1);
    const first = batches[0]!;
    expect(first).toContainEqual(["create", 2, 0]);
    expect(first).toContainEqual(["setClass", 2, "p-4 bg-surface"]);
    expect(first).toContainEqual(["listen", 2, 0, 1]);
    expect(first).toContainEqual(["createText", 4, "HP "]);
    expect(first).toContainEqual(["createText", 5, "3"]);
    expect(first.at(-1)).toEqual(["insert", 0, 1, -1]);
    // The mount's root fills the surface, so a component's `absolute inset-0` covers the screen.
    expect(first).toContainEqual(["setClass", 1, "absolute inset-0 flex-col"]);
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
    writeFileSync(join(dir, "main.ts"), `export { Spinner } from "@origingame/engine/ui/components";\n`);
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
      mount(world, () => Text({ children: () => String(n()) }), {});
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
  renderers.delete(id);
});

// FilmChoices (the 底牌 playtest, film guide §4) takes og.film.view's value and
// derives its rows itself. Ways this could fail: a pressed row or hotspot
// reporting its place in the list instead of ChoiceView.index (choice 3 is
// listed before 1 here); a disabled choice, or any after the lock, still
// taking presses; the pending choice not marked; the screen showing with no
// window.
test("FilmChoices shows a film's window from its view and reports the ChoiceView.index pressed", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-film-"));
  try {
    writeFileSync(join(dir, "project.ogt"), "type project\nversion 1\n---\n");
    writeFileSync(join(dir, "main.ts"), `export { FilmChoices } from "@origingame/engine/ui/components";\n`);
    const out = join(dir, "game.js");
    const b = await bundleGame(join(dir, "main.ts"), out, "native");
    if (!b.ok) throw new Error(b.reason);
    const rt = fileURLToPath(new URL("../src/ui/rt.ts", import.meta.url));
    const ui = fileURLToPath(new URL("../src/ui/index.ts", import.meta.url));
    writeFileSync(out, readFileSync(out, "utf8").replace(/(["'])[^"']*\/ui\/rt(\.ts)?\1/g, JSON.stringify(rt)).replace(/(["'])@origingame\/engine\/ui\1/g, JSON.stringify(ui)));
    const { FilmChoices } = await import(out);
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
    mount(world, FilmChoices, { get view() { return v(); }, onChoose: (_e: unknown, index: number) => chosen.push(index) });
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
