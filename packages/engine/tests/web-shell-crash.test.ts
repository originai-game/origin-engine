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
import { og } from "../src/index.ts";
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
