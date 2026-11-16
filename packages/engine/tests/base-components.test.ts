import { afterAll, beforeAll, expect, setDefaultTimeout, test } from "bun:test";
import { cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { defineSystem, og } from "../src/index.ts";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

setDefaultTimeout(60_000);
let root = "";
type Event = { name: string; rowKey: string };
type Element = { id: string; name: string; text: string; role: string; path: string; x: number; y: number; width: number; height: number; disabled: boolean; states: string[] };

beforeAll(() => {
  loadNativeHost();
  root = mkdtempSync(join(tmpdir(), "og-base-components-"));
  mkdirSync(join(root, "assets/ui"), { recursive: true });
  cpSync(join(import.meta.dir, "../sheets/components/assets/fonts"), join(root, "assets/fonts"), { recursive: true });
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Base components"\nid "base-components-test"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  writeFileSync(join(root, "assets/ui/host.svelte"), `<meta name="screen" content="BaseHost" data-layer="menu" data-input="menu">
<div style="display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; gap: 16px; padding: 16px">
{#if $kind == 'empty'}<EmptyState title="No builds" actionLabel="Create build" disabled={$disabled} />
{:else if $kind == 'error'}<ErrorState body="Reconnect to continue" busy={$busy} canRetry={$canRetry} dismissLabel="Back" />
{:else if $kind == 'row'}{#each $rows as row (row.id)}<ListItem title={row.title} subtitle={row.subtitle} selected={row.selected} disabled={row.disabled} loading={row.loading} />{/each}
{:else}<Steps steps={$rows} />{/if}
</div>`);
  const cooked = Bun.spawnSync([ogBin(), "cook", root], { cwd: root, stdout: "ignore", stderr: "pipe" });
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
});
afterAll(() => { if (root) rmSync(root, { recursive: true, force: true }); });

function mount(kind: string, data: Record<string, unknown> = {}) {
  const events: Event[] = [];
  const collect = defineSystem("CollectBaseIntents", {}, (_q, _t, world) => events.push(...og.ui.docEvents(world) as unknown as Event[]));
  const result = createEngine(og.game({
    systems: [collect],
    start(world) {
      const font = og.load.font(world, "fonts/InstrumentSans-Regular.ttf");
      if (!font.ok) throw new Error(font.reason);
      og.ui.configure(world, { font: font.value, reducedMotion: true });
      const loaded = og.load.document(world, "ui/host.svelte");
      if (!loaded.ok) throw new Error(loaded.reason);
    },
    ui(world) {
      const scope = og.ui.scope(world, "test");
      const inputs = Object.fromEntries(Object.entries({ kind, disabled: false, busy: false, canRetry: true, rows: [], ...data }).map(([k, v]) => [k, Array.isArray(v) ? scope.collection(k, v, { key: "id" }) : scope.writable(k, v)]));
      const mounted = scope.mount("page", "BaseHost", inputs);
      if (!mounted.ok) throw new Error(mounted.reason);
    },
  }) as never, { root });
  if (!result.ok) throw new Error(result.reason);
  const engine = result.value;
  og.ui.previewViewport(engine.world, { width: 390, height: 844, dpr: 2, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { top: 0, bottom: 0, left: 0, right: 0 } });
  engine.advance(3);
  return { engine, events };
}
function elements(engine: Engine): Element[] { return (engine.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements; }
function click(engine: Engine, element: Element) {
  engine.probe.commands(`0:touch_move:${element.x + element.width / 2},${element.y + element.height / 2};0.02:touch_start;0.06:touch_end`);
  engine.advance(8);
}

test("empty-state action is silent when disabled", () => {
  for (const disabled of [false, true]) {
    const { engine, events } = mount("empty", { disabled });
    try {
      click(engine, elements(engine).find(e => e.id === "Action")!);
      expect(events.filter(e => e.name === "activate").length).toBe(disabled ? 0 : 1);
    } finally { engine.destroy(); }
  }
});
test("retry and dismiss cannot duplicate an in-flight recovery; terminal errors omit retry", () => {
  for (const busy of [false, true]) {
    const { engine, events } = mount("error", { busy });
    try {
      for (const id of ["Retry", "Dismiss"]) click(engine, elements(engine).find(e => e.id === id)!);
      expect(events.filter(e => ["retry", "dismiss"].includes(e.name)).map(e => e.name)).toEqual(busy ? [] : ["retry", "dismiss"]);
    } finally { engine.destroy(); }
  }
  const { engine } = mount("error", { canRetry: false });
  try { expect(elements(engine).some(e => e.id === "Retry")).toBe(false); } finally { engine.destroy(); }
});
test("list-item preserves the enclosing row key and refuses disabled or loading rows", () => {
  const { engine, events } = mount("row", { rows: [{ id: "chosen", title: "Chosen", selected: true }, { id: "locked", title: "Locked", disabled: true }, { id: "pending", title: "Shared build", subtitle: "Retrieving equipment…", loading: true }] });
  try {
    for (const row of elements(engine).filter(e => e.role === "button")) click(engine, row);
    expect(events.filter(e => e.name === "choose").map(e => e.rowKey)).toEqual(["chosen"]);
    expect(elements(engine).some(e => e.name === "Chosen" && e.states.includes("pressed"))).toBe(true);
    const title = elements(engine).find(e => e.text === "Shared build")!;
    const subtitle = elements(engine).find(e => e.text === "Retrieving equipment…")!;
    const spinner = elements(engine).find(e => e.role === "progressbar")!;
    expect(spinner.y).toBeGreaterThanOrEqual(title.y);
    expect(spinner.y + spinner.height).toBeLessThanOrEqual(subtitle.y + subtitle.height);
  } finally { engine.destroy(); }
});
test("steps expose progress in words and return the chosen step key without enabling blocked steps", () => {
  const { engine, events } = mount("steps", { rows: [{ id: "party", label: "Party", number: "1", state: "done" }, { id: "route", label: "Route", number: "2", state: "current" }, { id: "launch", label: "Launch", number: "3", state: "upcoming", disabled: true }] });
  try {
    const buttons = elements(engine).filter(e => e.role === "button");
    for (const button of buttons) click(engine, button);
    expect(events.filter(e => e.name === "choose").map(e => e.rowKey)).toEqual(["party", "route"]);
    const text = elements(engine).map(e => e.text).join(" ");
    expect(text).toContain("Complete"); expect(text).toContain("Current"); expect(text).toContain("Upcoming");
  } finally { engine.destroy(); }
});
