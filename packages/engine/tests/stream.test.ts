// Streaming (phase 3): what the web host hands the engine to stream with.
// The Worker gives the engine the browser's DecompressionStream to inflate
// terrain tiles off the engine's work (boot.ts installInflater): each
// zlib stream comes back inflated under its key, a broken one fails under
// its key (the engine then inflates that tile itself), and without the
// stream the engine is told to inflate on its own.
import { expect, test } from "bun:test";
import { deflateSync } from "node:zlib";
import { installInflater, storedTurn } from "../src/web/boot.ts";

function host() {
  const put = new Map<number, Uint8Array>();
  const failed = new Map<number, string>();
  let inflate: ((key: number, bytes: Uint8Array) => void) | null = null;
  const glue = {
    og_web_set_inflater: (f: (key: number, bytes: Uint8Array) => void) => {
      inflate = f;
    },
    og_web_put_inflated: (key: number, bytes: Uint8Array) => void put.set(key, bytes),
    og_web_fail_inflated: (key: number, reason: string) => void failed.set(key, reason),
  };
  return { glue, put, failed, inflate: (k: number, b: Uint8Array) => inflate!(k, b) };
}

async function settled(done: () => boolean): Promise<void> {
  for (let i = 0; i < 200 && !done(); i++) await new Promise((r) => setTimeout(r, 5));
}

test("the Worker's inflater answers each zlib stream inflated under its key", async () => {
  const h = host();
  expect(installInflater(h.glue)).toBeTrue();
  const a = new Uint8Array(131 * 131 * 2).map((_, i) => (i * 7) & 255);
  const b = new Uint8Array(34 * 34 * 4).fill(200);
  h.inflate(7, new Uint8Array(deflateSync(a)));
  h.inflate(8, new Uint8Array(deflateSync(b)));
  h.inflate(9, new Uint8Array([120, 156, 1, 2, 3]));
  await settled(() => h.put.size + h.failed.size >= 3);
  expect(h.put.get(7)).toEqual(a);
  expect(h.put.get(8)).toEqual(b);
  expect(h.put.has(9)).toBeFalse();
  expect(h.failed.has(9)).toBeTrue();
});

test("without DecompressionStream the engine inflates on its own", () => {
  const saved = (globalThis as { DecompressionStream?: unknown }).DecompressionStream;
  (globalThis as { DecompressionStream?: unknown }).DecompressionStream = undefined;
  try {
    expect(installInflater(host().glue)).toBeFalse();
  } finally {
    (globalThis as { DecompressionStream?: unknown }).DecompressionStream = saved;
  }
});

// A remote image's texels, which createImageBitmap has turned upright, go
// back as its file stores them (installRemoteFetcher), so the texture's
// EXIF orientation means on the web what it means natively. The truth is
// og-draw's UiOrientation (crates/og-draw/src/ui/mod.rs): the turn a
// displayed point takes back to the stored one, bit 2 swapping the axes
// first, then bit 0 flipping x and bit 1 flipping y. A wrong matrix for one
// value (a quarter turn the wrong way, 5 and 7 swapped) maps some pixel of
// the asymmetric 3 x 2 image to another stored texel.
test("storedTurn puts each upright texel back where its file stores it, for every EXIF orientation", () => {
  const bits: Record<number, number> = { 1: 0, 2: 0b001, 3: 0b011, 4: 0b010, 5: 0b100, 6: 0b110, 7: 0b111, 8: 0b101 };
  const [uw, uh] = [3, 2];
  for (let exif = 1; exif <= 8; exif++) {
    const b = bits[exif]!;
    const t = storedTurn(exif, uw, uh);
    expect([t.width, t.height]).toEqual(b & 0b100 ? [uh, uw] : [uw, uh]);
    const [a, bb, c, d, e, f] = t.matrix;
    for (let uy = 0; uy < uh; uy++) {
      for (let ux = 0; ux < uw; ux++) {
        const [px, py] = [(ux + 0.5) / uw, (uy + 0.5) / uh];
        let [x, y] = b & 0b100 ? [py, px] : [px, py];
        if (b & 0b001) x = 1 - x;
        if (b & 0b010) y = 1 - y;
        const want = [x * t.width, y * t.height];
        const got = [a * (ux + 0.5) + c * (uy + 0.5) + e, bb * (ux + 0.5) + d * (uy + 0.5) + f];
        expect(got.map((v) => Math.round(v * 1000) / 1000)).toEqual(want.map((v) => Math.round(v * 1000) / 1000));
      }
    }
  }
  expect(storedTurn(0, 3, 2)).toEqual({ width: 3, height: 2, matrix: [1, 0, 0, 1, 0, 0] });
});
