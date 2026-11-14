// The engine Worker (docs/design.md §2.2): the wasm instance, the renderer on
// the OffscreenCanvas the shell transferred, and the game bundle. A game's
// worker entry is two lines (og build writes it):
//
//   import { startWorker } from "@origingame/engine/web/worker";
//   import game from "./game.ts";
//   startWorker(game);
//
// The shell (shell.ts) sends `start`, then `call`s. With `root`, the Worker
// fetches the game's cooked bundle before the engine starts (the manifest,
// then every blob in parallel) and hands it to the wasm; once the renderer is
// attached it transcodes the Basis textures with the vendored transcoder
// (basis.ts) to the block format the renderer takes for each kind (BC7/BC5,
// ASTC 4x4 or ETC2/EAC, else RGBA8), so every load of the bundle completes at
// once, before the game starts; later files are fetched on request.
// Probe frames arrive through the request ring (rpc-ring.ts) when the page is
// cross-origin isolated, else as `rpc` messages; input arrives through the
// input ring, else as `input` messages. Ticks follow the display frame
// (requestAnimationFrame in the Worker) unless the shell drives them.
//
// Messages to the shell: { type: "progress", value: 0..1, stage, loaded,
// total } (bytes of the wasm and the cooked bundle actually received, so the
// value is real and never goes backwards), then { type: "ready", engine,
// tier, transport, rpc, textureFormat, files, textures } or { type: "error",
// message } before ready; { type: "frame", first, boot } after the first
// rendered frame (`boot`: BootPhases, where the Worker's boot went); { type: "crash", message, error } when the engine fails after
// ready (a trap, a Rust panic on the engine's thread or in a lane, a failing
// system, an uncaught error; `error` is the structured HostError of shell.ts:
// kind, message, tick, lane, location); { type: "beat", tick } about once a
// second while frames play, which the shell's frame deadline watches;
// { type: "log", lines } with
// the engine's log and the Worker's console (the crash card shows the last
// 200); { type: "text", target } when the UI's focused text control changes
// and { type: "copy", text } when a copy took text (text-input.ts); { type: "saved", scenes } when the inspector saved scenes this page cannot write (page.ts routes them: wire.ts routeSavedScenes); { type: "haptics", patterns } when the UI's feedback played haptics (haptics.ts); { type: "rumble", rumble } when og.input.rumble asked for one (gamepad.ts playRumble); { type: "a11y", tree } when the UI's
// accessibility tree changed (a11y.ts); { type: "load-failure", file, reason } when a file
// of the bundle did not arrive (loadFailure: field telemetry's load failures); { type: "cursor", cursor, image } when the UI's pointer shape changed (og.ui.frameCursor, and its CSS url() image or null); { type: "pointer-lock", wanted } when og.PointerLock.wanted
// changed (the shell requests the lock on the next press on the canvas, or exits it);
// { type: "result", id, value | error } for `call` (`advance`,
// `command`, `hash`, `frame`, `probe`, `timings`: the played frames'
// display intervals and engine work, and `telemetry`: their counts since the
// last take, frames.ts, with `start`'s `telemetry: true`); { type: "rpc",
// frame } for an answer that did not cross the ring.
//
// Requests on the ring are served between frames by the request loop
// whether or not the clock is playing (a page whose display frames stop, a
// hidden tab or frame, still answers), and the loop always waits for the
// next request instead of spinning: a waiting request used to starve the
// Worker's animation frames while the clock played, so isolated pages never
// answered.

import type { GameToken } from "../runtime.ts";
import { boundHost } from "../runtime.ts";
import { WorkerAudio } from "./audio.ts";
import { basisJob } from "./boot.ts";
import { callsOf } from "./worker/calls.ts";
import { Gate } from "./worker/gate.ts";
import { FrameLoop } from "./worker/loop.ts";
import type { WorkerScope } from "./worker/messages.ts";
import { pumpRoom, toPage } from "./worker/platform.ts";
import { Run } from "./worker/run.ts";
import { start } from "./worker/start.ts";
export type { BootPhases } from "./worker/state.ts";
export { BEAT_MS } from "./worker/loop.ts";
export { describeHostError, hostErrorOf } from "./worker/run.ts";

declare const self: WorkerScope;

/** The most log lines the Worker keeps for a crash card. */
export const LOG_LINES = 200;

/**
 * Run `game` in this Worker, driven by the shell's messages. Its concerns
 * live under worker/: the run's state and how it fails (run.ts), the frame
 * loop (loop.ts), the play gate (gate.ts), the boot (start.ts), the calls
 * (calls.ts), og.platform and the rooms (platform.ts), the GPU device and
 * the warm-up's pacing (gpu.ts), and the state each frame keeps (state.ts).
 */
