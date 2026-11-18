// Exercise the documents through the native host, as a game consumes them.
// In particular, a QTE is an input surface, not a gameplay timer: the game
// supplies resolution; early release cancels a hold and keyed rows retain ids.
import { afterAll, beforeAll, expect, test } from "bun:test";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineSystem, og } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { cookProject } from "./support/og-bin.ts";

const ENGINE = dirname(dirname(fileURLToPath(import.meta.url)));
const PATHS = ["base/pips", "hud/boss-bar", "hud/combo", "hud/hero-button", "hud/pickup-feed", "hud/perf-readout", "hud/qte-prompt", "hud/status-grid"];
const members = PATHS.map((path) => {
  const text = readFileSync(join(ENGINE, `src/ui/components/${path}.svelte`), "utf8");
  const name = /name="component" content="([^"]+)"/.exec(text)![1]!;
  const fixtures = Object.fromEntries([...text.matchAll(/name="fixture:([^"]+)" content='([^']*)'/g)].map((m) => [m[1]!, JSON.parse(m[2]!)]));
  return { name, fixtures };
});
let root = "";
type El = { path: string; text: string; value: string; role: string; name: string; disabled: boolean; states: string[]; x: number; y: number; width: number; height: number };
type Intent = { name: string; rowKey: string; path: string };
beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-hud-builtins-"));
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  mkdirSync(join(root, "assets/icons"), { recursive: true });
  copyFileSync(join(ENGINE, "../../templates/styles/shared/ink-icons.png"), join(root, "assets/icons/ink-icons.png"));
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "HUD builtins"\nid "hud-builtins"\nentry "main.ts"\nprofile "2d"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  for (const m of members) {
    const props = Object.keys(m.fixtures.default).map((p) => `${p}={$${p === "state" ? "statusValue" : p}}`).join(" ");
    writeFileSync(join(root, `assets/ui/${m.name}.svelte`), `<meta name="screen" content="Host_${m.name}" data-layer="menu" data-input="menu">\n<div id="Page" style="width: stretch; padding: 16px"><${m.name} ${props} /></div>`);
  }
  writeFileSync(join(root, "assets/ui/keyed.svelte"), '<meta name="screen" content="Keyed" data-layer="menu" data-input="menu">\n<div id="Page">{#each prompts as p (p.id)}<QtePrompt label={p.label} mode={p.mode} holdMs={300} state={p.state} disabled={p.disabled} />{/each}</div>');
  cookProject(root, root);
});
afterAll(() => { if (root) rmSync(root, { recursive: true, force: true }); });

