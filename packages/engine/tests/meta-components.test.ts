import { afterAll, beforeAll, expect, setDefaultTimeout, test } from "bun:test";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { defineSystem, og } from "../src/index.ts";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

setDefaultTimeout(60_000);
let root = "";
const defaults: Record<string, Record<string, unknown>> = {};
type El = { path: string; name: string; text: string; role: string; part: string; states: string[]; x: number; y: number; width: number; height: number };
type Intent = { name: string; rowKey: string };
beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-meta-components-"));
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  cpSync(join(import.meta.dir, "../sheets/components/assets/fonts"), join(root, "assets/fonts"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Meta components"\nid "meta-components-test"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  for (const [name, file] of Object.entries({ PassTrack: "meta/pass-track", Reveal: "meta/reveal", Inbox: "meta/inbox", News: "meta/news", TooltipCard: "panels/tooltip-card" })) {
    const source = readFileSync(join(import.meta.dir, `../src/ui/components/${file}.svelte`), "utf8");
    const props = [...source.matchAll(/<meta name="prop:([^"]+)" content="([^"]+)"([^>]*)>/g)];
    defaults[name] = Object.fromEntries(props.map(m => {
      const value = /data-default="([^"]*)"/.exec(m[3]!)?.[1];
      return [m[1], m[2] === "string" ? value ?? "" : JSON.parse(value ?? (m[2] === "list" ? "[]" : m[2] === "bool" ? "false" : "0"))];
    }));
    Object.assign(defaults[name]!, JSON.parse(/<meta name="fixture:default" content='([^']+)'/.exec(source)![1]!));
    writeFileSync(join(root, `assets/ui/${name}.svelte`), `<meta name="screen" content="Host${name}" data-layer="menu" data-input="menu"><div style="display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; padding: 16px"><${name} ${props.map(m => `${m[1]}={$input_${m[1]}}`).join(" ")} /></div>`);
  }
  const cooked = Bun.spawnSync([ogBin(), "cook", root], { cwd: root, stdout: "ignore", stderr: "pipe" });
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
});
afterAll(() => { if (root) rmSync(root, { recursive: true, force: true }); });
function mount(name: string, overrides: Record<string, unknown> = {}) {
  const events: Intent[] = [];
  const collect = defineSystem("MetaIntents", {}, (_q, _t, world) => events.push(...og.ui.docEvents(world) as unknown as Intent[]));
  const result = createEngine(og.game({ systems: [collect], start(world) {
    const font = og.load.font(world, "fonts/InstrumentSans-Regular.ttf"); if (!font.ok) throw new Error(font.reason);
    og.ui.configure(world, { font: font.value, reducedMotion: true });
    const loaded = og.load.document(world, `ui/${name}.svelte`); if (!loaded.ok) throw new Error(loaded.reason);
  }, ui(world) {
    const scope = og.ui.scope(world, "test");
    const inputs = Object.fromEntries(Object.entries({ ...defaults[name], ...overrides }).map(([k, v]) => [`input_${k}`, Array.isArray(v) ? scope.collection(k, v, { key: "id" }) : scope.writable(k, v)]));
    const mounted = scope.mount("page", `Host${name}`, inputs); if (!mounted.ok) throw new Error(mounted.reason);
  } }) as never, { root });
  if (!result.ok) throw new Error(result.reason);
  const engine = result.value;
  og.ui.previewViewport(engine.world, { width: 1280, height: 900, dpr: 1, coarsePointer: false, breakpoint: "desktop", safeArea: { top: 0, bottom: 0, left: 0, right: 0 } });
  engine.advance(3); return { engine, events };
}
function elements(engine: Engine): El[] { return (engine.probe.dump({ what: "ui" }) as unknown as { elements: El[] }).elements; }
function click(engine: Engine, el: El) { engine.probe.commands(`0:touch_move:${el.x + el.width / 2},${el.y + el.height / 2};0.02:touch_start;0.06:touch_end`); engine.advance(8); }
function buttons(engine: Engine) { return elements(engine).filter(e => e.role === "button"); }

