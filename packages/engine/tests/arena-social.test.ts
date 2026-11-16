import { afterAll, beforeAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { defineSystem, og } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";
import { FIXTURES } from "../sheets/arena-social/src/fixtures.ts";

type El = { path: string; text: string; role: string; x: number; y: number; width: number; height: number; clipped?: boolean; disabled?: boolean; states?: string[] };
type Intent = { name: string; rowKey: string };
const MEMBERS = { ready: "ArenaReady", vote: "ArenaVote", spectator: "ArenaSpectator" };
let root = "";
beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-arena-social-"));
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Arena social"\nid "arena-social-test"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});');
  for (const [name, component] of Object.entries(MEMBERS)) {
    const source = readFileSync(new URL(`../src/ui/components/arena/${name}.svelte`, import.meta.url), "utf8");
    const fixtures = Object.fromEntries([...source.matchAll(/<meta name="fixture:([^"]+)" content='([^']*)'>/g)].map((m) => [m[1]!, JSON.parse(m[2]!)]));
    expect(FIXTURES[name]).toEqual(fixtures);
    const props = [...source.matchAll(/name="prop:([^"]+)"/g)].map((m) => `${m[1]}={$${m[1]}}`).join(" ");
    writeFileSync(join(root, `assets/ui/${name}.svelte`), `<meta name="component" content="Test_${component}"><div style="width: stretch; height: stretch; padding: 16px"><${component} ${props} /></div>`);
  }
  const cook = Bun.spawnSync([ogBin(), "cook", root], { stdout: "ignore", stderr: "pipe" });
  if (cook.exitCode) throw new Error(new TextDecoder().decode(cook.stderr));
});
afterAll(() => { if (root) rmSync(root, { recursive: true, force: true }); });
const elements = (e: Engine) => (e.probe.dump({ what: "ui" }) as unknown as { elements: El[] }).elements.filter((x) => x.width > 0 && x.height > 0);
const button = (e: Engine, id: string) => elements(e).find((x) => x.role === "button" && x.path.endsWith(`/${id}`))!;
const row = (e: Engine, key: string) => elements(e).find((x) => x.role === "button" && x.path.endsWith(`#${key}`))!;
const tap = (e: Engine, el: El) => { e.probe.commands(`0:touch_move:${el.x + el.width / 2},${el.y + el.height / 2};0.02:touch_start;0.06:touch_end`); e.advance(8); };
function run(name: keyof typeof MEMBERS, fixture: string, width: number, check: (e: Engine, events: Intent[]) => void, height = 844) {
  const data = { ...FIXTURES[name]!.default, ...FIXTURES[name]![fixture] };
  const events: Intent[] = [];
  const collect = defineSystem("SocialIntents", {}, (_q, _t, world) => { for (const event of og.ui.docEvents(world)) if (event.kind === "emit") events.push(event as unknown as Intent); });
  const result = createEngine(og.game({
    systems: [collect],
    start(world) { const r = og.load.document(world, `ui/${name}.svelte`); if (!r.ok) throw new Error(r.reason); },
    ui(world) {
      const scope = og.ui.scope(world, "social");
      const inputs = Object.fromEntries(Object.entries(data).map(([k, v]) => [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
      const r = scope.mount("page", `Test_${MEMBERS[name]}`, inputs);
      if (!r.ok) throw new Error(r.reason);
    },
  }) as unknown as GameToken, { root });
  if (!result.ok) throw new Error(result.reason);
  const e = result.value;
  try {
    og.ui.previewViewport(e.world, { width, height, dpr: 1, safeArea: { top: 0, bottom: 0, left: 0, right: 0 }, coarsePointer: width < 600, breakpoint: width < 600 ? "phonePortrait" : "desktop" } as never);
    e.advance(3);
    expect(og.ui.scope(e.world, "social").takeErrors()).toEqual([]);
    const frame = JSON.parse(native().og_node_ui_frame((e as unknown as { id: number }).id));
    expect((frame.warnings ?? []).filter((x: string) => x.includes("computed-value"))).toEqual([]);
    check(e, events);
  } finally { e.destroy(); }
}
for (const width of [1280, 390]) {
  for (const name of Object.keys(MEMBERS) as (keyof typeof MEMBERS)[]) test(`${name}: every state fits at ${width}px`, () => {
    for (const fixture of Object.keys(FIXTURES[name]!)) run(name, fixture, width, (e) => {
      for (const el of elements(e).filter((x) => x.text || x.role === "button")) {
        expect(`${fixture} ${el.path}: ${el.x >= 0 && el.x + el.width <= width + 0.5 && !el.clipped}`).toBe(`${fixture} ${el.path}: true`);
      }
      const texts = elements(e).map((x) => x.text);
      if (fixture === "loading") expect(texts).toContain(name === "vote" ? "Loading maps…" : "Loading players…");
      if (fixture === "empty") expect(texts).toContain(name === "vote" ? "No maps available" : name === "ready" ? "No players yet" : "No players to watch");
    });
  }, 120_000);
  test(`Ready: accepts/declines, reflects authoritative response and blocks ended/loading/empty checks at ${width}px`, () => {
    run("ready", "default", width, (e, events) => {
      tap(e, button(e, "Accept")); tap(e, button(e, "Decline"));
      expect(events.map((x) => x.name)).toEqual(["accept", "decline"]);
      og.ui.scope(e.world, "social").writable("response", "").set("ready"); e.advance(2);
      expect(button(e, "Accept").states).toContain("pressed");
      expect(button(e, "Decline").states ?? []).not.toContain("pressed");
    });
    for (const fixture of ["disabled", "closed", "loading", "empty"]) run("ready", fixture, width, (e, events) => {
      expect(button(e, "Accept").disabled).toBeTrue(); expect(button(e, "Decline").disabled).toBeTrue();
      tap(e, button(e, "Accept")); tap(e, button(e, "Decline")); expect(events).toEqual([]);
    });
  }, 120_000);
  test(`Vote: selected and winner differ; stable keys survive reorder; unavailable and ended options cannot vote at ${width}px`, () => {
    run("vote", "default", width, (e, events) => {
      expect(row(e, "foundry").states).toContain("pressed");
      const scope = og.ui.scope(e.world, "social");
      scope.collection("options", [], { key: "id" }).move("grove", "foundry"); e.advance(2);
      tap(e, row(e, "grove")); tap(e, row(e, "citadel"));
      expect(events.map((x) => `${x.name}:${x.rowKey}`)).toEqual(["choose:grove"]);
      expect(row(e, "foundry").states).toContain("pressed"); // No optimistic local vote.
      scope.writable("selected", "").set("grove"); e.advance(2);
      expect(row(e, "grove").states).toContain("pressed");
    });
    for (const fixture of ["disabled", "closed"]) run("vote", fixture, width, (e, events) => {
      tap(e, row(e, "grove")); expect(events).toEqual([]);
      if (fixture === "closed") {
        expect(elements(e).map((x) => x.text)).toContain("Winner");
        expect(elements(e).map((x) => x.text)).not.toContain("18s");
      }
    });
  }, 120_000);
  test(`Spectator: keyed follow, camera actions, selected/free modes and loading exit at ${width}px`, () => {
    run("spectator", "default", width, (e, events) => {
      expect(row(e, "kai").states).toContain("pressed");
      tap(e, row(e, "rhea")); tap(e, row(e, "ana"));
      for (const id of ["Previous", "Next", "Free", "Leave"]) tap(e, button(e, id));
      expect(events.map((x) => `${x.name}:${x.rowKey}`)).toEqual(["follow:rhea", "previous:", "next:", "free:", "leave:"]);
      og.ui.scope(e.world, "social").writable("freeCamera", false).set(true); e.advance(2);
      expect(button(e, "Free").states).toContain("pressed");
      expect(row(e, "kai").states ?? []).not.toContain("pressed");
    });
    run("spectator", "loading", width, (e, events) => {
      expect(button(e, "Previous").disabled).toBeTrue();
      expect(button(e, "Leave").disabled).toBeFalse(); tap(e, button(e, "Leave"));
      expect(events.map((x) => x.name)).toEqual(["leave"]);
    });
  }, 120_000);
}

test("Ready, vote and spectator actions are reachable and activatable by keyboard", () => {
  for (const [name, target, expected] of [["ready", /\/Accept$/, "accept:"], ["vote", /#grove$/, "choose:grove"], ["spectator", /\/Leave$/, "leave:"]] as const) {
    run(name, "default", 1280, (e, events) => {
      const focused = () => elements(e).find((x) => x.states?.includes("focused"))?.path ?? "";
      for (let i = 0; i < 12 && !target.test(focused()); i++) {
        e.probe.commands("0:key_down:Tab;0.05:key_up:Tab"); e.advance(8);
      }
      expect(focused()).toMatch(target);
      e.probe.commands("0:key_down:Enter;0.05:key_up:Enter"); e.advance(8);
      expect(events.map((x) => `${x.name}:${x.rowKey}`)).toEqual([expected]);
    });
  }
}, 60_000);

test("Bounded phone panels scroll their last row into view without losing response or exit actions", () => {
  for (const name of ["ready", "vote", "spectator"] as const) run(name, "overflow", 390, (e, events) => {
    const last = () => name === "ready" ? elements(e).find((x) => x.text === "Kai")! : row(e, name === "vote" ? "grove" : "kai");
    const scroll = JSON.parse(native().og_node_ui_frame((e as unknown as { id: number }).id)).scrolls.find((x: { contentHeight: number; viewHeight: number }) => x.contentHeight > x.viewHeight);
    expect(scroll).toBeDefined();
    const scroller = elements(e).find((x) => x.path === scroll.path)!;
    e.probe.commands(`0:touch_move:${scroller.x + scroller.width / 2},${scroller.y + scroller.height / 2};0.02:scroll:0,1000`); e.advance(8);
    expect(last().clipped).not.toBeTrue();
    expect(last().y).toBeGreaterThanOrEqual(scroller.y);
    expect(last().y + last().height).toBeLessThanOrEqual(scroller.y + scroller.height);
    expect(last().y + last().height).toBeLessThanOrEqual(360);
    if (name === "ready") {
      expect(button(e, "Accept").y + button(e, "Accept").height).toBeLessThanOrEqual(360);
      tap(e, button(e, "Accept")); expect(events.map((x) => x.name)).toEqual(["accept"]);
    } else if (name === "spectator") {
      expect(button(e, "Leave").y + button(e, "Leave").height).toBeLessThanOrEqual(360);
      tap(e, last()); tap(e, button(e, "Leave")); expect(events.map((x) => x.name)).toEqual(["follow", "leave"]);
    } else { tap(e, last()); expect(events.map((x) => `${x.name}:${x.rowKey}`)).toEqual(["choose:grove"]); }
  }, 360);
}, 60_000);
