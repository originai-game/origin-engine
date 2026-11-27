import type * as Api from "@origingame/engine";
import { type Custom, EngineProbe, RuntimeProbe } from "../generated/calls.ts";
import { DISPATCH_COMMAND, DISPATCH_CONDITION, DISPATCH_FACTS, DISPATCH_HOOK, DISPATCH_PLATFORM, DISPATCH_SYSTEM, DISPATCH_UI, type Dispatcher } from "../generated/abi.ts";
import type { A11yUpdate } from "../web/a11y.ts";
import { type PlatformAnswer, type PlatformCall, type PlatformHost, type PlatformLink, type PlatformNotice, noPlatform } from "../platform-local.ts";
import { type GameToken, callbacks, conditions, engineGames, engineRoots, gameHandles, hookEntries, systems } from "./declare.ts";
import { abi, words } from "./host.ts";
import { checkedProbe } from "./probe-params.ts";
import { dirtyRecords, flushRecords, forgetRecords } from "./resources.ts";
import { type CursorImage, type Result, json, parse, refused } from "./result.ts";
import { rt } from "./state.ts";
import { QueryImpl, learnBuild } from "./views.ts";
import { rateOf, uiReactor, uiReactors, worldFor, worlds } from "./world.ts";
import { disposeRenderer, mount, readLocale, renderers, runPresentationMounts } from "../ui/index.ts";
import { flowScreen } from "../ui/rt.ts";
import type { UiChild } from "../ui/types.ts";
import { disposeInspector, pumpInspector } from "../ui/inspector-host.ts";

// ── the dispatcher ────────────────────────────────────────────────────────

export function message(e: unknown): string {
  const text = e instanceof Error ? (e.stack ?? e.message) : String(e);
  if (text.length === 0) return "game code threw an empty error";
  return text.startsWith("=") ? ` ${text}` : text;
}

const engines = new Map<number, Engine>();