function mount(name: string, stores: Record<string, unknown>, phone = false, events: Intent[] = []): Engine {
  const Collect = defineSystem("HudBuiltinIntents", {}, (_q, _t, w) => {
    for (const e of og.ui.docEvents(w)) events.push(e as unknown as Intent);
  });
  const r = createEngine(og.game({
    systems: [Collect],
    start(w) {
      const ink = og.load.texture(w, "icons/ink-icons.png");
      if (!ink.ok) throw new Error(ink.reason);
      og.ui.configure(w, { style: null, fallbackIcons: { texture: ink.value, columns: 8, rows: 4, names: ["heart", "star", "coin", "lock", "pause", "gear", "sword", "shield"] } });
      const d = og.load.document(w, `ui/${name === "Keyed" ? "keyed" : name}.svelte`);
      if (!d.ok) throw new Error(d.reason);
    },
    ui(w) {
      const scope = og.ui.scope(w, "test");
      const inputs = Object.fromEntries(Object.entries(stores).map(([k, v]) => [k === "state" ? "statusValue" : k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
      const m = scope.mount("page", name === "Keyed" ? name : `Host_${name}`, inputs);
      if (!m.ok) throw new Error(m.reason);
    },
  }) as unknown as GameToken, { root });
  if (!r.ok) throw new Error(r.reason);
  og.ui.previewViewport(r.value.world, { width: phone ? 390 : 1280, height: phone ? 844 : 720, dpr: 1, safeArea: { left: 0, right: 0, top: 0, bottom: 0 }, coarsePointer: phone, breakpoint: phone ? "phonePortrait" : "desktop" });
  r.value.advance(3);
  return r.value;
}
const elements = (e: Engine) => (e.probe.dump({ what: "ui" }) as unknown as { elements: El[] }).elements;
const words = (e: Engine) => elements(e).filter((x) => x.width > 0).map((x) => x.text);
const button = (e: Engine, label: string) => elements(e).find((x) => x.role === "button" && x.name === label)!;
const tap = (e: Engine, el: El, duration = 0.06) => {
  e.probe.commands(`0:touch_move:${el.x + el.width / 2},${el.y + el.height / 2};0.02:touch_start;${duration}:touch_end`);
  e.advance(Math.ceil(duration * 60) + 5);
};
function check(name: string, fixture: string, fn: (e: Engine) => void, over: Record<string, unknown> = {}) {
  const m = members.find((x) => x.name === name)!;
  const e = mount(name, { ...m.fixtures.default, ...m.fixtures[fixture], ...over });
  try { fn(e); } finally { e.destroy(); }
}

for (const phone of [false, true]) test(`status counts and seconds read side by side below their icon on ${phone ? "phone" : "desktop"}`, () => {
  const m = members.find((x) => x.name === "StatusGrid")!;
  const e = mount(m.name, m.fixtures.default, phone);
  try {
    const boxes = elements(e);
    for (const [count, time, label] of [["×3", "4.2 s", "Barrier"], ["×2", "6 s", "Burning"]]) {
      const stack = boxes.find((x) => x.text === count)!;
      const timer = boxes.find((x) => x.text === time)!;
      const effect = boxes.find((x) => x.name === label)!;
      const icon = boxes.find((x) => x.role === "image" && x.path.startsWith(effect.path + "/"))!;
      expect(stack).toBeDefined();
      expect(timer).toBeDefined();
      expect(Math.min(stack.y + stack.height, timer.y + timer.height)).toBeGreaterThan(Math.max(stack.y, timer.y));
      expect(stack.x + stack.width).toBeLessThanOrEqual(timer.x);
      expect(Math.min(stack.y, timer.y)).toBeGreaterThanOrEqual(icon.y + icon.height);
    }
  } finally { e.destroy(); }
});

for (const phone of [false, true]) for (const m of members) test(`${m.name}: all fixtures are readable in default style on ${phone ? "phone" : "desktop"}`, () => {
  for (const [name, fixture] of Object.entries(m.fixtures)) {
    const e = mount(m.name, { ...m.fixtures.default, ...fixture as object }, phone);
    try {
      expect(og.ui.scope(e.world, "test").takeErrors()).toEqual([]);
      const warnings: string[] = JSON.parse(native().og_node_ui_frame((e as unknown as { id: number }).id)).warnings ?? [];
      expect(warnings).toEqual([]);
      const boxes = elements(e).filter((x) => x.width > 0);
      expect(boxes.length).toBeGreaterThan(2);
      for (const b of boxes) expect(`${m.name}/${name} ${b.path}: ${b.x + b.width <= (phone ? 390 : 1280) + 0.5}`).toEndWith("true");
    } finally { e.destroy(); }
  }
}, 120_000);

test("resource displays distinguish fractional/empty units, phases and game-supplied outcomes", () => {
  check("Pips", "default", (e) => expect(elements(e).filter((x) => x.role === "meter").map((x) => x.value)).toEqual(["100%", "50%", "0%"]));
  check("Pips", "empty", (e) => expect(words(e)).toContain("No units"));
  check("BossBar", "default", (e) => {
    expect(words(e)).toContain("58%");
    expect(words(e)).toContain("WARDEN OF THE NINEFOLD GATE");
    expect(elements(e).find((x) => x.role === "meter" && x.name === "Boss health")!.value).toBe("5,800 / 10,000");
  });
  check("BossBar", "default", (e) => {
    expect(words(e)).toContain("0%");
    expect(words(e)).not.toContain("Defeated");
  }, { max: 0 });
  check("BossBar", "critical", (e) => expect(words(e)).not.toContain("Stagger"), { stagger: -1 });
  check("Combo", "default", (e) => {
    expect(words(e)).toContain("17");
    expect(words(e)).toContain("×2.5");
    expect(words(e)).toContain("Active");
  }, { remaining: 0 });
  check("PickupFeed", "default", (e) => {
    expect(words(e)).toContain("+12");
    expect(words(e)).toContain("-40");
  });
  check("PerfReadout", "connecting", (e) => {
    expect(words(e).filter((x) => x === "Unavailable")).toHaveLength(4);
    expect(words(e)).not.toContain("0");
    expect(words(e)).not.toContain("0 ms");
  });
  check("PerfReadout", "zero", (e) => {
    expect(words(e)).toContain("0");
    expect(words(e)).toContain("0 ms");
    expect(words(e)).not.toContain("Unavailable");
  });
  check("PerfReadout", "default", (e) => expect(words(e)).toContain("16.7 ms"));
  for (const name of ["Pips", "PickupFeed"]) check(name, "loading", (e) => {
    expect(elements(e).some((x) => x.role === "progressbar" && x.name === "Loading")).toBeTrue();
    expect(words(e)).toContain("Loading");
    expect(words(e)).not.toContain("+12");
    expect(elements(e).filter((x) => x.role === "meter")).toHaveLength(0);
  });
});

test("HeroButton: its name is its words; the key cap beside them is not read with them", () => {
  check("HeroButton", "default", (e) => {
    const named = elements(e).filter((x) => x.role === "button").map((x) => x.name);
    expect(named).toEqual(["End turn"]);
    expect(words(e)).toContain("E");
  });
});

test("QTE press and mash keep stable prompt ids; resolved, waiting and disabled prompts send nothing", () => {
  const events: Intent[] = [];
  const prompts = [{ id: "gate-b", label: "Brace gate", mode: "press", state: "active", disabled: false }, { id: "escape-a", label: "Escape", mode: "mash", state: "active", disabled: false }];
  const e = mount("Keyed", { prompts }, true, events);
  try {
    tap(e, button(e, "Brace gate"));
    tap(e, button(e, "Escape"));
    tap(e, button(e, "Escape"));
    expect(events.filter((x) => x.name === "respond").map((x) => x.rowKey)).toEqual(["gate-b", "escape-a", "escape-a"]);
    const rows = og.ui.scope(e.world, "test").collection("prompts", prompts, { key: "id" });
    rows.move("escape-a", "gate-b"); e.advance(3);
    tap(e, button(e, "Brace gate"));
    expect(events.filter((x) => x.name === "respond").at(-1)!.rowKey).toBe("gate-b");
    e.probe.commands("0:key_down:Enter;0.05:key_up:Enter"); e.advance(8);
    e.probe.commands("0:gamepad:a;0.05:gamepad:-"); e.advance(8);
    expect(events.filter((x) => x.name === "respond").slice(-2).map((x) => x.rowKey)).toEqual(["gate-b", "gate-b"]);
    for (const state of ["success", "failure", "expired", "waiting", "active"]) {
      rows.patch("gate-b", { state, disabled: state === "active" }); e.advance(3);
      const b = button(e, "Brace gate");
      expect(b.disabled).toBeTrue(); tap(e, b);
    }
    expect(events.filter((x) => x.name === "respond")).toHaveLength(6);
  } finally { e.destroy(); }
});

test("QTE hold cancels early, fires once at its threshold, and supports keyboard activation", () => {
  const events: Intent[] = [];
  const e = mount("Keyed", { prompts: [{ id: "brace", label: "Brace", mode: "hold", state: "active", disabled: false }] }, false, events);
  try {
    const b = button(e, "Brace");
    tap(e, b, 0.2);
    expect(events.filter((x) => x.name === "respond")).toEqual([]);
    tap(e, b, 0.8);
    expect(events.filter((x) => x.name === "respond").map((x) => x.rowKey)).toEqual(["brace"]);
    e.probe.commands("0:key_down:Space;0.8:key_up:Space"); e.advance(55);
    expect(events.filter((x) => x.name === "respond").map((x) => x.rowKey)).toEqual(["brace", "brace"]);
    e.probe.commands(`0:touch_move:${b.x + b.width / 2},${b.y + b.height / 2};0.02:touch_start`); e.advance(10);
    expect(og.ui.scope(e.world, "test").collection<{ id: string; state: string }>("prompts", [], { key: "id" }).patch("brace", { state: "expired" }).ok).toBeTrue();
    e.advance(2);
    expect(button(e, "Brace").disabled).toBeTrue();
    expect(events.filter((x) => x.name === "respond")).toHaveLength(2);
    e.advance(43);
    expect(events.filter((x) => x.name === "respond")).toHaveLength(2);
    e.probe.commands("0:touch_end;0.05:key_down:Space;0.8:key_up:Space"); e.advance(55);
    expect(events.filter((x) => x.name === "respond")).toHaveLength(2);
  } finally { e.destroy(); }
});
