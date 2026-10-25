// The web host's decoder Worker (src/web/audio-decoder.ts) against the
// engine's own QOA decoder: a voice that reaches a frame before the Worker
// answered decodes it in the engine, so the two must give the same samples
// or the mix would depend on the Worker's timing. og-audio's
// `tests/decode.rs` decodes the same fixture (written by og's encoder:
// full-scale squares with noise, a sweep, silence, impulses; two full frames
// and a short one, stereo) to the same hash, whole and frame by frame.
import { expect, test } from "bun:test";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { AudioDecoder, decodeQoa, decoderSource, type DecodeReply, type DecodeRequest } from "../src/web/audio-decoder.ts";

declare const Bun: {
  build(o: { entrypoints: string[]; target: string; format: string; minify?: boolean }): Promise<{ success: boolean; logs: unknown[]; outputs: Array<{ text(): Promise<string> }> }>;
};

const here = dirname(fileURLToPath(import.meta.url));
const fixture = new Uint8Array(readFileSync(join(here, "fixtures", "qoa", "frames.qoa")));
/** og-audio tests/decode.rs FIXTURE_HASH. */
const FIXTURE_HASH = 0x080bf5513188bc60n;

/** FNV-1a over the samples' little-endian bytes. */
function fnv(samples: Int16Array): bigint {
  let h = 0xcbf29ce484222325n;
  const bytes = new Uint8Array(samples.buffer, samples.byteOffset, samples.byteLength);
  for (const b of bytes) h = ((h ^ BigInt(b)) * 0x100000001b3n) & 0xffffffffffffffffn;
  return h;
}

/** The file's frames as the bank's jobs carry them: `qoaf`, the frame's length, the frame. */
function oneFrameFiles(file: Uint8Array): Uint8Array[] {
  const view = new DataView(file.buffer, file.byteOffset, file.byteLength);
  const out: Uint8Array[] = [];
  for (let pos = 8; pos < file.byteLength; ) {
    const len = view.getUint16(pos + 4);
    const size = view.getUint16(pos + 6);
    const one = new Uint8Array(8 + size);
    const v = new DataView(one.buffer);
    v.setUint32(0, 0x716f6166);
    v.setUint32(4, len);
    one.set(file.subarray(pos, pos + size), 8);
    out.push(one);
    pos += size;
  }
  return out;
}

function concat(parts: Int16Array[]): Int16Array {
  const out = new Int16Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

test("decodeQoa gives the engine's samples, whole and frame by frame", () => {
  const whole = decodeQoa(fixture)!;
  expect(whole.length).toBe(12_000 * 2);
  expect(fnv(whole)).toBe(FIXTURE_HASH);
  const files = oneFrameFiles(fixture);
  expect(files.length).toBe(3);
  expect(fnv(concat(files.map((f) => decodeQoa(f)!)))).toBe(FIXTURE_HASH);
  // Malformed: truncated, wrong magic, a frame size that does not match.
  expect(decodeQoa(fixture.subarray(0, fixture.length - 1))).toBeNull();
  const magic = fixture.slice();
  magic[0] = 0;
  expect(decodeQoa(magic)).toBeNull();
  const sized = files[0]!.slice();
  sized[15] ^= 1;
  expect(decodeQoa(sized)).toBeNull();
});

test("the decoder Worker's source decodes the same after the export's minifying build", async () => {
  // The build bundles worker.ts with Bun.build({ minify }) (src/node/web-build.ts);
  // decoderSource() stringifies decodeQoa there, so it must survive minifying.
  const r = await Bun.build({ entrypoints: [join(here, "..", "src", "web", "audio-decoder.ts")], target: "browser", format: "esm", minify: true });
  expect(r.success).toBe(true);
  const dir = mkdtempSync(join(tmpdir(), "og-decoder-"));
  try {
    const bundle = join(dir, "decoder.mjs");
    writeFileSync(bundle, await r.outputs[0]!.text());
    const built = (await import(bundle)) as { decoderSource(): string };
    const source = built.decoderSource();
    expect(source).not.toBe(decoderSource());
    // Run it as the Worker would: a request in, a reply out.
    const replies: DecodeReply[] = [];
    const self: { onmessage?: (e: { data: DecodeRequest }) => void; postMessage(m: DecodeReply): void } = { postMessage: (m) => void replies.push(m) };
    new Function("self", source)(self);
    const files = oneFrameFiles(fixture);
    self.onmessage!({ data: { keys: [7, 8, 9], files } });
    expect(replies[0]!.keys).toEqual([7, 8, 9]);
    expect(fnv(concat(replies[0]!.samples.map((s) => s!)))).toBe(FIXTURE_HASH);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("AudioDecoder feeds the bank's jobs to a Worker and answers the bank", async () => {
  const files = oneFrameFiles(fixture);
  // A bank with three jobs; it takes each answer once.
  let queue: unknown[] = files.flatMap((f, i) => [100 + i, i, f]);
  const put = new Map<number, Int16Array>();
  const asked: number[] = [];
  const glue = {
    og_web_audio_decode_jobs(engine: number, max: number): unknown[] {
      expect(engine).toBe(3);
      asked.push(max);
      const out = queue.slice(0, max * 3);
      queue = queue.slice(max * 3);
      return out;
    },
    og_web_audio_put_decoded(engine: number, key: number, samples: Int16Array): boolean {
      expect(engine).toBe(3);
      if (put.has(key)) return false;
      put.set(key, samples);
      return true;
    },
  };
  const url = URL.createObjectURL(new Blob([decoderSource()], { type: "text/javascript" }));
  const decoder = AudioDecoder.start(glue, 3, () => new Worker(url))!;
  expect(decoder).not.toBeNull();
  expect(decoder.feed()).toBe(3);
  expect(decoder.inFlight).toBe(3);
  expect(asked).toEqual([32]);
  expect(decoder.feed()).toBe(0);
  for (let i = 0; i < 200 && decoder.decoded < 3; i++) await new Promise((r) => setTimeout(r, 5));
  expect(decoder.decoded).toBe(3);
  expect(decoder.inFlight).toBe(0);
  expect(fnv(concat([100, 101, 102].map((k) => put.get(k)!)))).toBe(FIXTURE_HASH);
  decoder.close();
  URL.revokeObjectURL(url);
  // A module without the decode members: no decoder, the engine decodes as it mixes.
  expect(AudioDecoder.start({}, 3)).toBeNull();
});
