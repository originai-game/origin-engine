import { expect, test } from "bun:test";

import { type RtsCommandsResult, type LedgerResult } from "@origingame/engine/extensions/strategy/index.ts";

import { cityLedger, CITY, rtsRun } from "./support/patterns.ts";

import { elements, tapOn, byPath, key } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

test("ledger: a heading sorts each section by it, amounts high to low first; the same heading again turns the order over; the net is income less expenses", () => {
  // Wrong: lines sorted across sections, the order not turned over, or expenses added to the net.
  const results: LedgerResult[] = [];
  const st = { sort: "", descending: false };
  const engine = cityLedger(st, results);
  engine.advance(10);
  const order = (): string[] => elements(engine).filter((e) => /\/List\/[a-z]+$/.test(e.path) && CITY.some((l) => e.path.endsWith(`/${l.key}`))).sort((a, b) => a.y - b.y).map((e) => e.path.split("/").pop()!);
  expect(order()).toEqual(["res", "com", "tour", "police", "fire", "parks"]);
  tapOn(engine, byPath(engine, "Header/amount")!);
  engine.advance(4);
  expect([st.sort, st.descending]).toEqual(["amount", true]);
  expect(order()).toEqual(["res", "com", "tour", "police", "fire", "parks"]);
  tapOn(engine, byPath(engine, "Header/amount")!);
  engine.advance(4);
  expect([st.sort, st.descending]).toEqual(["amount", false]);
  expect(order()).toEqual(["tour", "com", "res", "parks", "fire", "police"]);
  tapOn(engine, byPath(engine, "Header/change")!);
  engine.advance(4);
  expect(order()).toEqual(["tour", "res", "com", "parks", "police", "fire"]);
  expect(elements(engine).find((e) => e.path.endsWith("Net/Value"))?.text).toBe("+5,800");
});

test("ledger: a line's budget slider answers its key and value (not a press on the line); a locked one answers no budget; a press on a line selects it", () => {
  // Wrong: the slider's move lost or answered under another line, a locked policy moved, a line's press not answered.
  const results: LedgerResult[] = [];
  const engine = cityLedger({ sort: "", descending: false }, results);
  engine.advance(10);
  const end = (key: string): string => {
    const el = elements(engine).find((e) => e.path.includes(`/List/${key}/`) && e.path.endsWith("/Funding"))!;
    return `${Math.round(el.x + el.width - 2)},${Math.round(el.y + el.height / 2)}`;
  };
  for (const k of ["police", "fire"]) {
    engine.probe.commands(`0:touch_move:${end(k)};0.05:touch_start;0.1:touch_end`);
    engine.advance(10);
  }
  const moves = results.filter((r) => r.funding).map((r) => r.funding!);
  expect(moves.length > 0 && moves.every((m) => m.key === "police")).toBeTrue();
  expect(moves[moves.length - 1]!.value).toBe(150);
  tapOn(engine, byPath(engine, "List/tour/Main/Name")!);
  const picked = results.filter((r) => r.selected).map((r) => r.selected);
  expect([picked.includes("police"), picked[picked.length - 1]]).toEqual([false, "tour"]);
});

test("rtsCommands: a command naming a page opens it, whose Back returns; a locked command is refused by press or key; Shift queues", () => {
  // Wrong: Build answered as an order, a sub-page with no way back, a locked command fired by its hotkey, or Shift ignored.
  const results: RtsCommandsResult[] = [];
  const st = { page: "", targeting: "" };
  const engine = rtsRun(st, results);
  tapOn(engine, byPath(engine, "Commands/Grid/Row#0/build")!);
  expect(st.page).toBe("build");
  expect(results.some((r) => r.command === "build")).toBeFalse();
  tapOn(engine, byPath(engine, "PageBack")!);
  expect(st.page).toBe("root");
  key(engine, "KeyS");
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["storm"]);
  engine.probe.commands("0:key_down:ShiftLeft;0.05:key_down:KeyA;0.1:key_up:KeyA;0.15:key_up:ShiftLeft");
  engine.advance(12);
  expect(results.filter((r) => r.command).map((r) => [r.command, r.queued])).toEqual([["attack", true]]);
});

test("rtsCommands: a group's key recalls it and twice quickly focuses it; Ctrl sets and Shift adds; the prompt names a command from another page; Back cancels it", () => {
  // Wrong: Ctrl+1 recalling instead of setting, a slow second tap focusing, or Back leaving the targeting order in place.
  const results: RtsCommandsResult[] = [];
  const st = { page: "", targeting: "" };
  const engine = rtsRun(st, results);
  key(engine, "Digit1");
  engine.advance(60);
  key(engine, "Digit1");
  key(engine, "Digit1");
  engine.probe.commands("0:key_down:ControlLeft;0.05:key_down:Digit1;0.1:key_up:Digit1;0.15:key_up:ControlLeft");
  engine.advance(12);
  engine.probe.commands("0:key_down:ShiftLeft;0.05:key_down:Digit2;0.1:key_up:Digit2;0.15:key_up:ShiftLeft");
  engine.advance(12);
  const got = (k: keyof RtsCommandsResult) => results.filter((r) => r[k]).map((r) => r[k]);
  expect([got("recall"), got("focus"), got("assign"), got("add")]).toEqual([["g1", "g1"], ["g1"], ["g1"], ["g2"]]);
  st.targeting = "barracks";
  engine.advance(4);
  expect(elements(engine).find((e) => e.path.endsWith("RtsPrompt/Text"))?.text).toBe("Barracks: choose a target");
  key(engine, "Escape");
  expect(results.some((r) => r.cancelTarget)).toBeTrue();
});
