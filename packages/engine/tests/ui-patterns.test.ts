import { expect, test } from "bun:test";

import { og, defineSystem } from "@origingame/engine";

import type * as Api from "@origingame/engine";

import { createEngine, type GameToken } from "../src/runtime.ts";

import { run, byPath, elements, tapOn, systemIds, key } from "./support/patterns.ts";

import "./support/patterns.ts";

// ── puzzle goals ────────────────────────────────────────────────────────

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
