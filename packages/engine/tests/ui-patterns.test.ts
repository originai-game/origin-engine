import { expect, test } from "bun:test";

import { og, defineSystem } from "@origingame/engine";

import type * as Api from "@origingame/engine";

import { createEngine, type GameToken } from "../src/runtime.ts";

import { run, byPath, centre, elements, tapOn, systemIds, key } from "./support/patterns.ts";

import "./support/patterns.ts";

test("heroButton: a press is a click; disabled it is inert; a desktop shows its keycap and says it", () => {
  // Wrong: a disabled call to action still answers clicked, or the keycap claims a key the name never says.
  let clicked = 0;
  let off = 0;
  const e = run(() => {
    if (og.ui.heroButton("Go", "End turn", { tone: "primary", key: "E" }).clicked) clicked += 1;
    if (og.ui.heroButton("Wait", "Waiting", { disabled: true }).clicked) off += 1;
  }, {});
  const go = byPath(e, "Root/Go")!;
  expect(go.name).toBe("End turn, E");
  expect(byPath(e, "Go/Key/Text")!.text).toBe("E");
  for (const b of [go, byPath(e, "Root/Wait")!]) {
    e.probe.commands(`0:touch_move:${centre(b)};0.05:touch_start;0.1:touch_end`);
    e.advance(10);
  }
  expect(clicked).toBe(1);
  expect(off).toBe(0);
});

test("heroButton: a label longer than its row wraps inside the button instead of running past the row", () => {
  // Wrong: a one-line label sets the button's least width, so a narrow row (a phone at 150 %) is overrun.
  const e = run(() => og.ui.column("Box", { width: { kind: "fixed", px: 150 } }, () => og.ui.heroButton("Go", "Reroll the rewards for fifty gold")), {});
  const box = byPath(e, "Root/Box")!;
  const go = byPath(e, "Box/Go")!;
  const label = byPath(e, "Go/Label")!;
  expect(go.x + go.width).toBeLessThanOrEqual(box.x + box.width + 0.5);
  expect(label.x + label.width).toBeLessThanOrEqual(go.x + go.width + 0.5);
  expect(label.text).toBe("Reroll the rewards for fifty gold");
});

// ── puzzle goals ────────────────────────────────────────────────────────

test("starBar: a star is earned at its threshold and reported crossed once, not every tick after", () => {
  // Wrong: crossed reported on every tick while over the line (the game's fanfare looping), or a star lit below it.
  let score = 900;
  const results: Api.StarBarResult[] = [];
  const engine = run(() => {
    results.push(og.ui.starBar("Stars", { value: score, thresholds: [1000, 2000, 3000], label: "Score" }));
  });
  expect(results.at(-1)!.earned).toBe(0);
  score = 2100;
  engine.advance(10);
  expect(results.at(-1)!.earned).toBe(2);
  expect(results.reduce((n, r) => n + r.crossed, 0)).toBe(2);
  const meter = elements(engine).find((e) => e.role === "meter" && e.path.endsWith("Stars"))!;
  expect(meter.name).toBe("Score");
  expect(meter.value).toBe("2,100");
});

test("overlayBar: a press shows an overlay and its legend; a press on the shown one turns it off", () => {
  // Wrong: a second press on the shown overlay showing it again, or a legend with no overlay shown.
  const results: Api.OverlayBarResult[] = [];
  const st = { selected: "" };
  const engine = run(() => {
    const r = og.ui.overlayBar("Lenses", {
      overlays: [
        { key: "traffic", name: "Traffic", ramp: ["#2ecc71", "#f1c40f", "#e74c3c"], low: "Free", high: "Jammed" },
        { key: "power", name: "Power", ramp: ["#222244", "#ffd23f"], low: "None", high: "Full" },
      ],
      selected: st.selected,
    });
    results.push(r);
    if (r.selected !== null) st.selected = r.selected;
  });
  engine.advance(10);
  expect(byPath(engine, "Lenses/Legend") === undefined).toBeTrue();
  tapOn(engine, byPath(engine, "Row/power")!);
  engine.advance(4);
  expect(st.selected).toBe("power");
  expect(elements(engine).find((e) => e.path.endsWith("Legend/Name"))?.text).toBe("Power");
  tapOn(engine, byPath(engine, "Row/power")!);
  engine.advance(4);
  expect(st.selected).toBe("");
  expect(byPath(engine, "Lenses/Legend") === undefined).toBeTrue();
});

