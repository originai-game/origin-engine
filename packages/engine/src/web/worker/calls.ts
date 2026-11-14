// The shell's `call`s to the engine Worker (worker.ts), by method: a
// harness's ticks and frames, the command channel, the probe, and what the
// Worker measured of the run.

import { boundHost, type Engine } from "../../runtime.ts";
import * as pipelines from "../pipelines.ts";
import { pumpRoom } from "./platform.ts";
import type { Run } from "./run.ts";

export type Calls = Record<string, (e: Engine, args: unknown[]) => unknown>;

/** The calls `run` answers. */
export function callsOf(run: Run): Calls {
  return {
    // A harness's ticks carry the room as the page's own frames do: lockstep's inputs leave with them.
    advance: (e, [n]) => {
      const answered = e.advance(Number(n));
      pumpRoom(e);
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
        run.glue?.og_web_scripted_frame?.(e.id, true);
        try {
          return e.frame(Number(ms));
        } finally {
          run.glue?.og_web_scripted_frame?.(e.id, false);
        }
      });
      run.relay(answered);
      pumpRoom(e);
      return answered;
    },
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
    probe: (e, [member, ...rest]) => {
      const f = (e.probe as unknown as Record<string, unknown>)[String(member)];
      if (typeof f !== "function") throw new TypeError(`probe has no member '${String(member)}'`);
      return (f as (...a: unknown[]) => unknown)(...rest);
    },
  };
}
