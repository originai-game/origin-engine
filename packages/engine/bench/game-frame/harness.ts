// The game-frame benchmark's core, the same on every JavaScript engine: it
// runs a game's frames on a bound host and splits each frame's time into the
// game's JavaScript (its systems, hooks, commands and UI callbacks, less the
// engine calls they make), the runtime's JavaScript around the engine, and
// the engine's own native time. bench/game-frame/node.ts drives it under node
// and bun, hermes.ts under Hermes (crates/og-host-c/hermes).
//
// Two modes. `totals` times only the frame call and the dispatcher, so the
// game's figure includes the engine calls it makes, with next to no
// overhead. `calls` also times every engine call, attributing each span's
// self time to the system or binding symbol it belongs to; its clock reads
// add overhead, so it ranks the hot paths and `totals` gives the frame time.

import type { Dispatcher, HostAbi } from "../../src/generated/abi.ts";
import { type CreateOptions, type GameToken, type HostBinding, bindHost, createEngine } from "../../src/runtime.ts";
import { systems } from "../../src/runtime/declare.ts";

export type Mode = "totals" | "calls";

export interface Options {
  readonly binding: HostBinding;
  readonly game: GameToken;
  /** The project directory whose cooked/ the engine opens, if any. */
  readonly root?: string;
  readonly project?: CreateOptions["project"];
  /** Milliseconds, any monotonic origin. */
  readonly now: () => number;
  readonly mode: Mode;
  readonly warm: number;
  readonly frames: number;
  readonly tickMs: number;
  /** Commands run once the engine exists, before the warm frames (`spawn_wave 10000`). */
  readonly commands?: readonly (readonly string[])[];
}

/** A span's self time, by what it was. */
type Bucket = { ms: number; calls: number };

export interface Report {
  readonly frames: number;
  /** Mean milliseconds per frame. */
  readonly frameMs: number;
  /** The game's JavaScript per frame (dispatch less the engine calls inside it; in `totals` mode, with them). */
  readonly gameMs: number;
  /** The runtime's JavaScript outside the dispatcher (the frame call's answer, the loaders) per frame. */
  readonly runtimeMs: number;
  /** The engine's native time per frame. */
  readonly nativeMs: number;
  /** Per-frame self time by system, callback kind or binding symbol, largest first. */
  readonly top: { name: string; ms: number; calls: number }[];
  /** Per-frame times, sorted, for percentiles. */
  readonly p50: number;
  readonly p95: number;
}

const KINDS = ["system", "command", "facts", "condition", "hook", "platform", "ui"];

/** `OG_BENCH_COMMANDS`: commands separated by `;`, arguments by spaces. */
export function commands(text: string | undefined): string[][] {
  return (text ?? "").split(";").map((c) => c.trim().split(/\s+/)).filter((c) => c[0] !== "");
}

export function run(o: Options): Report {
  const { now } = o;
  const buckets = new Map<string, Bucket>();
  // The open spans: each one's start and the time its children took.
  const stack: { start: number; child: number }[] = [];
  let measuring = false;
  let game = 0;
  let native = 0;
  const add = (name: string, ms: number) => {
    if (!measuring) return;
    const b = buckets.get(name);
    if (b) {
      b.ms += ms;
      b.calls++;
    } else buckets.set(name, { ms, calls: 1 });
  };
  const enter = () => stack.push({ start: now(), child: 0 });
  const leave = (): number => {
    const s = stack.pop()!;
    const total = now() - s.start;
    if (stack.length > 0) stack[stack.length - 1]!.child += total;
    return total - s.child;
  };

  const raw = o.binding.abi as unknown as Record<string, (...a: unknown[]) => unknown>;
  const wrapped: Record<string, unknown> = {};
  for (const symbol of Object.keys(raw)) {
    const f = raw[symbol]!;
    if (typeof f !== "function") continue;
    if (symbol === "og_host_set_dispatcher") {
      wrapped[symbol] = (d: Dispatcher) =>
        f((kind: number, engine: number, id: number, query: number, tick: number, dt: number, elapsed: number, payload: string) => {
          enter();
          try {
            return d(kind, engine, id, query, tick, dt, elapsed, payload);
          } finally {
            const self = leave();
            if (measuring) game += self;
            add(kind === 0 ? `system ${systems.get(id)?.name ?? id}` : `${KINDS[kind] ?? kind} callback`, self);
          }
        });
    } else if (o.mode === "calls" || symbol === "og_engine_frame") {
      wrapped[symbol] = (...a: unknown[]) => {
        enter();
        try {
          return f(...a);
        } finally {
          const self = leave();
          if (measuring) native += self;
          add(symbol, self);
        }
      };
    } else wrapped[symbol] = f;
  }
  bindHost({ ...o.binding, abi: wrapped as unknown as HostAbi<Uint32Array, unknown> });

  const created = createEngine(o.game, { ...(o.root === undefined ? {} : { root: o.root }), ...(o.project === undefined ? {} : { project: o.project }) });
  if (!created.ok) throw new Error(`createEngine: ${created.reason}`);
  const engine = created.value;
  for (const [action, ...args] of o.commands ?? []) {
    const r = engine.command(action!, args);
    if (!r.ok) throw new Error(`${action}: ${r.reason}`);
  }
  for (let i = 0; i < o.warm; i++) {
    const a = engine.frame(o.tickMs);
    if (!a.ok) throw new Error(`warm frame ${i}: ${a.reason}`);
  }
  const times: number[] = [];
  measuring = true;
  const t0 = now();
  for (let i = 0; i < o.frames; i++) {
    const s = now();
    const a = engine.frame(o.tickMs);
    times.push(now() - s);
    if (!a.ok) throw new Error(`frame ${i}: ${a.reason}`);
  }
  const wall = now() - t0;
  measuring = false;
  engine.destroy();
  times.sort((a, b) => a - b);
  const n = o.frames;
  const per = (x: number) => +(x / n).toFixed(4);
  return {
    frames: n,
    frameMs: per(wall),
    gameMs: per(game),
    nativeMs: per(native),
    runtimeMs: per(wall - game - native),
    top: [...buckets]
      .map(([name, b]) => ({ name, ms: per(b.ms), calls: +(b.calls / n).toFixed(2) }))
      .sort((a, b) => b.ms - a.ms)
      .slice(0, 25),
    p50: +times[Math.floor(n * 0.5)]!.toFixed(4),
    p95: +times[Math.floor(n * 0.95)]!.toFixed(4),
  };
}