export function startWorker(game: GameToken): void {
  // The probe's requests ask the frame loop, made after the run, for frames.
  const run = new Run(() => loop.schedule());
  const gate = new Gate(run);
  const loop = new FrameLoop(run, gate);

  // The console and the engine's log reach the shell (the crash card's last lines).
  const consoleRef = (globalThis as unknown as { console?: Record<string, (...a: unknown[]) => void> }).console;
  if (consoleRef) {
    for (const level of ["error", "warn", "log", "info"] as const) {
      const original = consoleRef[level]?.bind(consoleRef);
      consoleRef[level] = (...args: unknown[]) => {
        original?.(...args);
        try {
          run.log(`${level === "log" || level === "info" ? "" : `${level}: `}${args.map((a) => (a instanceof Error ? (a.stack ?? a.message) : typeof a === "string" ? a : JSON.stringify(a))).join(" ")}`);
        } catch {
          /* an unserializable argument is not worth a crash */
        }
      };
    }
  }
  self.addEventListener?.("error", (e) => run.crash(e.message ?? String(e.error)));
  self.addEventListener?.("unhandledrejection", (e) => run.crash(e.reason instanceof Error ? e.reason.message : String(e.reason)));

  const calls = callsOf(run);

  self.onmessage = async ({ data }) => {
    // Fork s62: a copy of this Worker in the boot's Basis pool.
    if ((data as { type: string }).type === "og-basis") return basisJob(data as unknown as Parameters<typeof basisJob>[0]);
    if (data.type === "start") {
      await start(game, data, run, gate, loop);
      return;
    }
    const engine = run.engine;
    if (!engine || run.crashed) {
      // A request is answered even now, so that nothing on the page waits for an answer that never comes.
      const why = run.crashed ? "the engine stopped" : "the engine has not started";
      if (data.type === "call") self.postMessage({ type: "result", id: data.id, error: why });
      if (data.type === "rpc") run.probes.refuse(data.frame, why, false);
      return;
    }
    const glue = run.glue;
    if (data.type === "input") engine.probe.input(run.stamped(data.event, performance.now()));
    if (data.type === "viewport" && glue) glue.og_web_set_viewport(engine.id, JSON.stringify(data.viewport));
    if (data.type === "resize" && glue) {
      if (run.canvas) {
        run.canvas.width = Math.max(1, data.width);
        run.canvas.height = Math.max(1, data.height);
      }
      glue.og_web_resize?.(engine.id, data.width, data.height);
      glue.og_web_set_viewport(engine.id, JSON.stringify(data.viewport));
      if (!run.playing) gate.present();
    }
    if (data.type === "fullscreen") glue?.og_web_fullscreen?.(engine.id, data.on);
    if (data.type === "mouse-report") glue?.og_web_mouse_report?.(engine.id, data.active, data.unsupported);
    if (data.type === "keyboard-layout") boundHost()?.abi.og_input_set_keyboard_layout(JSON.stringify(data.labels));
    if (data.type === "audio-latency") run.audio?.latency(data.latency);
    if (data.type === "audio" && glue && !run.audio) {
      const attached = WorkerAudio.attach(glue, engine.id, data);
      if (attached.ok) run.audio = attached.value;
      else run.log(`warn: audio: ${attached.reason}`);
    }
    if (data.type === "rpc") run.probes.serve(data.frame, false);
    if (data.type === "platform-answer") {
      engine.answerPlatform(data.request, data.answer);
      run.platformCounter.answered();
    }
    if (data.type === "platform-notice") engine.notifyPlatform(data.notice);
    if (data.type === "platform-host") engine.attachPlatform({ ...toPage(data.host, data.capabilities, run.platformCounter), time: data.time });
    if (data.type === "room-host") {
      boundHost()?.abi.og_room_set_host(engine.id, JSON.stringify({ host: data.host, ...(data.join ? { join: data.join } : {}) }));
      // A Play link's join leaves now, not with the next frame (the page may be behind its gate).
      pumpRoom(engine);
    }
    if (data.type === "room-events") {
      const abi = boundHost()?.abi;
      for (const event of data.events) abi?.og_room_deliver(engine.id, JSON.stringify(event));
      pumpRoom(engine);
    }
    if (data.type === "play") loop.play(data.on);
    if (data.type === "call") {
      try {
        run.pumpInput(performance.now());
        const call = calls[data.method];
        if (!call) throw new Error(`unknown call '${data.method}'`);
        self.postMessage({ type: "result", id: data.id, value: call(engine, data.args) });
      } catch (e) {
        self.postMessage({ type: "result", id: data.id, error: e instanceof Error ? e.message : String(e) });
      }
    }
  };
}
