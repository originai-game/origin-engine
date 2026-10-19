// Streaming (phase 3): what the web host hands the engine to stream with.
// The Worker gives the engine the browser's DecompressionStream to inflate
// terrain tiles off the engine's work (worker.ts installInflater): each
// zlib stream comes back inflated under its key, a broken one fails under
// its key (the engine then inflates that tile itself), and without the
// stream the engine is told to inflate on its own.
import { expect, test } from "bun:test";
import { deflateSync } from "node:zlib";
import { installInflater } from "../src/web/worker.ts";

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
