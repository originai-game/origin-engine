// A throw in GameDesc.start on the web: the engine Worker (src/web/worker.ts)
// and the main-thread shell (src/web/shell.ts) as the play page wires them
// (shell/page.ts: autoplay off, one priming tick behind the gate, and with
// autoplay on as `og dev` plays), joined by a stand-in for the Worker's
// message port, with the native host bound in place of the wasm. No browser:
// a stand-in canvas and document take the shell's DOM calls.
//
// The ways this could fail, each asserted below: the refusal stops at the
// Worker (no `crash` message, so the page shows nothing and keeps waiting);
// it arrives as a start `error` the shell rejects with no card (wrong side of
// ready); the message loses the error or its stack; the shell's onCrash (the
// play page's error card, shell/page.ts `fail`) is not called, or is called
// twice; calls after it wait for an answer that never comes.
import { beforeAll, expect, test } from "bun:test";
import { defineSystem, og } from "../src/index.ts";
import { connectPage } from "../shell/transport.ts";
import { browserOf, crashFields, deviceOf, gpuOf, type Context } from "../shell/telemetry.ts";
import type { FrameStats } from "../src/web/frames.ts";
import { loadNativeHost } from "../src/node/index.ts";
import type { GameToken } from "../src/runtime.ts";
import { Shell, type HostError } from "../src/web/shell.ts";
import { startWorker } from "../src/web/worker.ts";

function buildTheVillage(): void {
  throw new Error("the village has no well");
}

/** Anything a DOM call asks for: every member is a callable that answers another. */
function anything(): any {
  const f = () => {};
  return new Proxy(f, {
    get: (_, key) => (key === Symbol.toPrimitive ? () => "" : key === "then" ? undefined : anything()),
    set: () => true,
    apply: () => anything(),
  });
}

/** The page's canvas: CSS size, an OffscreenCanvas to transfer, and a document for the text entry and the accessibility mirror. */
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

/**
 * The Worker as the shell holds it, its other end being this process's
 * global scope (where startWorker listens): messages cross as tasks, both ways.
 */
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

/** The process's own scope back, as it was before startWorker took it. */
function restore(): void {
  scope.postMessage = saved.postMessage;
  scope.onmessage = saved.onmessage;
  Object.assign(console, saved.console);
}

/** The reason a call's promise rejected with, or "resolved". */
const rejection = (p: Promise<unknown>) => p.then(() => "resolved", (e: unknown) => (e instanceof Error ? e.message : String(e)));

async function crashOf(autoplay: boolean): Promise<{ shell: Shell; crashes: Array<{ message: string; error: HostError }>; logs: string[] }> {
  startWorker(og.game({ start: () => buildTheVillage() }) as unknown as GameToken);
  const crashes: Array<{ message: string; error: HostError }> = [];
  const logs: string[] = [];
  let crashed: () => void = () => {};
  const arrived = new Promise<void>((r) => (crashed = r));
  const shell = await Shell.start({
    canvas: canvas(),
    worker: new Port() as unknown as Worker,
    glue: new URL("./fixtures/web-glue/native.ts", import.meta.url).href,
    wasm: "unused.wasm",
    autoplay,
    prime: !autoplay,
    onLog: (lines) => logs.push(...lines),
    onCrash: (message, error) => {
      crashes.push({ message, error });
      crashed();
    },
  });
  await Promise.race([arrived, new Promise((_, reject) => setTimeout(() => reject(new Error("no crash reached the shell in 3 s")), 3000))]);
  // Anything late (a second crash) would arrive within a few tasks.
  await new Promise((r) => setTimeout(r, 50));
  return { shell, crashes, logs };
}

for (const autoplay of [false, true]) {
  test(`a throw in GameDesc.start reaches the shell's crash path after ready (${autoplay ? "autoplay" : "the play page: primed behind the gate"})`, async () => {
    const { shell, crashes } = await crashOf(autoplay);
    expect(crashes.length).toBe(1);
    const [{ message, error }] = crashes as [{ message: string; error: HostError }];
    expect(message).toContain("GameDesc.start threw");
    expect(message).toContain("the village has no well");
    expect(message).toContain("buildTheVillage");
    expect(error.kind).toBe("error");
    expect(error.message).toBe(message);
    expect(shell.error).toEqual(error);
    // Nothing waits on a stopped engine.
    expect(await rejection(shell.probe("tick"))).toContain("the engine stopped");
    expect(await rejection(shell.call("hash"))).toContain("the engine stopped");
    shell.dispose();
    restore();
  });
}

/** Throws once the world reaches tick 90, after frames were played (the crash report's test). */
function digTooDeep(tick: number): void {
  if (tick >= 90) throw new Error(`the mine caved in under player ada@example.com at tick ${tick}`);
}
const Dig = defineSystem("Dig", {}, (_q: unknown, time: { tick: number }) => digTooDeep(time.tick));