/** Dispatch kinds: system, command, facts, condition, hook, platform, UI registration. */
export const dispatch: Dispatcher = (kind, engine, id, query, tick, dt, elapsed, payload) => {
  if (kind === DISPATCH_PLATFORM) {
    try {
      engines.get(engine)?.pumpPlatform();
      pumpInspector(engine);
      // The TSX UI's ops since the last boundary, and its events back.
      const ui = renderers.get(engine);
      if (ui) {
        ui.clock.set({ tick, dt });
        readLocale(ui, engine);
        ui.flush();
        const errors = ui.errors.splice(0);
        if (errors.length > 0) return errors.join("\n");
      }
      return "";
    } catch (e) {
      return message(e);
    }
  }
  const fn = callbacks[id];
  const world = worldFor(engine);
  const outer = rt.running;
  const outerCall = rt.calling;
  rt.calling = { engine, tick, world };
  rt.engineCalls++;
  rt.inGame++;
  // What the shared buffer held for an outer call is gone.
  const described = ++rt.sharedStamp;
  try {
    // start runs inside og_engine_create, before Engine's constructor.
    // Assertions in that first callback must know whether this is publish.
    learnBuild(engine);
    if (kind === DISPATCH_UI) {
      const errors: string[] = [];
      const outerRegistration = rt.uiRegistrationErrors;
      rt.uiRegistrationErrors = errors;
      try {
        // "flow": project.ogt's `flow`, drawn by the built-in FlowScreen under the game's own UI
        // (a bundle of a project with the line holds it: node/build.ts).
        const result = uiReactor(world).registerPresentation(() => {
          runPresentationMounts(engine, () => {
            if (payload === "flow") {
              const Screen = flowScreen.component;
              if (!Screen) throw new Error("project.ts's `flow`: this bundle holds no FlowScreen; bundle the game with og (og run, og build), which adds it for a project with a flow object");
              mount(world as unknown as Api.World, Screen as (props: Record<string, never>) => UiChild, {});
            }
            fn?.(world);
          });
        });
        if (!result.ok) errors.push(result.reason);
        return errors.join("\n");
      } finally { rt.uiRegistrationErrors = outerRegistration; }
    }
    if (kind === DISPATCH_SYSTEM) {
      const system = systems.get(id);
      if (!system || !fn) return `the dispatcher has no system with callback id ${id}: the bundle declared ${systems.size} system(s); was the engine built from another bundle?`;
      rt.running = { system, tick };
      // The shared buffer opens with the time scale: scale, hitStopTicks, scaledDt, scaledElapsed.
      words();
      const time = Object.freeze({
        tick, dt, elapsed, rate: rateOf(engine),
        scale: rt.sharedFloats[0]!, hitStopTicks: rt.sharedFloats[1]!, scaledDt: rt.sharedFloats[2]!, scaledElapsed: rt.sharedFloats[3]!,
      });
      fn(new QueryImpl(query, system, engine, described), time, world);
      flushRecords();
      return "";
    }
    if (!fn) return `the dispatcher has no callback with id ${id}`;
    if (kind === DISPATCH_CONDITION || kind === DISPATCH_HOOK) {
      const entry = (kind === DISPATCH_CONDITION ? conditions : hookEntries).get(id);
      if (!entry) return `the dispatcher has no ${kind === DISPATCH_CONDITION ? "run condition" : "hook"} with callback id ${id}`;
      rt.running = { system: entry, tick };
      if (kind === DISPATCH_CONDITION) {
        const holds = fn(world) === true;
        flushRecords();
        return holds ? "=true" : "=false";
      }
      fn(world, Number(payload));
      flushRecords();
      return "";
    }
    if (kind === DISPATCH_COMMAND) {
      fn(world, Object.freeze(JSON.parse(payload || "[]")));
      flushRecords();
      return "";
    }
    if (kind === DISPATCH_FACTS) {
      const facts = JSON.stringify(fn(world) ?? null);
      flushRecords();
      return "=" + facts;
    }
    return `unknown dispatch kind ${kind}`;
  } catch (e) {
    return message(e);
  } finally {
    // What game code wrote before it threw is written too, as a write
    // straight through would have been.
    if (dirtyRecords.length > 0) {
      try {
        flushRecords();
      } catch {
        // The throw already failed the call; this refusal adds nothing.
      }
    }
    // The TSX UI's ops this call wrote (a store a system set, through
    // useStore) reach the tree now, not at the next boundary: og.ui.end
    // lays the tree out inside the tick, so a store set in tick t is drawn
    // in tick t's frame.
    const ui = renderers.get(engine);
    if (ui?.pending) {
      try {
        ui.flush();
      } catch (e) {
        ui.errors.push(message(e));
      }
    }
    rt.inGame--;
    rt.running = outer;
    rt.calling = outerCall;
    rt.epoch++;
    rt.engineCalls++;
    // A system's archetypes in the shared buffer are retired with its views.
    rt.sharedStamp++;
  }
};

// ── engines ───────────────────────────────────────────────────────────────

export interface CreateOptions {
  /** Overrides GameDesc.seed (a replay seed). */
  readonly seed?: number;
  /** The project directory (native) or URL base (web) whose `cooked/` bundle the engine opens before the first tick. */
  readonly root?: string;
  /** The content base og.load.data reads catalogs from: a directory (`og run --content`) or an https URL (the page's `content`). */
  readonly content?: string;
  /** The host names the content base after the engine starts (og_engine_set_content: a page waiting for its Player's hello); og.load.data loads wait for it. */
  readonly contentLater?: boolean;
  /**
   * Normalized startup data from the checked project (`og run`, `og test`,
   * `og dev` and `og build` pass it): `tick`, `seed`, `scenes` and `scene` fill the
   * GameDesc fields the game leaves unset, and one set in both places to
   * different values refuses the engine.
   */
  readonly project?: Api.ProjectStartup;
  /** Run GameDesc.start at the first tick boundary instead of now, so a host can attach its renderer and report the viewport first (the web Worker does). */
  readonly deferStart?: boolean;
  /** The locale the host asks og.i18n for: the browser's language, `og run --locale`. */
  readonly locale?: string;
  /** The host mirrors the UI's accessibility tree (the web Worker): a frame's answer carries `a11y` when the tree changed. */
  readonly a11y?: boolean;
}

/** A file the command channel produced (`og_engine_take_outputs`). */
export interface CommandOutput {
  readonly tick: number;
  readonly action: string;
  /** Where to write it: the script's path, or `<action>-<tick>.<ext>`. */
  readonly file: string;
  readonly kind: "png" | "json" | "text" | "wav";
  /** Base64 for a PNG, the text otherwise. */
  readonly data: string;
}

