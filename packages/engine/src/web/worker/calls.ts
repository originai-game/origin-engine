// The shell's `call`s to the engine Worker (worker.ts), by method: a
// harness's ticks and frames, the command channel, the probe, and what the
// Worker measured of the run.

import { beginBundle, boundHost, type Engine } from "../../runtime.ts";
import * as pipelines from "../pipelines.ts";
import { pumpRoom } from "./platform.ts";
import { pumpShared } from "./shared.ts";
import type { Run } from "./run.ts";

export type Calls = Record<string, (e: Engine, args: unknown[]) => unknown>;

/** The calls `run` answers. */
export function callsOf(run: Run): Calls {
  return {
    // A harness's ticks carry the room and the shared world as the page's own frames do: lockstep's inputs and the proposals leave with them.
    advance: (e, [n]) => {
      const answered = e.advance(Number(n));
      pumpRoom(e);
      pumpShared(e);
      return answered;
    },
    command: (e, [action, args]) => e.command(String(action), (args as string[]) ?? []),
    /** The files the command channel wrote since the last call (a script's `dump_*`, as `og run --commands` writes them). */
    outputs: (e) => e.takeOutputs(),
    hash: (e) => e.hash(),
    /** What the engine holds (ProbeMemory: the wasm linear memory, the counted heap, the world, streaming, audio), in every build. */
    memory: (e) => JSON.parse(run.glue?.og_web_memory?.(e.id) ?? "null"),
    /**
     * The display frame drawn last as RGBA8 rows (the probe's Capture::frame,
     * `og render`'s every-frame capture), or null until its readback lands: the
     * first call turns the keeping of frames on; ask again before the next frame.
     */
    renderFrame: (e) => {
      const rgba = run.glue?.og_web_render_frame?.(e.id);
      return rgba && rgba.length > 0 ? rgba : null;
    },
    /** What the display draws now (og.render.display: the frame cap, render scale, automatic quality, device losses, GPU memory). */
    display: (e) => JSON.parse(boundHost()!.abi.og_render_display(e.id)),
    /** Test hook: destroy the GPU device (og_web_lose_device); the next frames make the renderer again and the game goes on. */
    loseDevice: (e) => run.glue?.og_web_lose_device?.(e.id) ?? false,
    /** What the lanes did so far (crates/og-runtime/src/author/lanes.rs `report`). */
    lanes: () => JSON.parse(run.glue?.og_web_lanes?.() ?? "null"),
    /** Test hook: the next item at lanes site `site` panics (og_web_lanes_fault); false when the module has no such hook or site. */
    fault: (_, [site]) => run.glue?.og_web_lanes_fault?.(String(site)) ?? false,
    frame: (e, [ms]) => {
      // Fork s60: a harness's frame is compared with the native one: nothing in it skipped for a pipeline compiling.
      // Its time is the harness's and the machine's, not the display's: automatic quality must not read it
      // (a loaded machine stepped the same script down a render-scale rung at a different tick each run).
      const answered = pipelines.drawnWhole(() => {
        run.glue?.og_web_frame_kind?.(e.id, "scripted");
        try {
          return e.frame(Number(ms));
        } finally {
          run.glue?.og_web_frame_kind?.(e.id, "display");
        }
      });
      run.relay(answered);
      pumpRoom(e);
      pumpShared(e);
      return answered;
    },
    /** og.shared.status on this page, as `og run --json` reports it natively (`shared`). */
    shared: (e) => JSON.parse(boundHost()!.abi.og_shared_status(e.id)),
    /** og.net on this page: og.net.status (lockstep running, its start tick, the seats, the first divergent tick) and the shared hash, as `og run --json` reports them natively (`lockstep`, `netHash`). */
    net: (e) => {
      const abi = boundHost()!.abi;
      return { status: JSON.parse(abi.og_net_status(e.id)), hash: JSON.parse(abi.og_net_hash(e.id)), room: JSON.parse(abi.og_platform_room_info(e.id)) };
    },
    // The frames played since the last reset (oldest first); `true` resets.
    /** Field telemetry: the frames played since the last take (frames.ts FrameStats), counting from now on; null when the page's build has telemetry off. */
    telemetry: () => run.frameCounter?.take() ?? null,
    timings: (_, [reset]) => run.records.take(Boolean(reset)),
    /**
     * The run's trace (src/trace.ts): `start` begins one (dropping any), `stop` ends it, `take` (the
     * default) answers the document so far, a Chrome trace as `og run --trace` writes it; null when
     * none is recorded or the module cannot trace.
     */
    trace: (_, [action]) => {
      const a = String(action ?? "take");
      if (a === "start") {
        run.trace.turn(run.engine, false);
        return run.trace.turn(run.engine, true);
      }
      if (a === "stop") return run.trace.turn(run.engine, false);
      return run.trace.document(run.engine, { tier: run.lastTier, lanes: run.lanes });
    },
    /**
     * The performance record (guide perf): `start` begins a fresh one, `stop` ends it, `report`
     * answers `{ json, text }` so far against `budgets` (a JSON object of line to ms). False when
     * the module cannot record.
     */
    perf: (_, [action, budgets]) => {
      const abi = boundHost()?.abi;
      if (!run.engine || !abi) return false;
      const a = String(action ?? "report");
      if (a === "start" || a === "stop") {
        abi.og_engine_perf(run.engine.id, a === "start");
        return true;
      }
      const report = JSON.parse(abi.og_engine_perf_report(run.engine.id, JSON.stringify(budgets ?? {}))) as { json: Record<string, unknown>; text: string };
      // The pipelines asked for asynchronously and when each was made (pipelines.ts `compiled`):
      // the UI's and the overlay's by name, so one still compiling at Play shows.
      const all = pipelines.compiled();
      report.json["pipelines"] = all;
      const ui = all.filter((c) => /^(ui|overlay)\b/.test(c.label));
      if (ui.length) {
        const ms = (v: number | null) => (v === null ? "compiling" : `${Math.round(v)} ms`);
        report.text += `\nui and overlay pipelines asked for asynchronously (${ui.length} of ${all.length})\n`;
        for (const c of ui) report.text += `  ${c.label.padEnd(32)} asked ${ms(c.asked).padStart(9)}  made ${ms(c.made).padStart(9)}  ${c.how ?? ""}\n`;
      }
      return report;
    },
    /**
     * A development build's live edit (the UI gallery): the cooked files a re-cook changed
     * (`[path, bytes]`, each path as the boot put it) replace those the engine holds, and what
     * they change is read again in place (og_runtime::dev_refresh): a document, a style or the
     * default style, the world, the code and the UI kept. No hot reload: a code change needs the
     * probe's `reload`. Answers `{ ms, putMs, steps, warnings }` timed in the Worker, or
     * `{ error }`.
     */
    devFiles: (e, [files]) => {
      const t0 = performance.now();
      for (const [path, bytes] of files as Array<[string, Uint8Array]>) run.glue?.og_web_put_file(path, bytes);
      const putMs = performance.now() - t0;
      const refreshed = run.glue?.og_web_dev_refresh?.(e.id);
      if (refreshed === undefined) return { error: "this engine module has no live edit (a publish build)" };
      return { ...(JSON.parse(refreshed) as Record<string, unknown>), putMs: Math.round(putMs * 10) / 10 };
    },
    /**
     * `og dev --web`'s code edit: the game's code bundle at `url` (build.ts `web-code`, which
     * reads the engine modules this page's own bundle kept) is imported, its declarations
     * replacing the old bundle's, and the probe's `reload` swaps it in (snapshot, swap,
     * restore: the world kept). Answers `{ importMs, reloadMs, report }`, timed in the Worker;
     * a failed import throws and the old code keeps running.
     */
    devCode: async (e, [url]) => {
      const t0 = performance.now();
      beginBundle();
      await import(/* @vite-ignore */ String(url));
      const t1 = performance.now();
      const report = e.probe.reload();
      const t2 = performance.now();
      const round = (ms: number) => Math.round(ms * 10) / 10;
      return { importMs: round(t1 - t0), reloadMs: round(t2 - t1), report };
    },
    /** `og dev --web`'s Stage channel: opens or closes the inspector (the `inspect` command). */
    inspector: (e, [open]) => e.command("inspect", [open ? "on" : "off"]),
    probe: (e, [member, ...rest]) => {
      const f = (e.probe as unknown as Record<string, unknown>)[String(member)];
      if (typeof f !== "function") throw new TypeError(`probe has no member '${String(member)}'`);
      return (f as (...a: unknown[]) => unknown)(...rest);
    },
  };
}
