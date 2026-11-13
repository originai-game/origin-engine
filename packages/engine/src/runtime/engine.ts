import type * as Api from "@origingame/engine";
import type { Dispatcher } from "../generated/abi.ts";
import { type PlatformAnswer, type PlatformCall, type PlatformHost, type PlatformLink, type PlatformNotice, noPlatform } from "../platform-local.ts";
import { type GameToken, callbacks, conditions, engineGames, engineRoots, gameHandles, hookEntries, systems } from "./declare.ts";
import { abi, words } from "./host.ts";
import { checkedProbe } from "./probe-params.ts";
import { dirtyRecords, flushRecords, forgetRecords } from "./resources.ts";
import { type CursorImage, type Result, json, parse, refused } from "./result.ts";
import { rt } from "./state.ts";
import { QueryImpl, learnBuild } from "./views.ts";
import { rates, uiReactor, uiReactors, worldFor, worlds } from "./world.ts";

// ── the dispatcher ────────────────────────────────────────────────────────

export function message(e: unknown): string {
  const text = e instanceof Error ? (e.stack ?? e.message) : String(e);
  if (text.length === 0) return "game code threw an empty error";
  return text.startsWith("=") ? ` ${text}` : text;
}

const engines = new Map<number, Engine>();

/** Dispatch kinds: system, command, facts, condition, hook, platform, UI registration. */
export const dispatch: Dispatcher = (kind, engine, id, query, tick, dt, elapsed, payload) => {
  if (kind === 5) {
    try {
      engines.get(engine)?.pumpPlatform();
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
    if (kind === 6) {
      const errors: string[] = [];
      const outerRegistration = rt.uiRegistrationErrors;
      rt.uiRegistrationErrors = errors;
      try {
        const result = uiReactor(world).registerPresentation(() => { fn?.(world); });
        if (!result.ok) errors.push(result.reason);
        return errors.join("\n");
      } finally { rt.uiRegistrationErrors = outerRegistration; }
    }
    if (kind === 0) {
      const system = systems.get(id);
      if (!system || !fn) return `the dispatcher has no system with callback id ${id}: the bundle declared ${systems.size} system(s); was the engine built from another bundle?`;
      rt.running = { system, tick };
      // The shared buffer opens with the time scale: scale, hitStop, scaledDt, scaledElapsed.
      words();
      const time = Object.freeze({
        tick, dt, elapsed, rate: rates.get(engine) ?? Math.round(1 / dt),
        scale: rt.sharedFloats[0]!, hitStop: rt.sharedFloats[1]!, scaledDt: rt.sharedFloats[2]!, scaledElapsed: rt.sharedFloats[3]!,
      });
      fn(new QueryImpl(query, system, engine, described), time, world);
      flushRecords();
      return "";
    }
    if (!fn) return `the dispatcher has no callback with id ${id}`;
    if (kind === 3 || kind === 4) {
      const entry = (kind === 3 ? conditions : hookEntries).get(id);
      if (!entry) return `the dispatcher has no ${kind === 3 ? "run condition" : "hook"} with callback id ${id}`;
      rt.running = { system: entry, tick };
      if (kind === 3) {
        const holds = fn(world) === true;
        flushRecords();
        return holds ? "=true" : "=false";
      }
      fn(world, Number(payload));
      flushRecords();
      return "";
    }
    if (kind === 1) {
      fn(world, Object.freeze(JSON.parse(payload || "[]")));
      flushRecords();
      return "";
    }
    if (kind === 2) {
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
  /** The physics profile, project.ogt's `profile`: `3d` (the default) or `2d`. */
  readonly profile?: "2d" | "3d";
  /**
   * project.ogt's text (`og run`, `og test`, `og dev` and `og build` pass
   * it): its `tick`, `seed`, `scenes` and `scene` fill the
   * GameDesc fields the game leaves unset, and one set in both places to
   * different values refuses the engine.
   */
  readonly project?: string;
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
  return { kind: "fixed", now: () => startMs + (engine.world.tick * 1000) / (rates.get(engine.id) ?? 60) };
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
    // The rate the engine runs at: GameDesc.tick, else project.ogt's, else 60.
    const time = parse<Result<{ rate?: number }>>(abi().og_world_resource(id, "og.Time"));
    rates.set(id, (time.ok ? time.value.rate : undefined) ?? game.tick);
    learnBuild(id);
    this.attachPlatform(noPlatform);
    engines.set(id, this);
  }

  /** The fixed tick rate the engine runs at, in hertz (GameDesc.tick, else project.ogt's `tick`, else 60). */
  get rate(): number {
    return rates.get(this.id) ?? this.game.tick;
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

  /** One display frame: tick by the wall clock (through the probe: due command steps run), then render. */
  frame(elapsedMs: number): {
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
    /** The UI's accessibility tree, on the frames where it changed, when the engine was created with `a11y` (web/a11y.ts mirrors it). */
    a11y?: Api.A11yTree;
    /** og.PointerLock.wanted: the game asks for the pointer lock (the web shell requests it on the next press on the canvas, and exits it when false). */
    pointerLock?: boolean;
    /** The pointer's shape the UI resolved over what it is on (a CSS cursor name: og.ui.frameCursor). */
    cursor?: string;
    /** On the frames where it changed: the first of the cursor's CSS `url()` images that loads, row-major RGBA8 in base64 with its hotspot, or null for none (`cursor` is its fallback). */
    cursorImage?: CursorImage | null;
    /** The frame cap og.render.setDisplay resolved to (0: the display's rate): the web Worker draws at most this many frames a second (web/pacing.ts). */
    maxFps?: number;
    /** The player's graphics settings ask the page to be fullscreen (og.render.setDisplay's `fullscreen`; the web shell enters or leaves it). */
    fullscreen?: boolean;
    /** The power preference the next load's GPU adapter is asked for: low power (the web shell keeps it in the page's storage). */
    lowPower?: boolean;
    /** Ask pointer lock for raw mouse movement (og.input.setMouse's `raw`: unadjustedMovement). */
    rawMouse?: boolean;
    /** The scene texts the inspector's Save (or a prefab Apply or Revert) made on a host that writes no files (development builds): the web shell routes them. */
    saved?: { scenes: { source: string; text: string }[] };
  } {
    this.pumpPlatform();
    const answer = parse<{ ok: boolean; ticks?: number }>(abi().og_engine_frame(this.id, elapsedMs));
    rt.epoch++;
    this.pumpPlatform();
    return answer;
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
  const a = abi;
  return checkedProbe({
    get version() { return a().og_runtime_probe_get_version(id); },
    tick: () => a().og_runtime_probe_tick(id),
    state: () => parse(a().og_runtime_probe_state(id)),
    advance: (steps: number) => { const t = a().og_runtime_probe_advance(id, steps); rt.epoch++; return t; },
    pause: () => a().og_runtime_probe_pause(id),
    resume: () => a().og_runtime_probe_resume(id),
    input: (event: unknown) => a().og_runtime_probe_input(id, JSON.stringify(event)),
    accepted: () => a().og_runtime_probe_accepted(id),
    pick: (x: number, y: number) => parse(a().og_runtime_probe_pick(id, x, y)),
    entity: (pid: string) => parse(a().og_runtime_probe_entity(id, pid)),
    bounds: (pid: string) => parse(a().og_runtime_probe_bounds(id, pid)),
    patch: (pid: string, parameter: string, value: unknown) => a().og_runtime_probe_patch(id, pid, parameter, JSON.stringify(value)),
    run: (program: unknown) => { const r = parse(a().og_runtime_probe_run(id, JSON.stringify(program))); rt.epoch++; return r; },
    hello: () => parse(a().og_engine_probe_hello(id)),
    capture: (width?: number, height?: number, fit?: number, region?: unknown) =>
      parse(a().og_engine_probe_capture(id, width, height, fit, json(region ?? undefined))),
    marks: (desc?: unknown) => parse(a().og_engine_probe_marks(id, json(desc))),
    timeline: () => parse(a().og_engine_probe_timeline(id)),
    sweep: (desc?: unknown) => parse(a().og_engine_probe_sweep(id, json(desc))),
    reload: () => parse(a().og_engine_probe_reload(id)),
    snapshot: () => parse(a().og_engine_probe_snapshot(id)),
    restore: (sid: string) => {
      rt.epoch++;
      const restored = a().og_engine_probe_restore(id, sid);
      uiReactors.get(id)?.restoreMounts();
      return restored;
    },
    dump: (desc?: unknown) => parse(a().og_engine_probe_dump(id, json(desc))),
    commands: (script: string) => parse(a().og_engine_probe_commands(id, script)),
    listen: (seconds?: number) => parse(a().og_engine_probe_listen(id, seconds)),
    memory: () => parse(a().og_engine_probe_memory(id)),
    describe: () => parse(a().og_engine_probe_describe(id)),
    expect: (what: string) => parse(a().og_engine_probe_expect(id, what)),
    expectations: () => parse(a().og_engine_probe_expectations(id)),
    assertions: (take?: boolean) => parse(a().og_engine_probe_assertions(id, take)),
    assertionEvents: () => parse(a().og_engine_probe_assertion_events(id)),
    observe: (records?: readonly string[] | null, types?: boolean) => parse(a().og_engine_probe_observe(id, json(records ?? undefined), types)),
    saveScene: (write?: boolean, spawned?: boolean) => parse(a().og_engine_probe_save_scene(id, write, spawned)),
    pickUi: (x: number, y: number) => parse(a().og_engine_probe_pick_ui(id, x, y)),
  });
}
