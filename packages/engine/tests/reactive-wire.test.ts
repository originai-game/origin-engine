import { expect, test } from "bun:test";
import { decodeUiPatch, encodeUiPatch } from "../src/ui/reactive-wire.ts";

test("UI wire preserves tags, little-endian numbers and sorted UTF-8 record keys", () => {
  const encoded = encodeUiPatch({ z: [null, false, true, 1.5], a: "é😀" });
  expect(encoded.ok).toBe(true);
  if (!encoded.ok) return;
  expect([...encoded.value]).toEqual([
    6, 2, 0, 0, 0,
    1, 0, 0, 0, 97, 4, 6, 0, 0, 0, 195, 169, 240, 159, 152, 128,
    1, 0, 0, 0, 122, 5, 4, 0, 0, 0, 0, 1, 2, 3, 0, 0, 0, 0, 0, 0, 248, 63,
  ]);
  const large = { label: "界😀\ud800".repeat(300), values: Array.from({ length: 300 }, (_, i) => i * 1.25 - 7), ids: ["ped-7\u007f", "ped-7\u0080é", ""] };
  const grown = encodeUiPatch(large);
  expect(grown.ok).toBe(true);
  if (grown.ok) expect(decodeUiPatch(grown.value)).toEqual({ ok: true, value: { ...large, label: "界😀�".repeat(300) } });
});

test("UI wire accepts the byte limit exactly and refuses oversized, deep, cyclic and non-value inputs", () => {
  const limit = 16 * 1024 * 1024;
  for (const suffix of ["", "é", "😀"]) {
    const size = new TextEncoder().encode(suffix).length;
    const value = "x".repeat(limit - 5 - size) + suffix;
    const encoded = encodeUiPatch(value);
    expect(encoded.ok).toBe(true);
    if (encoded.ok) {
      expect(encoded.value.length).toBe(limit);
      expect(decodeUiPatch(encoded.value)).toEqual({ ok: true, value });
    }
    expect(encodeUiPatch(value + "x").ok).toBe(false);
  }
  let nested: unknown = null;
  for (let i = 0; i < 64; i++) nested = [nested];
  expect(encodeUiPatch(nested).ok).toBe(true);
  expect(encodeUiPatch([nested]).ok).toBe(false);
  const cyclic: unknown[] = [];
  cyclic.push(cyclic);
  for (const value of [cyclic, NaN, Infinity, () => {}, Promise.resolve(1), new Date(), { get field() { throw new Error("must not invoke"); } }]) {
    expect(encodeUiPatch(value).ok).toBe(false);
  }
  const shared = { n: 3 };
  expect(encodeUiPatch([shared, shared]).ok).toBe(true);
});
