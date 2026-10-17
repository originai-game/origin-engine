// The probe's request ring (docs/design.md §3.4): the shell calls the engine
// Worker without a postMessage hop. Two SharedArrayBuffer byte rings, one for
// request frames (shell → Worker) and one for answer frames (Worker →
// shell), each carrying JSON-RPC 2.0 text framed as a little-endian u32
// length then UTF-8. The Worker drains requests between ticks and answers
// each with `og_engine_call`; the shell wakes on Atomics.notify.
//
// Layout of one ring: Int32 [0] write and [1] read byte counters (monotonic,
// unsigned 32-bit arithmetic), [2] a notify word, then `capacity` data bytes.
// A frame longer than a ring's capacity cannot cross it; `writeFrame`
// answers false and the caller falls back to postMessage.

const HEADER = 16;

/** The shell's and the Worker's halves share these two buffers. */
export interface RpcRings {
  readonly requests: SharedArrayBuffer;
  readonly answers: SharedArrayBuffer;
}

/** Requests are small; answers carry capture PNGs, so they get more room. */
export function createRpcRings(requestBytes = 1 << 20, answerBytes = 16 << 20): RpcRings {
  return {
    requests: new SharedArrayBuffer(HEADER + requestBytes),
    answers: new SharedArrayBuffer(HEADER + answerBytes),
  };
}

function capacity(ring: SharedArrayBuffer): number {
  return ring.byteLength - HEADER;
}

function copyIn(ring: SharedArrayBuffer, at: number, bytes: Uint8Array): void {
  const cap = capacity(ring);
  const data = new Uint8Array(ring, HEADER, cap);
  const start = at % cap;
  const first = Math.min(bytes.length, cap - start);
  data.set(bytes.subarray(0, first), start);
  if (first < bytes.length) data.set(bytes.subarray(first), 0);
}

function copyOut(ring: SharedArrayBuffer, at: number, length: number): Uint8Array {
  const cap = capacity(ring);
  const data = new Uint8Array(ring, HEADER, cap);
  const out = new Uint8Array(length);
  const start = at % cap;
  const first = Math.min(length, cap - start);
  out.set(data.subarray(start, start + first), 0);
  if (first < length) out.set(data.subarray(0, length - first), first);
  return out;
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** Append one frame; false when it does not fit now (or ever: longer than the ring). */
export function writeFrame(ring: SharedArrayBuffer, text: string): boolean {
  const ints = new Int32Array(ring, 0, 4);
  const body = encoder.encode(text);
  const need = 4 + body.length;
  const write = Atomics.load(ints, 0) >>> 0;
  const read = Atomics.load(ints, 1) >>> 0;
  if (need > capacity(ring) - ((write - read) >>> 0)) return false;
  const len = new Uint8Array(4);
  new DataView(len.buffer).setUint32(0, body.length, true);
  copyIn(ring, write, len);
  copyIn(ring, (write + 4) >>> 0, body);
  Atomics.store(ints, 0, (write + need) | 0);
  Atomics.add(ints, 2, 1);
  Atomics.notify(ints, 2);
  return true;
}

/** Every frame written since the last read, oldest first. */
export function readFrames(ring: SharedArrayBuffer): string[] {
  const ints = new Int32Array(ring, 0, 4);
  const write = Atomics.load(ints, 0) >>> 0;
  let read = Atomics.load(ints, 1) >>> 0;
  const out: string[] = [];
  while (read !== write) {
    const length = new DataView(copyOut(ring, read, 4).buffer).getUint32(0, true);
    out.push(decoder.decode(copyOut(ring, (read + 4) >>> 0, length)));
    read = (read + 4 + length) >>> 0;
  }
  Atomics.store(ints, 1, read | 0);
  return out;
}

/** Whether frames are waiting. */
export function hasFrames(ring: SharedArrayBuffer): boolean {
  const ints = new Int32Array(ring, 0, 4);
  return Atomics.load(ints, 0) !== Atomics.load(ints, 1);
}

type WaitAsync = (a: Int32Array, i: number, v: number, t?: number) => { async: boolean; value: Promise<string> | string };

/**
 * Resolves when the ring has frames or `timeoutMs` passed. Uses
 * Atomics.waitAsync where the platform has it (no polling), else a timer.
 */
export function waitForFrames(ring: SharedArrayBuffer, timeoutMs = 50): Promise<void> {
  if (hasFrames(ring)) return Promise.resolve();
  const ints = new Int32Array(ring, 0, 4);
  const waitAsync = (Atomics as unknown as { waitAsync?: WaitAsync }).waitAsync;
  if (waitAsync) {
    const r = waitAsync(ints, 2, Atomics.load(ints, 2), timeoutMs);
    return r.async ? (r.value as Promise<string>).then(() => undefined) : Promise.resolve();
  }
  return new Promise((resolve) => setTimeout(resolve, Math.min(timeoutMs, 4)));
}