test("pass claims are keyed, entitlement-gated and silent during a pending operation", () => {
  for (const [premiumUnlocked, disabled, expected] of [[false, false, ["keys"]], [true, false, ["keys", "blade"]], [true, true, []]] as const) {
    const { engine, events } = mount("PassTrack", { premiumUnlocked, disabled });
    try {
      for (const button of buttons(engine).filter(e => e.name === "Claim")) click(engine, button);
      expect(events.filter(e => e.name === "claim").map(e => e.rowKey)).toEqual([...expected]);
    } finally { engine.destroy(); }
  }
  const { engine } = mount("PassTrack");
  try {
    og.ui.previewViewport(engine.world, { width: 390, height: 844, dpr: 2, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { top: 0, bottom: 0, left: 0, right: 0 } });
    engine.advance(3);
    const track = elements(engine).find(e => e.name === "Season rewards")!;
    expect(elements(engine).some(e => e.text === "Gems")).toBe(false);
    engine.probe.commands(`0:touch_move:${track.x + track.width / 2},${track.y + track.height / 2};0.02:scroll:900,0`);
    engine.advance(12);
    const after = elements(engine).find(e => e.text === "Gems")!;
    expect(after).toBeDefined();
    expect(after.x + after.width).toBeLessThanOrEqual(390);
  } finally { engine.destroy(); }
});
test("reveal keeps rewards private until revealed, and only exposes the state's actions", () => {
  const rewards = [{ id: "rare", name: "Secret prize", amount: "1", icon: "gem", rarity: "rare", fresh: true }];
  for (const [state, action] of [["sealed", "open"], ["loading", ""], ["revealing", "skip"], ["revealed", "continue"]]) {
    const { engine, events } = mount("Reveal", { state, rewards });
    try {
      expect(elements(engine).some(e => e.text === "Secret prize")).toBe(state === "revealed");
      for (const button of buttons(engine)) click(engine, button);
      expect(events.filter(e => ["open", "skip", "continue"].includes(e.name)).map(e => e.name)).toEqual(action ? [action] : []);
      if (state === "revealed") expect(events.find(e => e.name === "inspect")?.rowKey).toBe("rare");
    } finally { engine.destroy(); }
  }
});
test("inbox actions retain the message key, not its attachment key, and respect expiry and busy", () => {
  for (const [expired, busy, expected] of [[false, false, ["claim", "delete"]], [true, false, ["delete"]], [false, true, []]] as const) {
    const messages = [{ id: "mail", subject: "Supplies", body: "Your expedition supplies.", hasAttachments: true, claimable: true, expired, canDelete: true }];
    const { engine, events } = mount("Inbox", { messages, selected: "mail", attachments: [{ id: "attachment", message: "mail", name: "Gold", amount: "500", icon: "coin" }], busy });
    try {
      for (const button of buttons(engine).filter(e => ["Claim attachments", "Delete message"].includes(e.name))) click(engine, button);
      expect(events.filter(e => ["claim", "delete"].includes(e.name)).map(e => [e.name, e.rowKey])).toEqual(expected.map(name => [name, "mail"]));
    } finally { engine.destroy(); }
  }
});
test("news expansion is game-owned and its optional action retains the article key", () => {
  for (const disabled of [false, true]) {
    const { engine, events } = mount("News", { disabled });
    try {
      for (const button of buttons(engine)) click(engine, button);
      expect(events.filter(e => e.name === "open").map(e => e.rowKey)).toEqual(disabled ? [] : ["patch"]);
      expect(events.filter(e => e.name === "choose").map(e => e.rowKey)).toEqual(disabled ? [] : ["patch", "event"]);
    } finally { engine.destroy(); }
  }
});
test("nested meta recovery exposes the parent's retry intent", () => {
  for (const name of ["PassTrack", "Reveal", "Inbox", "News"]) {
    const { engine, events } = mount(name, { error: "Reconnect to continue." });
    try { click(engine, buttons(engine).find(e => e.name === "Try again")!); expect(events.filter(e => e.name === "retry")).toHaveLength(1); } finally { engine.destroy(); }
  }
});
test("comparison presence is translated, restylable, and never treated as a delta", () => {
  const { engine } = mount("TooltipCard", { stats: [{ id: "added", label: "Burn resistance", value: "8%", change: "added", delta: 999, deltaText: "BAD_ADDED_DELTA" }, { id: "removed", label: "Poison resistance", value: "12%", change: "removed", delta: -999, deltaText: "BAD_REMOVED_DELTA" }, { id: "both", label: "Cooldown", value: "3 s", better: true, delta: -1, deltaText: "−1 s" }] });
  try {
    const rows = elements(engine), text = rows.map(e => e.text).join(" ");
    expect(text).toContain("Added"); expect(text).toContain("Removed"); expect(text).toContain("8%"); expect(text).toContain("12%"); expect(text).toContain("−1 s"); expect(text).not.toContain("BAD_");
    expect(rows.some(e => e.part?.includes("data-change=added"))).toBe(true);
    expect(rows.some(e => e.part?.includes("data-change=removed"))).toBe(true);
  } finally { engine.destroy(); }
});