test("a crash after play reports its stack, tick, engine, build, browser, device, GPU and memory, and the played frames were counted", async () => {
  startWorker(og.game({ tick: 60, systems: [Dig] }) as unknown as GameToken);
  const crashes: Array<{ message: string; error: HostError }> = [];
  let crashed: () => void = () => {};
  const arrived = new Promise<void>((r) => (crashed = r));
  const shell = await Shell.start({
    canvas: canvas(),
    worker: new Port() as unknown as Worker,
    glue: new URL("./fixtures/web-glue/native.ts", import.meta.url).href,
    wasm: "unused.wasm",
    autoplay: true,
    telemetry: true,
    onCrash: (message, error) => {
      crashes.push({ message, error });
      crashed();
    },
  });
  // The Worker counts the frames it plays (frames.ts), and a take starts the count again.
  let frames: FrameStats | null = null;
  for (let i = 0; i < 100 && !(frames && frames.count > 0); i++) {
    await new Promise((r) => setTimeout(r, 20));
    frames = (await shell.call("telemetry")) as FrameStats | null;
  }
  expect(frames!.count).toBeGreaterThan(0);
  expect(frames!.interval.reduce((a, b) => a + b, 0)).toBe(frames!.count);
  expect(frames!.interval.length).toBe(frames!.bins.length + 1);
  await Promise.race([arrived, new Promise((_, reject) => setTimeout(() => reject(new Error("no crash reached the shell in 5 s")), 5000))]);
  const [{ message, error }] = crashes as [{ message: string; error: HostError }];

  // The report, through the top-level page's transport to a stand-in play origin.
  const reports: Array<Record<string, unknown>> = [];
  const page = connectPage({ id: "runner01", releaseId: "release_abc123" }, {
    fetch: async (url, init) => {
      const body = JSON.parse(String(init.body)) as Record<string, unknown>;
      if (url.endsWith("/report")) reports.push(...(body["messages"] as Array<Record<string, unknown>>));
      return new Response(JSON.stringify(url.endsWith("/hello") ? { kind: "hello", capabilities: [] } : { accepted: 1 }), { status: url.endsWith("/hello") ? 200 : 202 });
    },
    storage: null,
    location: { hash: "", pathname: "/", search: "" },
    replaceUrl: () => {},
    now: () => 1_800_000_000_000,
    clock: () => 1000,
    random: (n) => new Uint8Array(n).fill(7),
    setTimer: () => 0,
    clearTimer: () => {},
  });
  const ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.1 Safari/605.1.15";
  const context: Context = {
    engine: "0.7.7",
    build: "0123456789abcdef",
    browser: browserOf(ua),
    device: deviceOf({ ua, coarse: true, touchPoints: 5, screen: { width: 1024, height: 1366 }, dpr: 2, cores: 8, memory: 8 }),
    gpu: gpuOf({ info: { vendor: "apple", architecture: "metal-3", description: "Apple M2 driver 1.2.3 at /System/Library" }, limits: { maxTextureDimension2D: 16384, maxBufferSize: 4294967296, notALimit: 1 }, features: new Set(["texture-compression-astc", "made-up-feature"]) }),
  };
  page.start();
  page.crash("EngineCrash", message, crashFields(error, context, { linear: 1 << 26, heapLive: 1 << 20, heapPeak: 1 << 21, streamUsed: null, streamBudget: null, audio: null, entities: 3 }));
  await page.flush();
  const crash = reports.find((m) => m["kind"] === "crash")!;
  expect(crash["name"]).toBe("EngineCrash");
  expect(crash["fingerprint"]).toMatch(/^oe-[0-9a-f]{8}$/);
  expect(crash["cause"]).toBe("error");
  expect(crash["tick"]).toBeGreaterThanOrEqual(89);
  const stack = crash["stack"] as string[];
  // The game's frames, cut to the file name and its line (the platform maps a bundle's with the build's maps).
  expect(stack[0]).toMatch(/^at digTooDeep \(web-shell-crash\.test\.ts:\d+:\d+\)$/);
  expect(crash["engine"]).toBe("0.7.7");
  expect(crash["build"]).toBe("0123456789abcdef");
  expect(crash["browser"]).toEqual({ name: "safari", major: 26, os: "macos" });
  expect(crash["device"]).toEqual({ class: "tablet", cores: 8, memory: 8, dpr: 2, touch: true });
  expect(crash["gpu"]).toEqual({ vendor: "apple", architecture: "metal-3", fallback: false, limits: { maxTextureDimension2D: 16384, maxBufferSize: 4294967296 }, features: ["texture-compression-astc"] });
  expect((crash["memory"] as { entities: number }).entities).toBe(3);
  // Nothing personal: not the message's text, an address, a path of this machine or the driver's description.
  const text = JSON.stringify(crash);
  for (const leak of ["ada@example.com", "mine caved", import.meta.dir, "driver", "/System/"]) expect(text).not.toContain(leak);
  shell.dispose();
  restore();
});