/** One log line (`og_engine_take_log`). */
export interface LogLine {
  readonly tick: number;
  readonly warning: boolean;
  readonly text: string;
}

/** Start a new bundle's declarations before importing it again (hot reload). */
export function beginBundle(): void {
  abi().og_host_begin_bundle();
}

/**
 * The host's wall clock as og.platform hands it to the game (og.platform.now,
 * PlatformResult.time): read only between ticks, when the platform attaches
 * and when an answer arrives, so it reaches the world as host input.
 */
export interface HostClock {
  /** PlatformInfo.clock: `system`, `browser`, `fixed` or `player`. */
  readonly kind: string;
  /** UTC milliseconds since 1970. */
  now(): number;
}

/** The machine's clock: the native host's (`system`), the web host's (`browser`). */
export const systemClock = (kind = "system"): HostClock => ({ kind, now: () => Date.now() });

/** A clock that starts at `startMs` on tick 0 and moves with the engine's ticks (`og run --clock`, playtests): reproducible. */
export function fixedClock(engine: Engine, startMs: number): HostClock {
  return { kind: "fixed", now: () => startMs + (engine.world.tick * 1000) / rateOf(engine.id) };
}

/** A frame's answer (`Engine.frame`, `Engine.present`). */
export interface FrameAnswer {
  ok: boolean;
  ticks?: number;
  tick?: number;
  rendered?: boolean;
  paused?: boolean;
  quit?: boolean;
  behind?: number;
  /** Wall milliseconds a tick took (this frame's mean, or the last frame's that ticked) and the last render took. */
  stepMs?: number;
  renderMs?: number;
  /** The command channel's steps still to run (scheduled, or a screenshot waiting for its frame): a script has ended at 0, however a restore moved og.Time. */
  pending?: number;
  reason?: string;
  /** The UI's focused text control (web/text-input.ts's TextTarget), or null. */
  text?: { path: string; rect: [number, number, number, number]; multiline: boolean; selection: string; composing: boolean } | null;
  /** Text a copy or cut took this frame. */
  copy?: string;
  /** The haptic patterns the UI's feedback played since the last frame, at the player's strength: [startMs, ms, low, high] pulses (web/haptics.ts plays them). */
  haptics?: [number, number, number, number][][];
  /** og.input.rumble's requests since the last frame: [pad (-1 every pad), low, high, ms] (web/gamepad.ts playRumble plays them). */
  rumble?: [number, number, number, number][];
  /** What changed in the UI's accessibility tree since the last frame, when the engine was created with `a11y` (web/a11y.ts mirrors it). */
  a11y?: A11yUpdate;
  /** A playing video player with `wait` could not show its frame: no tick runs until it can (web/shell.ts onBuffering). Absent: false. */
  buffering?: boolean;
  /** og.PointerLock.wanted: the game asks for the pointer lock (the web shell requests it on the next press on the canvas, and exits it when false). */
  pointerLock?: boolean;
  /** The pointer's shape the UI resolved over what it is on (a CSS cursor name: og.ui.frameCursor). */
  cursor?: string;
  /** On the frames where it changed: the first of the cursor's CSS `url()` images that loads, row-major RGBA8 in base64 with its hotspot, or null for none (`cursor` is its fallback). */
  cursorImage?: CursorImage | null;
  /** The player's graphics settings ask for fullscreen (og.render.setDisplay's `fullscreen`): the host enters or leaves it and reports it (`presentFullscreen`). */
  fullscreen?: boolean;
  /** The power preference the next load's GPU adapter is asked for: low power (the web shell keeps it in the page's storage). */
  lowPower?: boolean;
  /** Ask pointer lock for raw mouse movement (og.input.setMouse's `raw`: unadjustedMovement). */
  rawMouse?: boolean;
  /** The scene texts the inspector's Save (or a prefab Apply or Revert) made on a host that writes no files (development builds): the web shell routes them. */
  saved?: { scenes: { source: string; text: string }[] };
}

/** A running engine: one world, its probe, and the host's lifecycle calls. */
export class Engine {
  readonly id: number;
  readonly game: GameToken;
  /** The engine's world, as game code sees it. */
  readonly world: Api.World;
  /** The engine's probe: RuntimeProbe protocol 5 and the wire members. */
  readonly probe: Api.EngineProbe;

