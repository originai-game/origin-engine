import { afterAll, beforeAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { defineSystem, og } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { cookProject } from "./support/og-bin.ts";
import { FIXTURES } from "../sheets/profile/src/fixtures.ts";

type El = { path: string; text: string; name: string; role: string; x: number; y: number; width: number; height: number; disabled?: boolean; clipped?: boolean };
let root = "";
beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-profile-"));
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Profile"\nid "profile-test"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});');
  const source = readFileSync(new URL("../src/ui/components/meta/profile.svelte", import.meta.url), "utf8");
  expect(FIXTURES).toEqual(Object.fromEntries([...source.matchAll(/<meta name="fixture:([^"]+)" content='([^']*)'>/g)].map((m) => [m[1]!, JSON.parse(m[2]!)])));
  const props = [...source.matchAll(/name="prop:([^"]+)"/g)].map((m) => `${m[1]}={$${m[1]}}`).join(" ");
  writeFileSync(join(root, "assets/ui/page.svelte"), `<meta name="component" content="ProfileTest"><div style="padding: 16px; width: stretch; height: stretch"><Profile ${props} /></div>`);
  cookProject(root, process.cwd());
});
afterAll(() => { if (root) rmSync(root, { recursive: true, force: true }); });
const elements = (e: Engine) => (e.probe.dump({ what: "ui" }) as unknown as { elements: El[] }).elements.filter((x) => x.width > 0 && x.height > 0);
const texts = (e: Engine) => elements(e).map((x) => x.text);
const tap = (e: Engine, el: El) => { e.probe.commands(`0:touch_move:${el.x + el.width / 2},${el.y + el.height / 2};0.02:touch_start;0.06:touch_end`); e.advance(8); };
function run(fixture: string, width: number, check: (e: Engine, events: string[]) => void, height = 844) {
  const data = { ...FIXTURES.default, ...FIXTURES[fixture] };
  const events: string[] = [];
  const collect = defineSystem("ProfileIntents", {}, (_q, _t, world) => { for (const e of og.ui.docEvents(world)) if (e.kind === "emit") events.push(`${e.name}:${e.rowKey}`); });
  const result = createEngine(og.game({
    systems: [collect],
    start(world) { const r = og.load.document(world, "ui/page.svelte"); if (!r.ok) throw new Error(r.reason); },
    ui(world) {
      const scope = og.ui.scope(world, "profile");
      const inputs = Object.fromEntries(Object.entries(data).map(([k, v]) => [k, Array.isArray(v) ? scope.collection(k, v as { id: string }[], { key: "id" }) : scope.writable(k, v)]));
      const r = scope.mount("page", "ProfileTest", inputs); if (!r.ok) throw new Error(r.reason);
    },
  }) as unknown as GameToken, { root });
  if (!result.ok) throw new Error(result.reason);
  const e = result.value;
  try {
    og.ui.previewViewport(e.world, { width, height, dpr: 1, safeArea: { top: 0, bottom: 0, left: 0, right: 0 }, coarsePointer: width < 600, breakpoint: width < 600 ? "phonePortrait" : "desktop" } as never);
    e.advance(3);
    expect(og.ui.scope(e.world, "profile").takeErrors()).toEqual([]);
    const frame = JSON.parse(native().og_node_ui_frame((e as unknown as { id: number }).id));
    expect((frame.warnings ?? []).filter((x: string) => x.includes("computed-value"))).toEqual([]);
    check(e, events);
  } finally { e.destroy(); }
}
for (const width of [1280, 390]) {
  test(`Profile: all fixture states, privacy and stale-data suppression at ${width}px`, () => {
    for (const fixture of Object.keys(FIXTURES)) run(fixture, width, (e) => {
      const text = texts(e);
      if (["loading", "error", "empty"].includes(fixture)) {
        const lower = text.map((x) => x.toLowerCase());
        expect(lower).not.toContain("brenna vale"); expect(text).not.toContain("128"); expect(lower).not.toContain("add friend");
        expect(text).toContain(fixture === "loading" ? "Loading profile…" : fixture === "empty" ? "No profile found" : "Profile could not be loaded. Check your connection.");
        if (fixture === "loading") expect(elements(e).some((x) => x.role === "progressbar")).toBeTrue();
      } else if (fixture === "private") { expect(text).toContain("Stats are private"); expect(text).not.toContain("128"); }
      else if (fixture === "no-stats") expect(text).toContain("No stats yet");
      if (fixture === "default") {
        expect(text.filter((x) => x.toLowerCase() === "brenna vale")).toHaveLength(1);
        expect(elements(e).find((x) => x.role === "figure")?.name).toBe("Brenna Vale");
        expect(text).toContain("12"); expect(text).toContain("Online");
      }
      for (const el of elements(e).filter((x) => x.text || x.role === "button")) {
        // Long biography/stats can scroll vertically; actions stay in view.
        expect(`${fixture} ${el.path} horizontal fit: ${el.x >= 0 && el.x + el.width <= width + 0.5}`).toBe(`${fixture} ${el.path} horizontal fit: true`);
        if (el.role === "button") expect(el.clipped).not.toBeTrue();
      }
    });
  }, 120_000);
  test(`Profile: stable action id, disabled/pending silence and retry at ${width}px`, () => {
    run("default", width, (e, events) => {
      const scope = og.ui.scope(e.world, "profile");
      scope.collection("actions", [], { key: "id" }).move("invite", "friend"); e.advance(2);
      const buttons = elements(e).filter((x) => x.role === "button");
      tap(e, buttons.find((x) => x.path.includes("#invite/"))!);
      tap(e, buttons.find((x) => x.path.includes("#friend/"))!);
      expect(events).toEqual(["action:friend"]);
      expect(texts(e)).toContain("Player is in a match");
    });
    run("pending", width, (e, events) => {
      const action = elements(e).find((x) => x.role === "button")!;
      expect(action.disabled).toBeTrue(); expect(texts(e).map((x) => x.toLowerCase())).toContain("sending…");
      tap(e, action); expect(events).toEqual([]);
    });
    run("error", width, (e, events) => {
      tap(e, elements(e).find((x) => x.role === "button")!); expect(events).toEqual(["retry:"]);
    });
  }, 120_000);
}

test("Profile: bounded phone scroll preserves statistics and keyboard-accessible actions", () => {
  run("overflow", 390, (e, events) => {
    for (const el of elements(e).filter((x) => x.role === "button" || x.text === "This player does not accept invitations from non-friends")) {
      expect(el.y).toBeGreaterThanOrEqual(0); expect(el.y + el.height).toBeLessThanOrEqual(480);
      expect(el.clipped).not.toBeTrue();
    }
    const scroll = JSON.parse(native().og_node_ui_frame((e as unknown as { id: number }).id)).scrolls.find((x: { contentHeight: number; viewHeight: number }) => x.contentHeight > x.viewHeight);
    expect(scroll).toBeDefined();
    const scroller = elements(e).find((x) => x.path === scroll.path)!;
    e.probe.commands(`0:touch_move:${scroller.x + scroller.width / 2},${scroller.y + scroller.height / 2};0.02:scroll:0,1000`); e.advance(8);
    const last = elements(e).find((x) => x.text === "Across all regions")!;
    expect(last.y).toBeGreaterThanOrEqual(scroller.y);
    expect(last.y + last.height).toBeLessThanOrEqual(scroller.y + scroller.height);
    expect(last.clipped).not.toBeTrue();
    e.probe.commands("0:key_down:Tab;0.05:key_up:Tab;0.1:key_down:Enter;0.15:key_up:Enter"); e.advance(12);
    expect(events).toEqual(["action:friend"]);
  }, 480);
}, 60_000);
