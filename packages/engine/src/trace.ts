// The run's trace in the Chrome trace format (the Trace Event Format's JSON
// object form, which Perfetto and chrome://tracing open), the same document
// on both hosts: native `og run --trace` (src/node/run.ts) and the web
// Worker (src/web/worker.ts, read through the shell's `trace()`).
//
// The engine records every span where it ran and hands them over as Chrome
// trace events on its own clock (crates/og-runtime/src/trace.rs: `{ now,
// events }`, `ts` in µs); a take is placed on the document's origin (Unix
// milliseconds) by the difference between the host's time and the engine's
// clock read together. Threads of process 1: 1 engine, 2 render, 3 load,
// 4 cook (og's, natively), 5 gpu, 16 + n og-lanes' pool thread n, 48 + n
// the cook's thread n; threads that appear mid-run arrive with their names.

export type TraceEvent = Record<string, unknown> & { ph: string; pid: number; tid: number; name: string; ts?: number; dur?: number };

/** A Chrome trace document as written to a file. */
export type TraceDocument = {
  traceEvents: TraceEvent[];
  displayTimeUnit: "ms";
  otherData: { originUnixMs: number; producer: string; [k: string]: unknown };
};

/** The fixed threads and their names. */
export const TRACE_THREADS: readonly (readonly [number, string])[] = [
  [1, "engine"],
  [2, "render"],
  [3, "load"],
  [4, "cook"],
  [5, "gpu"],
];

/** The process and fixed threads' metadata events. */
export function traceMetadata(process: string): TraceEvent[] {
  const out: TraceEvent[] = [{ name: "process_name", ph: "M", pid: 1, tid: 0, args: { name: process } }];
  for (const [tid, name] of TRACE_THREADS) {
    out.push({ name: "thread_name", ph: "M", pid: 1, tid, args: { name } });
    out.push({ name: "thread_sort_index", ph: "M", pid: 1, tid, args: { sort_index: tid } });
  }
  return out;
}

/** Unix milliseconds, to the microsecond where the clock has it. */
export function unixMs(): number {
  return performance.timeOrigin + performance.now();
}

/** A trace being recorded: takes appended as they come, the document at the end. */
export class Trace {
  readonly events: TraceEvent[];
  readonly origin: number;
  readonly producer: string;
  /** Thread names already in the document (a take names a thread once per engine trace; a document may hold several). */
  private named = new Set<number>();
  /** µs from the engine's clock to the document's origin, set by the first take. */
  private shift: number | undefined;

  /** A new document from `origin` (Unix ms), or one continued from `prior` (og's prelude: the cook). */
  constructor(opts: { origin?: number; prior?: Partial<TraceDocument>; producer: string; process?: string }) {
    this.producer = opts.producer;
    if (opts.prior?.traceEvents) {
      this.events = [...opts.prior.traceEvents];
      this.origin = opts.prior.otherData?.originUnixMs ?? opts.origin ?? unixMs();
    } else {
      this.events = traceMetadata(opts.process ?? opts.producer);
      this.origin = opts.origin ?? unixMs();
    }
    for (const e of this.events) if (e.ph === "M" && e.name === "thread_name") this.named.add(e.tid);
  }

  /**
   * Places one take of the engine (`og_*_trace_take`'s JSON) on the
   * document's origin: `hostNow` is the host's Unix ms read beside the take.
   */
  append(taken: string | { now: number; events: TraceEvent[] }, hostNow: number = unixMs()): void {
    const t = typeof taken === "string" ? (JSON.parse(taken) as { now: number; events: TraceEvent[] }) : taken;
    // One shift for the whole trace, from the first take: both clocks are
    // monotonic at the same rate, and a shift read again at every take
    // would move spans that abut across two takes by its jitter (tens of
    // µs), so that they overlap.
    this.shift ??= Math.round((hostNow - t.now - this.origin) * 1000);
    const shift = this.shift;
    for (const e of t.events) {
      if (e.ph === "M") {
        if (e.name === "thread_name") {
          if (this.named.has(e.tid)) continue;
          this.named.add(e.tid);
        }
        this.events.push(e);
        continue;
      }
      this.events.push({ ...e, ts: (e.ts ?? 0) + shift });
    }
  }

