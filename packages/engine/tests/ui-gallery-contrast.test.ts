// The gallery's contrast pass reads a text's ground from the frame: ink drawn on a ground it was
// not chosen for (a pack's dark art under a component's dark ink) is listed, the same text in a
// readable ink is not, a disabled control's text is exempt and a line cut away is not read.
import { expect, test } from "bun:test";
import { lowContrast } from "../src/node/gallery.ts";

/** A 40x20 frame of `ground` with a glyph-like bar of `ink` across the middle rows. */
function frame(ground: number[], ink: number[]) {
  const width = 40, height = 20, rgba = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    const c = y >= 8 && y < 12 && x % 3 !== 0 ? ink : ground;
    rgba.set([c[0]!, c[1]!, c[2]!, 255], (y * width + x) * 4);
  }
  return { width, height, rgba };
}

const navy = [0x16, 0x3f, 0x92];
const dump = (ink: number[], disabled = false, clip?: number[]) => ({
  elements: [
    { id: "1", parent: "", disabled },
    { id: "2", parent: "1", text: "Equipped", ink: [...ink, 1], fontSize: 12, textLines: [[0, 0, 20, 10]], ...(clip ? { clip } : {}) },
  ],
});

test("dark ink on dark art is listed, with its ratio and the bar", () => {
  const gold = [0x24, 0x1a, 0x04];
  const low = lowContrast(dump(gold), frame(navy, gold), 2);
  expect(low).toHaveLength(1);
  expect(low[0]).toMatch(/^"Equipped" #241a04 on #163f92 1\.\d\d:1 \(under 4\.5:1\)$/);
});

test("the pack's art ink reads, and a disabled control's text is exempt", () => {
  const white = [255, 255, 255];
  expect(lowContrast(dump(white), frame(navy, white), 2)).toEqual([]);
  const gold = [0x24, 0x1a, 0x04];
  expect(lowContrast(dump(gold, true), frame(navy, gold), 2)).toEqual([]);
});

test("a line its ancestors' overflow cuts away is not read; one inside the cut still is", () => {
  const gold = [0x24, 0x1a, 0x04];
  // A strip clipped to x 0..10 holds its line at x 12..20 (a compass point slid out of view).
  const out = { elements: [{ id: "2", parent: "", text: "SE", ink: [...gold, 1], fontSize: 12, textLines: [[12, 0, 8, 10]], clip: [0, 0, 10, 10] }] };
  expect(lowContrast(out, frame(navy, gold), 2)).toEqual([]);
  expect(lowContrast(dump(gold, false, [0, 0, 20, 10]), frame(navy, gold), 2)).toHaveLength(1);
});
