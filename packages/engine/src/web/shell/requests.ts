// What the shell (shell.ts) asks the engine Worker and waits for: its calls
// and the probe's JSON-RPC requests, the probe's through the request ring
// when the page is cross-origin isolated. Once the engine stops, every
// request still waiting and every later one fails with why.

import { Pending } from "../pending.ts";
import { readFrames, waitForFrames, writeFrame, type RpcRings } from "../rpc-ring.ts";

export class Requests {
  readonly #worker: Worker;
  readonly #rpc: RpcRings | undefined;
  /** The calls and the probe requests the Worker has not answered yet. */
  readonly #calls = new Pending();
  readonly #probes = new Pending();
  #pumping = false;
  /** Why the engine no longer answers (it crashed, or the shell was disposed); every call after it fails with it. */
  #stopped: Error | null = null;

  constructor(worker: Worker, rpc: RpcRings | undefined) {
    this.#worker = worker;
    this.#rpc = rpc;
  }

  /** Why the engine no longer answers, or null while it does. */
  get stopped(): Error | null {
    return this.#stopped;
  }

  /** One probe member's request (Shell.probe). */
  probe(method: string, params: Record<string, unknown>): Promise<unknown> {
    if (this.#stopped) return Promise.reject(this.#stopped);
    return this.#probes.ask((id) => {
      const frame = JSON.stringify({ jsonrpc: "2.0", id, method, params });
      if (this.#rpc && writeFrame(this.#rpc.requests, frame)) void this.#pump();
      else this.#worker.postMessage({ type: "rpc", frame });
    });
  }

  /** A probe answer, from the ring or an `rpc` message. */
  answer(frame: string): void {
    const m = JSON.parse(frame) as { id?: number; result?: unknown; error?: { code?: number; message?: string } };
    // The JSON-RPC code rides on the error: -32601 is a member this build does not serve.
    const error = m.error ? Object.assign(new Error(m.error.message ?? "probe error"), { code: m.error.code ?? -32603 }) : undefined;
    this.#probes.settle(m.id, error, m.result);
  }

  /** Read answers from the ring while calls are outstanding. */
  async #pump(): Promise<void> {
    if (this.#pumping || !this.#rpc) return;
    this.#pumping = true;
    try {
      while (this.#probes.size > 0 && !this.#stopped) {
        await waitForFrames(this.#rpc.answers, 100);
        for (const frame of readFrames(this.#rpc.answers)) this.answer(frame);
      }
    } finally {
      this.#pumping = false;
    }
  }

  /** One call to the engine (Shell.call). */
  call(method: string, args: unknown[]): Promise<unknown> {
    if (this.#stopped) return Promise.reject(this.#stopped);
    return this.#calls.ask((id) => this.#worker.postMessage({ type: "call", id, method, args }));
  }

  /** A call's `result` message. */
  result(id: number, error: Error | undefined, value: unknown): void {
    this.#calls.settle(id, error, value);
  }

  /** The engine answers nothing more: every pending probe and call fails with `reason`, and so does every later one. */
  stop(reason: string): void {
    if (this.#stopped) return;
    this.#stopped = Object.assign(new Error(reason), { code: -32603 });
    this.#calls.fail(this.#stopped);
    this.#probes.fail(this.#stopped);
  }
}
