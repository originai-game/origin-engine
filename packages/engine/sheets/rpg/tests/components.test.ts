import { afterAll, beforeAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { defineSystem, og } from "@origingame/engine";
import { loadNativeHost } from "../../../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../../../src/runtime.ts";
import { ogBin } from "../../../tests/support/og-bin.ts";

type Element = { path: string; part: string; role: string; name: string; text: string; x: number; y: number; width: number; height: number; states?: string[]; disabled?: boolean };
type Intent = { name: string; rowKey: string };
let root = "";
beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-c-components-"));
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "C components"\nid "c-components"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  writeFileSync(join(root, "assets/ui/slots.svelte"), `<meta name="screen" content="Slots" data-layer="menu" data-input="menu">
<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 24px">{#each $items as item (item.id)}<ItemSlot name={item.name} icon="potion" count={item.count} cooldown={item.cooldown} cooldownMax={8} selected={item.selected} disabled={item.disabled} locked={item.locked} loading={item.loading} empty={item.empty} showName={true} />{/each}</div>`);
  writeFileSync(join(root, "assets/ui/avatar.svelte"), `<meta name="screen" content="Avatars" data-layer="menu" data-input="menu">
<div style="display: flex; flex-wrap: wrap; gap: 24px; padding: 24px"><Avatar name={$name} level={12} status={$status} loading={$loading} /><Avatar /></div>`);
  const screens = {
    crafting: '<RpgCrafting recipes={$rows} selected={$selected} ingredients={$ingredients} canIncrease={true} busy={$busy} />',
    trade: '<RpgTrade goods={$rows} selected={$selected} mode={$mode} busy={$busy} />',
    loot: '<RpgLoot items={$rows} canTakeAll={true} busy={$busy} />',
    backlog: '<RpgConversationBacklog lines={$rows} playing={$playing} loading={$busy} hasOlder={true} />',
  };
  for (const [name, body] of Object.entries(screens)) writeFileSync(join(root, `assets/ui/${name}.svelte`), `<meta name="screen" content="${name}" data-layer="menu" data-input="menu"><div style="width: stretch; height: stretch; padding: 24px">${body}</div>`);
  const cook = Bun.spawnSync([ogBin(), "cook", root], { env: process.env, stdout: "ignore", stderr: "pipe" });
  if (cook.exitCode) throw new Error(new TextDecoder().decode(cook.stderr));
});
afterAll(() => { if (root) rmSync(root, { recursive: true, force: true }); });

