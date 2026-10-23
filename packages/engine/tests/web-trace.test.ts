// The web host's trace (src/web/worker.ts `trace`, the shell's `trace()`):
// the engine Worker buffers the run's spans in the Chrome trace format
// (src/trace.ts, the document `og run --trace` writes natively) and the
// shell reads it. The real Worker and shell, joined by a stand-in for the
// Worker's message port, with the native host bound in place of the wasm
// (tests/fixtures/web-glue/native.ts maps og_web_trace* to the same
// og-runtime trace). No browser.
//
// The ways this could fail, each asserted below: the shell has no trace or
// the Worker no `trace` call (the web had timings and no trace file); a
// trace started at the boot misses the boot scene's loads or the first
// ticks; ticks taken by `advance` between frames are lost; spans are laid
// end to end, or nest wrongly (checkTrace); the document is not the
// native one (its threads, its categories, a system per tick per system);
// `stop` leaves it recording.
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem } from "../src/index.ts";
import { loadNativeHost } from "../src/node/index.ts";
import type { GameToken } from "../src/runtime.ts";
import { checkTrace, type TraceDocument, type TraceEvent } from "../src/trace.ts";
import { Shell } from "../src/web/shell.ts";
import { startWorker } from "../src/web/worker.ts";

/** Anything a DOM call asks for: every member is a callable that answers another. */
function anything(): any {
  const f = () => {};
  return new Proxy(f, {
    get: (_, key) => (key === Symbol.toPrimitive ? () => "" : key === "then" ? undefined : anything()),
    set: () => true,
    apply: () => anything(),
  });
}

function canvas(): HTMLCanvasElement {
  const target = new EventTarget();
  return Object.assign(target, {
    clientWidth: 320,
    clientHeight: 180,
    width: 0,
    height: 0,
    ownerDocument: anything(),
    transferControlToOffscreen: () => ({ offscreen: true }),
    getBoundingClientRect: () => ({ left: 0, top: 0, width: 320, height: 180 }),
  }) as unknown as HTMLCanvasElement;
}

type Scope = { onmessage: ((e: { data: unknown }) => void) | null; postMessage(m: unknown): void };
const scope = globalThis as unknown as Scope;
const saved = { postMessage: scope.postMessage, onmessage: scope.onmessage, console: { ...console } };

/** The Worker as the shell holds it, its other end being this process's global scope. */
class Port extends EventTarget {
  terminated = false;
  constructor() {
    super();
    scope.postMessage = (m: unknown) => setTimeout(() => !this.terminated && this.dispatchEvent(new MessageEvent("message", { data: m })), 0);
  }
  postMessage(m: unknown): void {
    setTimeout(() => !this.terminated && scope.onmessage?.({ data: m }), 0);
  }
  terminate(): void {
    this.terminated = true;
  }
}

beforeAll(() => loadNativeHost());

const Spin = defineSystem("Spin", { writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const r = a.column(og.Transform).quaternion;
    for (let i = 0; i < a.count; i++) r[i * 4 + 1] = Math.sin(time.elapsed);
  }
});
const Count = defineSystem("Count", { reads: [og.Transform], after: ["Spin"] }, () => {
  let x = 0;
  for (let i = 0; i < 5_000; i++) x = (x * 31 + i) % 65_521;
  if (x < 0) throw new Error("never");
});

test("the web host records the run's trace in the Worker and the shell reads it, as og run --trace writes it", async () => {
  startWorker(
    og.game({
      start(world) {
        for (let i = 0; i < 8; i++) world.spawn({ name: `box-${i}`, position: { x: i, y: 0, z: 0 } });
      },
      systems: [Spin, Count],
    }) as unknown as GameToken,
  );
  const shell = await Shell.start({
    canvas: canvas(),
    worker: new Port() as unknown as Worker,
    glue: new URL("./fixtures/web-glue/native.ts", import.meta.url).href,
    wasm: "unused.wasm",
    autoplay: false,
    trace: true,
  });
  try {
    await shell.call("advance", 20);
    for (let i = 0; i < 5; i++) await shell.call("frame", 16.67);
    await shell.call("advance", 10);
    const doc = (await shell.trace()) as TraceDocument;
    expect(doc).not.toBeNull();
    expect(doc.otherData.producer).toBe("og web trace");
    const check = checkTrace(doc);
    expect(check.errors).toEqual([]);
    // The native document's threads, named once each.
    for (const name of ["engine", "render", "load", "cook", "gpu"]) expect(Object.values(check.threads)).toContain(name);
    const spans = doc.traceEvents.filter((e) => e.ph === "X") as (TraceEvent & { cat: string; ts: number; dur: number; args: { tick?: number } })[];
    const ticks = spans.filter((s) => s.cat === "tick");
    // Every tick from the first: the advances' and the frames' (one tick each at 60 Hz).
    const numbers = ticks.map((t) => t.args.tick!).sort((a, b) => a - b);
    expect(numbers[0]).toBe(1);
    expect(numbers.length).toBeGreaterThanOrEqual(30);
    expect(new Set(numbers).size).toBe(numbers.length);
    // Each tick ran Spin and Count, inside it, Count after Spin.
    for (const t of ticks) {
      const inside = spans.filter((s) => s.cat === "system" && s.args.tick === t.args.tick);
      const spin = inside.find((s) => s.name === "Spin");
      const count = inside.find((s) => s.name === "Count");
      expect(spin && count ? count.ts >= spin.ts + spin.dur && spin.ts >= t.ts && count.ts + count.dur <= t.ts + t.dur : false).toBe(true);
    }
    // stop: nothing more is recorded; start: a fresh document from then on.
    expect(await shell.trace("stop")).toBe(true);
    expect(await shell.trace()).toBeNull();
    expect(await shell.trace("start")).toBe(true);
    await shell.call("advance", 3);
    const again = (await shell.trace()) as TraceDocument;
    expect(again.traceEvents.filter((e) => e.ph === "X" && (e as { cat?: string }).cat === "tick").length).toBe(3);
  } finally {
    shell.dispose();
    scope.postMessage = saved.postMessage;
    scope.onmessage = saved.onmessage;
    Object.assign(console, saved.console);
  }
});