  document(extra: Record<string, unknown> = {}): TraceDocument {
    return { traceEvents: this.events, displayTimeUnit: "ms", otherData: { originUnixMs: this.origin, producer: this.producer, ...extra } };
  }
}

/** What {@link checkTrace} found. */
export type TraceCheck = { errors: string[]; spans: number; threads: Record<number, string> };

/**
 * Holds a document to the format as Perfetto's importer reads it, and to
 * what this engine's traces promise: `traceEvents` a list; every `X` event
 * a name, integer `ts`, `dur` ≥ 0 and integer `pid`/`tid`; metadata with
 * `args.name`; every span on a thread nested inside the one it starts in
 * (never overlapping it); and spans that lie where they ran. A thread whose
 * children are laid end to end from their parent's start in every parent
 * (the first child at the parent's `ts`, each next one where the previous
 * ended), over at least `endToEndAfter` parents with two or more children,
 * is reported: recorded starts leave gaps somewhere. The GPU's passes
 * (`cat` "gpu") are exempt: one timestamp ends a pass and begins the next,
 * so they are back to back by the measurement's construction.
 */
export function checkTrace(doc: unknown, endToEndAfter = 8): TraceCheck {
  const errors: string[] = [];
  const threads: Record<number, string> = {};
  const d = doc as Partial<TraceDocument> | null;
  if (!d || !Array.isArray(d.traceEvents)) return { errors: ["traceEvents is not a list"], spans: 0, threads };
  const byTid = new Map<number, { ts: number; dur: number; name: string }[]>();
  const backToBack = new Set<number>();
  for (const e of d.traceEvents) {
    if (!Number.isInteger(e.pid) || !Number.isInteger(e.tid)) errors.push(`${e.name}: pid/tid not integers`);
    if (e.ph === "M") {
      const name = (e.args as { name?: unknown } | undefined)?.name;
      if ((e.name === "thread_name" || e.name === "process_name") && typeof name !== "string") errors.push(`${e.name} metadata without args.name`);
      if (e.name === "thread_name" && typeof name === "string") threads[e.tid] = name;
      continue;
    }
    if (e.ph !== "X") continue;
    if (typeof e.name !== "string" || !Number.isInteger(e.ts) || !Number.isInteger(e.dur) || (e.dur as number) < 0) {
      errors.push(`malformed span ${JSON.stringify(e).slice(0, 120)}`);
      continue;
    }
    let list = byTid.get(e.tid);
    if (!list) byTid.set(e.tid, (list = []));
    list.push({ ts: e.ts as number, dur: e.dur as number, name: e.name });
    if (e.cat === "gpu") backToBack.add(e.tid);
  }
  let spans = 0;
  for (const [tid, list] of byTid) {
    if (!(tid in threads)) errors.push(`thread ${tid} has spans and no name`);
    spans += list.length;
    // Parents first: earlier start, then the longer one.
    list.sort((a, b) => a.ts - b.ts || b.dur - a.dur);
    const stack: { ts: number; end: number; name: string; children: { ts: number; end: number }[] }[] = [];
    const closed: { ts: number; children: { ts: number; end: number }[] }[] = [];
    const close = () => closed.push(stack.pop()!);
    for (const s of list) {
      const end = s.ts + s.dur;
      while (stack.length && stack[stack.length - 1]!.end <= s.ts) close();
      const top = stack[stack.length - 1];
      if (top && end > top.end) {
        errors.push(`thread ${tid}: ${s.name} [${s.ts}, ${end}] overlaps ${top.name} [${top.ts}, ${top.end}] without nesting`);
        continue;
      }
      top?.children.push({ ts: s.ts, end });
      stack.push({ ts: s.ts, end, name: s.name, children: [] });
    }
    while (stack.length) close();
    const parents = closed.filter((p) => p.children.length >= 2);
    const laid = parents.filter((p) => p.children[0]!.ts === p.ts && p.children.every((c, i) => i === 0 || c.ts === p.children[i - 1]!.end));
    if (!backToBack.has(tid) && parents.length >= endToEndAfter && laid.length === parents.length) {
      errors.push(`thread ${tid} (${threads[tid] ?? "?"}): the children of all ${parents.length} parents are laid end to end from the parent's start: starts were derived, not recorded`);
    }
  }
  return { errors, spans, threads };
}