// ── settings tree ───────────────────────────────────────────────────────

test("settingsTree: entries are data (a game's own row among them); Apply answers only with changes pending; Back during the countdown reverts; the HUD page mounts the HUD layout editor", () => {
  // Wrong: an entry the pattern does not know dropped, Apply live with nothing pending, Back during the countdown leaving the screen with the display change kept, or the HUD page drawing its own copy of the editor.
  const results: Api.SettingsTreeResult[] = [];
  const st = { page: "graphics", fullscreen: false, pending: false, confirm: 0, customs: [] as string[] };
  const sys = defineSystem(`Settings${systemIds.next++}`, {}, (_q: unknown, _t: unknown, world: Api.World) => {
    og.ui.anchor("Root", { anchor: "topLeft", direction: "column", padding: { left: 24, top: 24, right: 24, bottom: 24 } } as never, () => {
      og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
        const r = og.ui.settingsTree(world, "Settings", {
          pages: [
            { key: "graphics", label: "Graphics", entries: [
              { key: "fullscreen", kind: "toggle", label: "Fullscreen", on: st.fullscreen, display: true, dirty: st.pending },
              { key: "scale", kind: "slider", label: "Render scale", value: 1, min: 0.5, max: 1, step: 0.05, unit: "%" },
              { key: "gamma", kind: "custom", label: "Brightness" },
            ] },
            { key: "controls", label: "Controls", entries: [{ key: "jump", kind: "binding", label: "Jump", action: "jump" }] },
            { key: "hud", label: "HUD", hud: true },
          ],
          page: st.page,
          pending: st.pending,
          confirmSeconds: st.confirm,
        }, (key) => {
          st.customs.push(key);
          og.ui.label("Mine", "custom", {});
        });
        results.push(r);
        if (r.page) st.page = r.page;
        if (r.change?.key === "fullscreen") {
          st.fullscreen = r.change.on;
          st.pending = true;
        }
        if (r.apply) {
          st.pending = false;
          st.confirm = 15;
        }
        if (r.confirmRevert) {
          st.fullscreen = false;
          st.confirm = 0;
        }
      });
    });
  });
  const made = createEngine(og.game({ systems: [sys], setup: (world: Api.World) => og.input.defineAction(world, { name: "jump", label: "Jump", bindings: [{ control: "Space" }] }) } as never) as unknown as GameToken, {});
  if (!made.ok) throw new Error(made.reason);
  const engine = made.value;
  engine.advance(10);
  expect(st.customs.at(-1)).toBe("gamma");
  expect(byPath(engine, "Rows/gamma/Control/Mine")?.text).toBe("custom");
  tapOn(engine, byPath(engine, "Foot/Apply")!);
  expect(results.some((r) => r.apply)).toBeFalse();
  tapOn(engine, byPath(engine, "Rows/fullscreen/Control/Switch")!);
  expect(st.fullscreen).toBeTrue();
  tapOn(engine, byPath(engine, "Foot/Apply")!);
  expect(st.confirm).toBe(15);
  expect(byPath(engine, "Confirm/Buttons/Keep")).toBeDefined();
  key(engine, "Escape");
  expect(st.fullscreen).toBeFalse();
  expect(st.confirm).toBe(0);
  tapOn(engine, byPath(engine, "Tabs/Row/controls")!);
  expect(elements(engine).some((e) => e.path.includes("Rows/jump/Control/Binding"))).toBeTrue();
  tapOn(engine, byPath(engine, "Tabs/Row/hud")!);
  tapOn(engine, byPath(engine, "Hud/Edit")!);
  engine.advance(4);
  expect(byPath(engine, "Hud/Editor/Actions/Done")).toBeDefined();
});
