// The engine Worker (docs/design.md §2.2): the wasm instance, the renderer on
// the OffscreenCanvas the shell transferred, and the game bundle. A game's
// worker entry is two lines:
//
//   import { startWorker } from "@origingame/engine/web/worker";
//   import game from "./game.ts";
//   startWorker(game);
//
// The shell (shell.ts) sends `start`, then `call`s; input arrives through the
// SharedArrayBuffer ring when the page is cross-origin isolated, else as
// `input` messages. Ticks follow the display frame (requestAnimationFrame in
// the Worker) unless the shell drives them with `advance`.

import { createEngine, type Engine, type GameToken } from "../runtime.ts";
import { loadWebHost, type WebGlue } from "./host.ts";
import { drainRing, type InjectedInput } from "./input.ts";

interface Start {
  type: "start";
  glue: string;
  wasm: string;
  canvas?: OffscreenCanvas;
  webgl?: boolean;
  seed?: number;
  ring?: SharedArrayBuffer;
  autoplay?: boolean;
}

interface Call {
  type: "call";
  id: number;
  method: string;
  args: unknown[];
}

type Message = Start | Call | { type: "input"; event: InjectedInput } | { type: "play"; on: boolean };

interface WorkerScope {
  onmessage: ((e: { data: Message }) => void) | null;
  postMessage(message: unknown): void;
  requestAnimationFrame?(fn: (t: number) => void): number;
}

declare const self: WorkerScope;

/** Run `game` in this Worker, driven by the shell's messages. */
export function startWorker(game: GameToken): void {
  let engine: Engine | null = null;
  let ring: SharedArrayBuffer | undefined;
  let playing = false;
  let last = 0;

  const pump = () => {
    if (engine && ring) for (const event of drainRing(ring)) engine.probe.input(event);
  };

  const frame = (now: number) => {
    if (!playing || !engine) return;
    pump();
    const answer = engine.frame(last ? now - last : 0);
    last = now;
    if (!answer.ok) {
      playing = false;
      self.postMessage({ type: "error", message: answer.reason });
      return;
    }
    schedule();
  };
  const schedule = () => {
    if (self.requestAnimationFrame) self.requestAnimationFrame(frame);
    else setTimeout(() => frame(performance.now()), 16);
  };

  const calls: Record<string, (e: Engine, args: unknown[]) => unknown> = {
    advance: (e, [n]) => e.advance(Number(n)),
    command: (e, [action, args]) => e.command(String(action), (args as string[]) ?? []),
    hash: (e) => e.hash(),
    frame: (e, [ms]) => e.frame(Number(ms)),
    probe: (e, [member, ...rest]) => (e.probe as unknown as Record<string, (...a: unknown[]) => unknown>)[String(member)]!(...rest),
  };

  self.onmessage = async ({ data }) => {
    if (data.type === "start") {
      try {
        const glue = (await import(data.glue)) as WebGlue;
        await loadWebHost(glue, data.wasm);
        const created = createEngine(game, data.seed === undefined ? {} : { seed: data.seed });
        if (!created.ok) throw new Error(created.reason);
        engine = created.value;
        ring = data.ring;
        let tier = "none";
        if (data.canvas) {
          const attach = (glue as unknown as { og_web_attach_canvas(e: number, c: OffscreenCanvas, webgl: boolean): Promise<string> }).og_web_attach_canvas;
          tier = await attach(engine.id, data.canvas, !!data.webgl);
        }
        self.postMessage({ type: "ready", engine: engine.id, tier, transport: ring ? "ring" : "message" });
        if (data.autoplay) {
          playing = true;
          schedule();
        }
      } catch (e) {
        self.postMessage({ type: "error", message: e instanceof Error ? e.message : String(e) });
      }
      return;
    }
    if (!engine) return;
    if (data.type === "input") engine.probe.input(data.event);
    if (data.type === "play") {
      playing = data.on;
      last = 0;
      if (playing) schedule();
    }
    if (data.type === "call") {
      try {
        pump();
        const call = calls[data.method];
        if (!call) throw new Error(`unknown call '${data.method}'`);
        self.postMessage({ type: "result", id: data.id, value: call(engine, data.args) });
      } catch (e) {
        self.postMessage({ type: "result", id: data.id, error: e instanceof Error ? e.message : String(e) });
      }
    }
  };
}