  #platform: PlatformHost = noPlatform;
  #clock: HostClock = systemClock();

  constructor(id: number, game: GameToken) {
    this.id = id;
    this.game = game;
    this.world = worldFor(id) as unknown as Api.World;
    this.probe = makeProbe(id) as unknown as Api.EngineProbe;
    learnBuild(id);
    this.attachPlatform(noPlatform);
    engines.set(id, this);
  }

  /** The fixed tick rate the engine runs at, in hertz (GameDesc.tick, else project.ogt's `tick`, else 60). */
  get rate(): number {
    return rateOf(this.id);
  }

  /** Attach what answers og.platform's calls: the native host's local store, the web shell's Player bridge. */
  attachPlatform(host: PlatformHost): void {
    this.#platform = host;
    abi().og_platform_set_host(this.id, JSON.stringify({ host: host.host, capabilities: host.capabilities, time: host.time ?? this.#clock.now(), clock: host.time === undefined ? this.#clock.kind : "player" }));
  }

  /** Set the clock og.platform tells the game (before attachPlatform, which tells it the time). */
  setClock(clock: HostClock): void {
    this.#clock = clock;
    abi().og_platform_set_host(this.id, JSON.stringify({ host: this.#platform.host, capabilities: this.#platform.capabilities, time: clock.now(), clock: clock.kind }));
  }

  /** Hand the platform calls made since the last pump to the attached host (before and after every advance and frame, after a command). */
  pumpPlatform(): void {
    const calls = parse<PlatformCall[]>(abi().og_platform_take(this.id));
    const host = this.#platform;
    if (calls.length === 0 && !host.poll) return;
    const link = this.#link;
    if (calls.length > 0) host.handle(calls, link.answer, link);
    host.poll?.(link);
  }

  /** What the attached host may do besides answering: notices, and pausing the game while an ad shows. */
  readonly #link: PlatformLink = (() => {
    const engine = this;
    return {
      get tick() {
        return engine.world.tick;
      },
      answer: (request: number, answer: PlatformAnswer) => void engine.answerPlatform(request, answer),
      notify: (notice: PlatformNotice) => void engine.notifyPlatform(notice),
      pause: () => void engine.probe.pause(),
      resume: () => void engine.probe.resume(),
    };
  })();

  /** Unasked host input (a purchase settled, entitlements changed, an invite): applied at the start of the next tick as og.PlatformNotice. */
  notifyPlatform(notice: PlatformNotice): boolean {
    const timed = typeof notice.time === "number" ? { ...notice, clock: "player" } : { ...notice, time: this.#clock.now() };
    return abi().og_platform_notify(this.id, JSON.stringify(timed));
  }

  /** og.platform's calls the world has no answer to yet: in flight at the host, or answered and applied at the next tick's start. */
  platformPending(): number {
    return (parse<{ pending?: number }>(abi().og_platform_info(this.id)).pending ?? 0) as number;
  }

  /** Answer one platform call; applied at the start of the next tick. */
  answerPlatform(request: number, answer: PlatformAnswer): boolean {
    // An answer that carries no time (a Player before the time ask, the local platform) gets the host clock's.
    const timed = typeof answer.time === "number" ? { ...answer, clock: "player" } : { ...answer, time: this.#clock.now() };
    return abi().og_platform_answer(this.id, request, JSON.stringify(timed));
  }

  /** Take fixed steps: the tick reached, or the first failing system with its tick. */
  advance(steps: number): Result<number> {
    const answer = parse<Result<number>>(abi().og_engine_advance(this.id, steps));
    rt.epoch++;
    return answer;
  }

  /** Run a registered command action now, between ticks. */
  command(action: string, args: readonly string[] = []): Result<void> {
    const answer = parse<Result<void>>(abi().og_engine_command(this.id, action, JSON.stringify(args)));
    rt.epoch++;
    this.pumpPlatform();
    return answer;
  }

  /** A harness's frame: tick by `elapsedMs` of its own clock (through the probe: due command steps run), then render. */
  frame(elapsedMs: number): FrameAnswer {
    this.pumpPlatform();
    const answer = parse<FrameAnswer>(abi().og_engine_frame(this.id, elapsedMs));
    rt.epoch++;
    this.pumpPlatform();
    return answer;
  }

  /**
   * One display frame of a presenting host at `nowMs` on its own clock (requestAnimationFrame's timestamp, the
   * desktop window loop's): the engine keeps the frame cap, measures the display and draws this frame or skips it
   * (`drawn`); `nextMs` is when the next frame to draw is due, which a host without a display callback waits until.
   */
  present(nowMs: number): FrameAnswer & { drawn: boolean; nextMs: number } {
    this.pumpPlatform();
    const answer = parse<FrameAnswer & { drawn: boolean; nextMs: number }>(abi().og_present_frame(this.id, nowMs));
    rt.epoch++;
    if (answer.drawn) this.pumpPlatform();
    return answer;
  }

  /** The clock stopped, held or went on (play paused or resumed, a capture's wait): the next `present` draws, handed no time from before. */
  presentResume(): void {
    abi().og_present_resume(this.id);
  }

  /** The display's refresh rate as the host reads it from the display (a desktop window's monitor), Hz; 0 is unknown. */
  presentRefresh(hz: number): void {
    abi().og_present_refresh(this.id, hz);
  }

  /** The host's window went fullscreen or left it: og.render.display's `fullscreen`, which the player's setting follows. */
  presentFullscreen(on: boolean): void {
    abi().og_present_fullscreen(this.id, on);
  }

  /** The probe's JSON-RPC 2.0 entry: one request frame in, its answer frame out. */
  call(frame: string): string {
    const answer = abi().og_engine_call(this.id, frame);
    rt.epoch++;
    return answer;
  }

  /** Files the command channel produced since the last call. */
  takeOutputs(): CommandOutput[] {
    return parse(abi().og_engine_take_outputs(this.id));
  }

  /** Log lines since the last call. */
  takeLog(): LogLine[] {
    return parse<LogLine[]>(abi().og_engine_take_log(this.id));
  }

  /** The world hash, hex: equal hashes mean equal worlds. */
  hash(): string {
    const dump = parse<{ hash?: string } | null>(abi().og_engine_probe_dump(this.id, JSON.stringify({ what: "world" })));
    return dump?.hash ?? "";
  }

  destroy(): boolean {
    rt.epoch++;
    disposeInspector(this.id);
    disposeRenderer(this.id);
    for (const scope of uiReactors.get(this.id)?.scopes.values() ?? []) scope.dispose();
    uiReactors.delete(this.id);
    engines.delete(this.id);
    worlds.delete(this.id);
    engineGames.delete(this.id);
    engineRoots.delete(this.id);
    forgetRecords(this.id);
    return abi().og_engine_destroy(this.id);
  }
}

/** Assemble an engine from a declared game (the bundle's default export). */
export function createEngine(game: GameToken, options: CreateOptions = {}): Result<Engine> {
  const handle = gameHandles.get(game);
  if (handle === undefined) return refused("not a game: pass the bundle's default export, the result of og.game()");
  rt.creatingRoot = options.root;
  let answer: Result<number>;
  try {
    answer = parse<Result<number>>(abi().og_engine_create(handle, JSON.stringify(options)));
  } finally {
    rt.creatingRoot = undefined;
  }
  if (!answer.ok) return answer;
  forgetRecords(answer.value);
  engineGames.set(answer.value, game);
  if (options.root !== undefined) engineRoots.set(answer.value, options.root);
  return { ok: true, value: new Engine(answer.value, game) };
}

function makeProbe(id: number) {
  return checkedProbe(new EngineProbe(id));
}

/** The probe's `custom` members (IDL `probe`): stepping and restoring make every view stale, and a restore remounts the UI. */
export const runtimeProbeCustom: Custom["RuntimeProbe"] = {
  advance(probe, steps) {
    const t = abi().og_runtime_probe_advance(RuntimeProbe.handleOf(probe), steps);
    rt.epoch++;
    return t;
  },
  run(probe, program) {
    const r = parse(abi().og_runtime_probe_run(RuntimeProbe.handleOf(probe), JSON.stringify(program)));
    rt.epoch++;
    return r;
  },
};

export const engineProbeCustom: Custom["EngineProbe"] = {
  restore(probe, sid) {
    rt.epoch++;
    const id = EngineProbe.handleOf(probe);
    const restored = abi().og_engine_probe_restore(id, sid);
    // The stores' restored values reach their subscribers.
    uiReactors.get(id)?.flush();
    return restored;
  },
};
