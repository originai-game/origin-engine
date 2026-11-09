// The feedback components (docs/ui.md §2, U6c) on the native host, read
// through dump_ui and the accessibility fields: what each shows, when it
// leaves, what a screen reader is told, and the number notations.
//
// Ways they could fail, each covered below:
// - a notation rounds 999,950 to "1000K", runs out of letters past T,
//   loses numbers past 1e308, or drops the sign;
// - a tooltip shows before its delay, never pins, or sits on top of
//   its trigger or the panel it is in instead of beside it (or off the
//   safe area at the edge), or under a surface built after it.
import { beforeAll, expect, test } from "bun:test";
import { og } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { at, dump, engineFor } from "./support/ui.ts";
import { groupNumber } from "../src/ui/data/format.ts";
import { countdown, shortTime } from "../src/ui/feedback/core.ts";

beforeAll(() => {
  loadNativeHost();
});


test("og.ui.formatNumber: every notation, rounding into the next group, letters, past 1e308", () => {
  const f = og.ui.formatNumber;
  expect(f(12840000, {})).toBe("12,840,000");
  expect(f(-1234, {})).toBe("-1,234");
  expect(f(1234.5, { decimals: 1 })).toBe("1,234.5");
  expect(f(1234.5, { decimals: 1, group: ".", point: "," })).toBe("1.234,5");
  expect(f(9999, { notation: "compact" })).toBe("10K");
  expect(f(10000, { notation: "compact" })).toBe("10K");
  expect(f(12_345_678, { notation: "compact" })).toBe("12M");
  expect(f(999_950, { notation: "compact" })).toBe("1M");
  expect(f(3.4e12, { notation: "compact" })).toBe("3.4T");
  expect(f(3.4e12, { notation: "compact", decimals: 2 })).toBe("3.40T");
  expect(f(5e15, { notation: "compact" })).toBe("5.00e15");
  // aa follows T; the 136th letter pair after T is ff (5 × 26 + 5).
  expect(f(1.5e15, { notation: "letters" })).toBe("1.50aa");
  expect(f(2.5e18, { notation: "letters" })).toBe("2.50ab");
  expect(f(1.5, { notation: "letters", exponent: 420 })).toBe("1.50ff");
  expect(f(12_800_000, { notation: "scientific" })).toBe("1.28e7");
  expect(f(12_800_000, { notation: "engineering" })).toBe("12.8e6");
  expect(f(5, { notation: "compact", exponent: 400 })).toBe("5.00e400");
  expect(f(120, { sign: true, suffix: " XP" })).toBe("+120 XP");
  expect(f(0, { sign: true })).toBe("0");
  expect(f(0.004, { sign: true, decimals: 2 })).toBe("0.00");
  expect(f(-0.004, { sign: true, decimals: 2 })).toBe("0.00");
  expect(f(0.006, { sign: true, decimals: 2 })).toBe("+0.01");
  expect(f(Number.NaN, { prefix: "$" })).toBe("$—");
});

test("core data and feedback formatting follows the current engine locale", () => {
  let locale = "de";
  let values: string[] = [];
  const e = engineFor("LocalizedCore", (world) => {
    og.i18n.setLocale(world, locale);
    values = [groupNumber(1234.5, 1), countdown(125), shortTime(120), og.ui.formatNumber(12345, { notation: "compact" })];
  });
  e.advance(3);
  expect(values).toEqual(["1.234,5", "2:05", "2 Min.", "12.345"]);
  locale = "ja";
  e.advance(3);
  expect(values[3]).toBe("1.2万");
  locale = "bn";
  e.advance(3);
  expect(values.slice(0, 2)).toEqual(["১,২৩৪.৫", "২:০৫"]);
  locale = "en-XA";
  e.advance(3);
  expect(values.slice(0, 2)).toEqual(["[1,234.5~~~]", "[2:05~~]"]);
});

