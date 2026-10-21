// The platform's first answers behind the play gate (docs/web-export.md
// §The shell). The shell takes the game's first tick behind the gate and
// holds the clock; og.platform's answers apply at the start of the next
// tick, so without more ticks a returning player's title would show the
// defaults (English, no Continue) until the tap. So after the first tick
// the Worker waits for the host's answers to the calls it made and takes one
// tick to apply them, then again for the calls that tick made (a slot's
// parts after its manifest), until a tick asks the host nothing new, at
// most SETTLE_TICKS ticks, before it draws the first frame (within
// SETTLE_FIRST_MS) and after it (within SETTLE_MS, each settling tick
// drawn). The ticks are the game's own ticks, and the answers reach the
// world as every answer does, at a tick's start, as host input a replay
// repeats; the gate's tap ends the settling, and play goes on from there.

import type { PlatformAnswer, PlatformCall, PlatformHost } from "../platform-local.ts";

/** The most ticks the Worker takes behind the gate to apply the platform's answers. */
export const SETTLE_TICKS = 8;
/** How long the first frame waits for the first tick's answers, ms. */
export const SETTLE_FIRST_MS = 300;
/** How long, from the first tick, the Worker keeps applying answers behind the gate, ms. */
export const SETTLE_MS = 5000;

/**
 * Counts what a platform host was asked and what it answered: the calls
 * sent to it (telemetry is never answered) and the answers that arrived
 * since the last tick took them.
 */
export class PlatformCounter {
  #sent = 0;
  #answered = 0;
  #fresh = false;
  #wake: (() => void) | null = null;

  /** The host, counting the calls it is handed. */
  wrap(host: PlatformHost): PlatformHost {
    return {
      host: host.host,
      capabilities: host.capabilities,
      handle: (calls: readonly PlatformCall[], answer: (request: number, result: PlatformAnswer) => void) => {
        this.sent(calls);
        host.handle(calls, (request, result) => {
          this.answered();
          answer(request, result);
        });
      },
    };
  }

  /** Calls handed to the host. */
  sent(calls: readonly PlatformCall[]): void {
    this.#sent += calls.filter((c) => c.method !== "telemetry").length;
  }

  /** One answer arrived (the engine has it; the next tick applies it). */
  answered(): void {
    this.#answered++;
    this.#fresh = true;
    if (this.waiting === 0) this.#wake?.();
  }

  /** Calls the host has not answered yet. */
  get waiting(): number {
    return Math.max(0, this.#sent - this.#answered);
  }

  /** Whether an answer arrived since the last `took`. */
  get fresh(): boolean {
    return this.#fresh;
  }

  /** A tick took the answers that had arrived. */
  took(): void {
    this.#fresh = false;
  }

  /** Resolves when the host answered every call it was handed, or at `until` (performance.now()). */
  quiet(until: number): Promise<void> {
    if (this.waiting === 0) return Promise.resolve();
    return new Promise((resolve) => {
      const done = () => {
        clearTimeout(timer);
        if (this.#wake === done) this.#wake = null;
        resolve();
      };
      const timer = setTimeout(done, Math.max(0, until - performance.now()));
      this.#wake = done;
    });
  }
}

/**
 * Behind the gate: while the host answers what the game asked, take one
 * tick to apply the answers (and `drawn` after each), until a tick leaves
 * nothing new to apply, `stop()` says play began, `until` passes or
 * `ticks` reach SETTLE_TICKS. `pending` is the engine's count of calls the
 * world has no answer to yet (Engine.platformPending): with nothing in
 * flight at the host, a pending call is one the engine answers itself at
 * the next tick (a slot whose parts all arrived), so that tick is taken
 * too. Returns the ticks taken.
 */
export async function settle(
  counter: PlatformCounter,
  advance: () => { ok: boolean; reason?: string },
  o: { until: number; ticks?: number; stop?: () => boolean; drawn?: () => void; pending?: () => number },
): Promise<number> {
  const most = o.ticks ?? SETTLE_TICKS;
  const due = () => counter.fresh || (counter.waiting === 0 && (o.pending?.() ?? 0) > 0);
  let taken = 0;
  while (taken < most && !o.stop?.()) {
    if (counter.waiting > 0) await counter.quiet(o.until);
    if (o.stop?.() || !due()) break;
    counter.took();
    const r = advance();
    if (!r.ok) throw new Error(r.reason ?? "a tick behind the gate failed");
    taken++;
    o.drawn?.();
    if (!due() && counter.waiting === 0) break;
    if (performance.now() >= o.until) break;
  }
  return taken;
}