function mount(page: string, data: Record<string, unknown>, phone = false): { engine: Engine; events: Intent[] } {
  const events: Intent[] = [];
  const collect = defineSystem("CollectComponentIntents", {}, (_q, _t, world) => { for (const event of og.ui.docEvents(world)) events.push(event as unknown as Intent); });
  const result = createEngine(og.game({
    systems: [collect],
    start(world) {
      const loaded = og.load.document(world, `ui/${page}.svelte`);
      if (!loaded.ok) throw new Error(loaded.reason);
    },
    ui(world) {
      const scope = og.ui.scope(world, "test");
      const stores = Object.fromEntries(Object.entries(data).map(([key, value]) => [key, Array.isArray(value) ? scope.collection(key, value as { id: string }[], { key: "id" }) : scope.writable(key, value)]));
      const mounted = scope.mount("page", page === "slots" ? "Slots" : page === "avatar" ? "Avatars" : page, stores);
      if (!mounted.ok) throw new Error(mounted.reason);
    },
  }) as unknown as GameToken, { root });
  if (!result.ok) throw new Error(result.reason);
  og.ui.previewViewport(result.value.world, { width: phone ? 390 : 1280, height: phone ? 844 : 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: phone, breakpoint: phone ? "phonePortrait" : "desktop" });
  result.value.advance(3);
  return { engine: result.value, events };
}
const elements = (engine: Engine): Element[] => (engine.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements;
function tap(engine: Engine, element: Element): void {
  engine.probe.commands(`0:touch_move:${element.x + element.width / 2},${element.y + element.height / 2};0.02:touch_start;0.06:touch_end`);
  engine.advance(8);
}

for (const phone of [false, true]) test(`item slots preserve keyed selection and suppress unavailable activation (${phone ? "phone" : "desktop"})`, () => {
  const slot = (id: string, overrides = {}) => ({ id, name: id, count: 1, cooldown: 0, selected: false, disabled: false, locked: false, loading: false, empty: false, ...overrides });
  const items = [slot("cooling", { cooldown: 3, count: 12, selected: true }), slot("locked", { locked: true }), slot("disabled", { disabled: true }), slot("loading", { loading: true }), slot("empty", { empty: true })];
  const { engine, events } = mount("slots", { items }, phone);
  try {
    const button = (id: string) => elements(engine).find((element) => element.role === "button" && element.path.includes(`#${id}/`))!;
    expect(button("cooling").states).toContain("pressed");
    expect(button("locked").disabled).toBe(true);
    expect(new Set(items.map((item) => button(item.id).height)).size).toBe(1);
    expect(elements(engine).some((element) => element.role === "progressbar" && element.path.includes("#loading/"))).toBe(true);
    for (const item of items) tap(engine, button(item.id));
    expect(events.filter((event) => event.name === "choose").map((event) => event.rowKey)).toEqual(["cooling", "empty"]);
    const scope = og.ui.scope(engine.world, "test");
    scope.collection("items", items, { key: "id" }).move("cooling");
    engine.advance(3);
    tap(engine, button("cooling"));
    expect(events.filter((event) => event.name === "choose").map((event) => event.rowKey)).toEqual(["cooling", "empty", "cooling"]);
    engine.probe.commands("0:key_down:Enter;0.05:key_up:Enter");
    engine.advance(8);
    engine.probe.commands("0:gamepad:a;0.05:gamepad:-");
    engine.advance(8);
    expect(events.filter((event) => event.name === "choose").slice(-2).map((event) => event.rowKey)).toEqual(["cooling", "cooling"]);
    expect(elements(engine).some((element) => element.text === "12")).toBe(true);
    expect(elements(engine).some((element) => element.text === "3.0")).toBe(true);
    expect(scope.takeErrors()).toEqual([]);
  } finally { engine.destroy(); }
});

test("avatar presence is worded, defaults have a name, and loading replaces presence", () => {
  const { engine } = mount("avatar", { name: "Brenna", status: "away", loading: false }, true);
  try {
    const words = () => elements(engine).map((element) => element.text);
    expect(words()).toContain("Away");
    expect(words()).toContain("Player");
    expect(elements(engine).some((element) => element.role === "progressbar")).toBe(false);
    const scope = og.ui.scope(engine.world, "test");
    scope.writable("loading", false).set(true);
    engine.advance(3);
    expect(words()).toContain("Loading");
    expect(elements(engine).some((element) => element.role === "progressbar")).toBe(true);
    expect(words()).not.toContain("Away");
    expect(words()).toContain("Brenna");
    expect(scope.takeErrors()).toEqual([]);
  } finally { engine.destroy(); }
});

for (const phone of [false, true]) test(`economy actions retain item keys and pending transactions suppress repeats (${phone ? "phone" : "desktop"})`, () => {
  for (const page of ["crafting", "trade", "loot"]) {
    const rows = [
      { id: "usable", name: "Healing draught", icon: "potion", description: "For the road.", canCraft: true, available: true, reason: "", price: "20 gold", stock: "8", count: 2 },
      { id: "blocked", name: "Storm crystal", icon: "gem", description: "A rare catalyst.", canCraft: false, available: false, reason: "Inventory full", price: "80 gold", stock: "1", count: 1 },
    ];
    const { engine, events } = mount(page, { rows, ingredients: [], selected: "usable", mode: "buy", busy: false }, phone);
    try {
      const action = page === "crafting" ? "craft" : page === "trade" ? "buy" : "take";
      const actionButton = () => elements(engine).find((e) => e.role === "button" && e.part.includes(`.${action}]`) && e.path.includes("#usable/"))!;
      expect(actionButton()).toBeDefined();
      tap(engine, actionButton());
      expect(events.filter((e) => e.name === action).map((e) => e.rowKey)).toEqual(["usable"]);
      const scope = og.ui.scope(engine.world, "test");
      scope.collection("rows", rows, { key: "id" }).move("usable");
      engine.advance(3);
      tap(engine, actionButton());
      expect(events.filter((e) => e.name === action).map((e) => e.rowKey)).toEqual(["usable", "usable"]);
      scope.writable("busy", false).set(true);
      engine.advance(3);
      expect(actionButton().disabled).toBe(true);
      tap(engine, actionButton());
      expect(events.filter((e) => e.name === action)).toHaveLength(2);
      scope.writable("busy", false).set(false);
      scope.writable("selected", "usable").set("blocked");
      engine.advance(3);
      const blocked = elements(engine).find((e) => e.role === "button" && e.part.includes(`.${action}]`) && e.path.includes("#blocked/"))!;
      expect(blocked.disabled).toBe(true);
      expect(elements(engine).some((e) => e.text === "Inventory full")).toBe(true);
      tap(engine, blocked);
      expect(events.filter((e) => e.name === action)).toHaveLength(2);
      expect(scope.takeErrors()).toEqual([]);
    } finally { engine.destroy(); }
  }
});

test("history replay follows entry keys and loading prevents new replays", () => {
  const rows = [{ id: "voice", speaker: "Marissa", text: "Bring the bell home.", canReplay: true }, { id: "silent", speaker: "", text: "The marsh is silent.", canReplay: false }];
  const { engine, events } = mount("backlog", { rows, playing: "", busy: false }, true);
  try {
    tap(engine, elements(engine).find((e) => e.role === "button" && e.name === "Replay Marissa")!);
    expect(events.filter((e) => e.name === "replay").map((e) => e.rowKey)).toEqual(["voice"]);
    og.ui.scope(engine.world, "test").writable("playing", "").set("voice");
    engine.advance(3);
    tap(engine, elements(engine).find((e) => e.role === "button" && e.name === "Stop replay")!);
    expect(events.filter((e) => e.name === "stop").map((e) => e.rowKey)).toEqual(["voice"]);
    const scope = og.ui.scope(engine.world, "test");
    scope.writable("playing", "").set("");
    scope.writable("busy", false).set(true);
    engine.advance(3);
    const replay = elements(engine).find((e) => e.role === "button" && e.name === "Replay Marissa")!;
    expect(replay.disabled).toBe(true);
    tap(engine, replay);
    expect(events.filter((e) => e.name === "replay")).toHaveLength(1);
  } finally { engine.destroy(); }
});

for (const phone of [false, true]) test(`long loot keeps collection and close actions visible (${phone ? "phone" : "desktop"})`, () => {
  const rows = Array.from({ length: 24 }, (_, i) => ({ id: `relic-${i}`, name: `Abbey relic ${i + 1}`, icon: "gem", description: "A recovered relic.", count: 1, available: true, reason: "" }));
  const { engine, events } = mount("loot", { rows, busy: false }, phone);
  try {
    const close = elements(engine).find((e) => e.role === "button" && e.name === "Close")!;
    const all = elements(engine).find((e) => e.role === "button" && e.name === "Take all")!;
    const first = elements(engine).find((e) => e.role === "button" && e.part.includes(".take]") && e.path.includes("#relic-0/"))!;
    for (const button of [first, close, all]) {
      expect(button.height).toBeGreaterThanOrEqual(44);
      expect(button.y + button.height).toBeLessThanOrEqual(phone ? 844 : 720);
    }
    expect(first.y).toBeLessThan(close.y);
    tap(engine, all);
    tap(engine, close);
    expect(events.filter((e) => e.name).map((e) => e.name)).toEqual(["takeAll", "close"]);
  } finally { engine.destroy(); }
});
