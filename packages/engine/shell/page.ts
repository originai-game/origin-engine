// The player shell: the main thread of an exported game page (docs/design.md
// §2.2, §4, §6, §9; docs/web-export.md). `og build` inlines the boot markup,
// the brand marks and this page's configuration into index.html and bundles
// this module as play.js. It owns everything that is not the game:
//
// - the boot splash (the brand symbol playing `genesis` once on ink), then
//   `progress` with the real bytes received (wasm + cooked bundle);
// - the play gate (ready to interact): the first gesture unlocks audio,
//   asks for fullscreen when the project wants it, takes a wake lock and
//   starts the clock; a Player that frames the page owns the gate instead
//   (protocol 2 `start`, `gesture`, `pause`, `resume`);
// - layout: letterbox (the project's aspect, centred on ink inside the safe
//   area) or fill (the whole `100dvh` viewport, the safe-area insets passed to
//   the UI; held within `aspect_min` to `aspect_max` where the project bounds
//   it, with bars past them), the device pixel ratio capped and followed
//   across monitors, resize and orientation changes resized in the Worker;
// - visibility: a hidden page pauses the clock and the audio and resumes both;
// - input: keyboard, pointers (touch ids carried), gamepads as keys;
// - the error card (§6): the reason, the last 200 log lines and the world
//   dump (state in a publish build), with Reload and Copy report;
// - the probe: `globalThis.__runtime_probe__` (protocol 5, every member a
//   Promise, plus `lanes`, `memory()` (what the engine holds), `trace()`
//   and `saveTrace()` (the run's Chrome trace; `?trace` records from the
//   boot), `boot`
//   (where the Worker's boot went) and
//   `error`: what stopped the engine, a panic with its tick and lane, an
//   error or a hang) and the platform's `og-probe`
//   postMessage wire; the played
//   frames' timings on `globalThis.__og_timings__` (`og run --device`);
// - PWA: the service worker (publish builds) and the add-to-home-screen hint
//   only when the install criteria hold;
// - "Made with Origin Engine" in the corner unless the project turns it off.

import { Shell, webGpuMissing } from "../src/web/shell.ts";
import { AudioShell } from "../src/web/audio.ts";
import { pollGamepads } from "../src/web/gamepad.ts";
import { mountWire, routeSavedScenes, SCENE_SAVED_CAPABILITY, type SavedScene } from "./wire.ts";
import { connectPlayer } from "./player.ts";
import { connectPage, pageGame, type PageConnection, type PageInfo } from "./transport.ts";
import { deferredPlatform, HELLO_WAIT_MS, playerPlatform, playerRooms, storagePersistence } from "./platform.ts";
import { localPlatform, memoryPersistence, type PlatformHost, type PlatformNotice } from "../src/platform-local.ts";
import type { HostError } from "../src/web/shell.ts";
import type { FrameStats } from "../src/web/frames.ts";
import { mapStack } from "../src/sourcemap.ts";
import { browserOf, crashFields, deviceOf, gpuOf, memoryOf, playFields, type Context, type MemoryInfo } from "./telemetry.ts";

/** The page's configuration (`<script type="application/json" id="og-config">`, written by og build). */
export interface ShellConfig {
  readonly engine: string;
  /** The build's id (og-build.json `sourcemaps.build`): a crash report's `build`. */
  readonly buildId: string;
  /** project.ogt `publish { telemetry }`: `off` measures and sends nothing beyond the session's lifecycle. */
  readonly telemetry: "on" | "off";
  readonly build: "publish" | "dev";
  readonly name: string;
  readonly profile: "2d" | "3d";
  /** project.ogt's text, for the engine's tick, seed, scenes and scene (CreateOptions.project). */
  readonly project?: string;
  readonly isolation: "required" | "preferred" | "off";
  readonly fit: "letterbox" | "fill";
  /** Width over height for letterbox; null takes the screen's. */
  readonly aspect: number | null;
  /**
   * With `fill`: the narrowest and widest aspects (width over height) the canvas
   * takes; on a screen outside them it is that aspect, centred on ink (letterbox
   * bars on a taller screen, pillarbox bars on a wider one: a game framed for
   * 16:10 to 21:9 on a 32:9 monitor). Null: no bound on that side.
   */
  readonly aspectMin: number | null;
  readonly aspectMax: number | null;
  readonly orientation: "any" | "landscape" | "portrait";
  readonly dprCap: number;
  readonly fullscreen: "gesture" | "off";
  readonly gate: "tap" | "none";
  readonly madeWith: boolean;
  readonly serviceWorker: boolean;
  /** Fork s61: the progress caption per boot stage (`fetch`, `transcode`, `prime`, `present`); absent: "Loading". */
  readonly stageLabels?: Readonly<Record<string, string>>;
  /**
   * Fork s61: "page": the page draws the loading screen itself (the engine's boot splash stays hidden) from
   * the window events og:progress {value, stage, fraction}, og:stage {id, fraction} (the game's own stages),
   * og:first-frame and og:failed {message}; absent or "engine": the splash.
   */
  readonly bootScreen?: "engine" | "page";
  /** project.ogt `publish { pipelines }` (src/web/pipelines.ts): "async", the frame pipelines with createRenderPipelineAsync; "prewarm", only the warm-up behind the gate's; absent: synchronously. */
  readonly pipelines?: "async" | "prewarm";
  /** Bytes the boot fetches; `lanes` is the lanes variant's module, 0 when the build ships none. */
  readonly expect: { readonly wasm: number; readonly cooked: number; readonly lanes: number };
  /**
   * A publish build's content-hashed names: each file's own name to the
   * name it is served by (`"game.js": "game.3f2a9c1e04b7.js"`); a file it
   * does not name is served by its own name (docs/web-export.md
   * §Content-hashed names).
   */
  readonly files: Readonly<Record<string, string>>;
}

/** The URL of the build's file `name` (`game.js`), under the name the build serves it by. */
export function fileUrl(config: Pick<ShellConfig, "files">, name: string, base: string): string {
  const served = config.files[name];
  return new URL(`./${typeof served === "string" ? served : name}`, base).href;
}

const PROBE_GLOBAL = "__runtime_probe__";
const TIMINGS_GLOBAL = "__og_timings__";
const REPLAY_SEED_PARAM = "replay-seed";
const TRACE_PARAM = "trace";
const LOG_LINES = 200;
/** Field telemetry's throttle (docs/web-export.md §Telemetry): a play sample at most this often, on a hidden page after at least the second, at most this many a session. */
const SAMPLE_EVERY_MS = 60_000;
const SAMPLE_HIDDEN_MS = 10_000;
const SAMPLES_MAX = 60;
/** The files that did not arrive, kept for the next sample, and reported at most this many a session. */
const FAILURES_MAX = 100;
const INSTALL_DISMISSED = "og-install-dismissed";

const MEMBERS = [
  "hello", "tick", "state", "advance", "pause", "resume", "input", "accepted", "pick", "entity",
  "bounds", "patch", "run", "capture", "sweep", "reload", "snapshot", "restore", "dump", "commands",
  "describe", "expect", "expectations",
] as const;

/** Positional protocol-5 arguments to JSON-RPC params, as og-probe reads them. */
export function probeParams(member: string, args: unknown[]): Record<string, unknown> {
  switch (member) {
    case "advance":
      return { steps: args[0] };
    case "input":
      return { event: args[0] };
    case "pick":
      return { x: args[0], y: args[1] };
    case "entity":
    case "bounds":
    case "restore":
      return { id: args[0] };
    case "patch":
      return { id: args[0], parameter: args[1], value: args[2] };
    case "run":
      return { program: args[0] };
    case "capture":
      return { width: args[0] ?? 0, height: args[1] ?? 0 };
    case "commands":
      return { script: args[0] };
    case "expect":
      return { what: args[0] };
    case "sweep":
    case "dump":
      return (args[0] as Record<string, unknown>) ?? {};
    default:
      return {};
  }
}

const zh = /^zh\b/i.test(globalThis.navigator?.language ?? "");
const T = zh
  ? { loading: "正在加载", play: "点击开始", tapToPlay: "点击开始", startFailed: "游戏无法启动", noWebGpu: "此浏览器不支持 WebGPU", noWebGpuDetail: "本游戏需要 WebGPU，而此浏览器没有。请用最新版的 Chrome 或 Edge（Windows、macOS、ChromeOS，或 Android 12 及以上）、Safari 26 及以上（macOS、iOS、iPadOS），或 Windows 与 macOS 上的 Firefox 打开。", noWebGpuAdapterTitle: "没有可供 WebGPU 使用的显卡", noWebGpuAdapter: "此浏览器支持 WebGPU，但没有可用的显卡：请在浏览器设置中开启硬件加速，或更新显卡驱动后重新打开。", stopped: "游戏已停止", reload: "重新加载", copy: "复制报告", copied: "已复制", log: "日志", dump: "世界快照", install: "添加到主屏幕", installOffer: "安装{name}，离线也能玩", installIos: "点按分享，再选“添加到主屏幕”", dismiss: "不用了", back: "返回", madeWith: "Made with Origin Engine" }
  : { loading: "Loading", play: "Play", tapToPlay: "Tap to play", startFailed: "The game could not start", noWebGpu: "This browser does not support WebGPU", noWebGpuDetail: "This game runs on WebGPU, which this browser does not have. Open it in a current Chrome or Edge (Windows, macOS, ChromeOS, or Android 12 and later), Safari 26 or later (macOS, iOS, iPadOS), or Firefox on Windows or macOS.", noWebGpuAdapterTitle: "No graphics adapter for WebGPU", noWebGpuAdapter: "This browser has WebGPU but found no graphics adapter it may use: turn on hardware acceleration in the browser's settings, or update the graphics driver, then open the game again.", stopped: "The game stopped", reload: "Reload", copy: "Copy report", copied: "Copied", log: "Log", dump: "World", install: "Install", installOffer: "Install {name} to play offline", installIos: "Tap Share, then Add to Home Screen", dismiss: "Not now", back: "Back", madeWith: "Made with Origin Engine" };

export function readConfig(doc: Document = document): ShellConfig {
  const text = doc.getElementById("og-config")?.textContent ?? "{}";
  const c = JSON.parse(text) as Partial<ShellConfig>;
  return {
    engine: c.engine ?? "0.0.0",
    buildId: typeof c.buildId === "string" ? c.buildId : "",
    telemetry: c.telemetry === "off" ? "off" : "on",
    build: c.build === "publish" ? "publish" : "dev",
    name: c.name ?? "Origin Engine",
    profile: c.profile === "2d" ? "2d" : "3d",
    ...(typeof c.project === "string" ? { project: c.project } : {}),
    isolation: c.isolation ?? "preferred",
    fit: c.fit === "fill" ? "fill" : "letterbox",
    aspect: typeof c.aspect === "number" && c.aspect > 0 ? c.aspect : null,
    aspectMin: typeof c.aspectMin === "number" && c.aspectMin > 0 ? c.aspectMin : null,
    aspectMax: typeof c.aspectMax === "number" && c.aspectMax > 0 ? c.aspectMax : null,
    orientation: c.orientation ?? "any",
    dprCap: typeof c.dprCap === "number" && c.dprCap > 0 ? c.dprCap : 2,
    fullscreen: c.fullscreen === "gesture" ? "gesture" : "off",
    gate: c.gate === "none" ? "none" : "tap",
    madeWith: c.madeWith !== false,
    serviceWorker: c.serviceWorker === true,
    ...(c.stageLabels && typeof c.stageLabels === "object" ? { stageLabels: c.stageLabels } : {}),
    ...(c.bootScreen === "page" ? { bootScreen: "page" as const } : {}),
    ...(c.pipelines === "async" || c.pipelines === "prewarm" ? { pipelines: c.pipelines } : {}),
    expect: { wasm: c.expect?.wasm ?? 0, cooked: c.expect?.cooked ?? 0, lanes: c.expect?.lanes ?? 0 },
    files: c.files && typeof c.files === "object" ? c.files : {},
  };
}

/**
 * The canvas's CSS box for a viewport (CSS px) and its safe-area insets:
 * `letterbox` fits `aspect` inside the safe area, centred on ink (bars above
 * and below on a taller screen, beside it on a wider one); `fill` covers the
 * viewport, held within `range` (width over height) where one is given, as a
 * letterbox at the bound it passes.
 */
export function layoutCanvas(
  viewport: { width: number; height: number },
  insets: { top: number; right: number; bottom: number; left: number },
  fit: "letterbox" | "fill",
  aspect: number | null,
  range: { readonly min: number | null; readonly max: number | null } = { min: null, max: null },
): { left: number; top: number; width: number; height: number; safeArea: { top: number; right: number; bottom: number; left: number } } {
  if (fit === "fill") {
    const a = Math.max(1, viewport.width) / Math.max(1, viewport.height);
    const bound = range.max !== null && a > range.max ? range.max : range.min !== null && a < range.min ? range.min : null;
    if (bound !== null) return layoutCanvas(viewport, insets, "letterbox", bound);
  }
  if (fit === "fill" || aspect === null) {
    return { left: 0, top: 0, width: Math.max(1, viewport.width), height: Math.max(1, viewport.height), safeArea: insets };
  }
  const aw = Math.max(1, viewport.width - insets.left - insets.right);
  const ah = Math.max(1, viewport.height - insets.top - insets.bottom);
  const width = Math.min(aw, ah * aspect);
  const height = width / aspect;
  return {
    left: insets.left + (aw - width) / 2,
    top: insets.top + (ah - height) / 2,
    width: Math.max(1, Math.round(width)),
    height: Math.max(1, Math.round(height)),
    safeArea: { top: 0, right: 0, bottom: 0, left: 0 },
  };
}

function safeAreaInsets(): { top: number; right: number; bottom: number; left: number } {
  const probe = document.createElement("div");
  probe.style.cssText = "position:fixed;visibility:hidden;pointer-events:none;padding:env(safe-area-inset-top) env(safe-area-inset-right) env(safe-area-inset-bottom) env(safe-area-inset-left)";
  document.body.append(probe);
  const s = getComputedStyle(probe);
  const out = { top: parseFloat(s.paddingTop) || 0, right: parseFloat(s.paddingRight) || 0, bottom: parseFloat(s.paddingBottom) || 0, left: parseFloat(s.paddingLeft) || 0 };
  probe.remove();
  return out;
}

const $ = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T | null;

/** What the shell keeps of the engine's log and the page's console. */
class LogTail {
  readonly lines: string[] = [];
  push(lines: readonly string[]) {
    for (const l of lines) this.lines.push(l);
    if (this.lines.length > LOG_LINES) this.lines.splice(0, this.lines.length - LOG_LINES);
  }
}

export interface PlayHandle {
  readonly shell: Shell;
  readonly config: ShellConfig;
}

export async function play(): Promise<PlayHandle> {
  const config = readConfig();
  const at = (name: string) => fileUrl(config, name, location.href);
  const query = new URLSearchParams(location.search);
  // A Play link to a room (`?room=<id>`, docs/platform-asks.md #16): the engine joins it at load
  // once a host carries rooms (the Player's room.*), og.RoomJoined answering as for roomJoin.
  const roomLink = query.get("room") ?? undefined;
  const t0 = performance.now();
  const log = new LogTail();
  // ── field telemetry (./telemetry.ts): nothing of it in a build that opts out ──
  const telemetry = config.telemetry !== "off";
  const nav = navigator as Navigator & { deviceMemory?: number; userAgentData?: { brands?: Array<{ brand: string; version: string }> } };
  let context: Context | null = telemetry
    ? {
        engine: config.engine,
        build: config.buildId,
        browser: browserOf(nav.userAgent, nav.userAgentData?.brands),
        device: deviceOf({
          ua: nav.userAgent,
          coarse: matchMedia?.("(pointer: coarse)").matches ?? false,
          touchPoints: nav.maxTouchPoints ?? 0,
          screen: { width: screen.width, height: screen.height },
          dpr: devicePixelRatio,
          ...(nav.hardwareConcurrency ? { cores: nav.hardwareConcurrency } : {}),
          ...(nav.deviceMemory ? { memory: nav.deviceMemory } : {}),
        }),
        gpu: null,
      }
    : null;
  /** The engine's memory as last read (a crash report carries it: the Worker may not answer after one). */
  let lastMemory: MemoryInfo | null = null;
  const failures: Array<{ file: string; reason: string }> = [];
  let failuresSeen = 0;
  // The inspector's Save writes no file here (development builds): to a Stage whose hello
  // lists scene_saved, the texts over the og-probe wire; otherwise a download each
  // (docs/platform-asks.md #22).
  let wire: { announce(): void; push(message: Record<string, unknown>): void } | null = null;
  const download = (name: string, text: string) => {
    const url = URL.createObjectURL(new Blob([text], { type: "text/plain" }));
    const a = document.createElement("a");
    a.href = url;
    a.download = name;
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  const onScenesSaved = (scenes: readonly SavedScene[]) => {
    const push = wire?.push;
    routeSavedScenes(scenes, { capable: !!push && player.capabilities.includes(SCENE_SAVED_CAPABILITY), push: (m) => push?.(m), download });
  };
  const onLoadFailure = (f: { file: string; reason: string }) => {
    if (!telemetry || failuresSeen >= FAILURES_MAX) return;
    failuresSeen++;
    failures.push({ file: f.file, reason: f.reason });
  };
  const playerEvents: import("./player.ts").PlayerEvents = {};
  // A top-level page the platform published (`window.__OG_GAME`) speaks Player protocol 2 to
  // its play origin over HTTP (./transport.ts); a framed page to its Player over postMessage;
  // a top-level Stage build told its own origin (the Stage overlay in this document) to its
  // own window over postMessage (./player.ts); any other page plays alone.
  const top = (() => {
    try {
      return window.parent === window;
    } catch {
      return false;
    }
  })();
  const game = top ? pageGame((globalThis as { __OG_GAME?: unknown }).__OG_GAME) : null;
  const page: PageConnection | null = game
    ? connectPage(game, {
        fetch: (url, init) => fetch(url, init),
        storage: (() => {
          try {
            return localStorage;
          } catch {
            return null;
          }
        })(),
        location,
        replaceUrl: (url) => {
          try {
            history.replaceState(history.state, "", url);
          } catch {
            /* sandboxed */
          }
        },
        now: () => Date.now(),
        clock: () => performance.now(),
        random: (n) => crypto.getRandomValues(new Uint8Array(n)),
      }, playerEvents)
    : null;
  const own = page ? null : connectPlayer(playerEvents);
  const player = page ?? own!;
  // A Player answers over postMessage: the one framing the page, or the Stage overlay in it.
  const linked = !!own?.peer;
  const stageSelf = own?.peer === "self";
  const canvas = $<HTMLCanvasElement>("og-canvas") ?? document.body.appendChild(document.createElement("canvas"));
  const boot = $("og-boot");
  const mark = boot?.querySelector<SVGSVGElement>(".og-mark") ?? null;
  const label = $("og-boot-label");
  // Fork s61: the page's own loading screen (bootScreen "page") hears the boot through window events.
  const pageBoot = config.bootScreen === "page";
  const emit = (type: string, detail?: unknown) => dispatchEvent(new CustomEvent(type, { detail }));
  if (pageBoot && boot) boot.hidden = true;
  const badge = $("og-made-with");
  if (badge) badge.hidden = !config.madeWith;
  const back = $<HTMLButtonElement>("og-back");
  // What the Player page drew around the game, drawn by the page itself: the cover's
  // backdrop behind the boot splash and Back to the Portal page that opened the game.
  const paintPage = (info: PageInfo) => {
    const backdrop = $("og-backdrop");
    const image = info.backdrop ?? info.cover;
    if (backdrop && image) {
      const img = document.createElement("img");
      img.alt = "";
      img.decoding = "async";
      // An isolated page (COEP require-corp) loads the Portal's image through CORS.
      if (globalThis.crossOriginIsolated) img.crossOrigin = "anonymous";
      img.addEventListener("load", () => backdrop.setAttribute("data-shown", ""), { once: true });
      img.src = image;
      backdrop.replaceChildren(img);
    }
    if (back && info.back) {
      const target = info.back;
      back.setAttribute("aria-label", T.back);
      back.title = T.back;
      const text = back.querySelector("span");
      if (text) text.textContent = T.back;
      back.hidden = false;
      // The UI keeps its anchored groups out of Back from now on (Viewport.covered).
      if (shellRef) shellRef.resize(box.width, box.height, box.safeArea, backCover());
      back.onclick = () => {
        let fromPortal = false;
        try {
          fromPortal = !!info.portal && new URL(document.referrer).origin === info.portal;
        } catch {
          /* no referrer */
        }
        // Only a web page: a `javascript:` or `data:` back would run on the play origin.
        let web = false;
        try {
          web = /^https?:$/.test(new URL(target, location.href).protocol);
        } catch {
          /* not a URL */
        }
        if (fromPortal && history.length > 1) history.back();
        else if (web) location.assign(target);
      };
    }
  };

  // ── boot: genesis once, then progress with the real value ────────────
  let progress = 0;
  let genesisDone = !mark || matchMedia?.("(prefers-reduced-motion: reduce)").matches;
  const showProgress = () => {
    if (pageBoot || !mark || !genesisDone || phase !== "loading") return;
    if (mark.dataset["motion"] !== "progress") mark.dataset["motion"] = "progress";
    const percent = Math.round(progress * 100);
    mark.querySelector<SVGElement>("[data-part=wipe]")?.style.setProperty("stroke-dashoffset", String(100 - percent));
    if (percent >= 100) mark.setAttribute("data-complete", "");
  };
  // The motion never delays the answer: progress shows at the end of genesis, or at once.
  mark?.querySelector('[data-part="outer"]')?.addEventListener("animationend", () => {
    genesisDone = true;
    showProgress();
  });
  setTimeout(() => {
    genesisDone = true;
    showProgress();
  }, 1200);
  let phase: "loading" | "gate" | "playing" | "failed" = "loading";
  let caption = T.loading;
  const STAGE_CLASS: Record<string, string> = { wasm: "fetch", bundle: "fetch", fetch: "fetch", transcode: "transcode", ready: "prime", prime: "prime", present: "present" };
  const setProgress = (value: number, stage?: string, fraction?: number) => {
    if (stage !== undefined) emit("og:progress", { value, stage, fraction: fraction ?? value });
    // Fork s61: the stage's caption, when the build names one (og-config stageLabels).
    const named = stage === undefined ? undefined : config.stageLabels?.[STAGE_CLASS[stage] ?? stage];
    if (named) caption = named;
    if (value <= progress) return;
    progress = Math.min(1, value);
    boot?.setAttribute("aria-valuenow", String(Math.round(progress * 100)));
    if (label) label.textContent = `${caption} ${Math.round(progress * 100)}%`;
    player.loading(progress);
    showProgress();
  };
  if (label) label.textContent = T.loading;

  // ── the error card (§6) ──────────────────────────────────────────────
  let shellRef: Shell | null = null;
  // Fork s61: settled by the first frame or a failure, whichever comes first: what the gate waits for.
  let drawnOrFailed: () => void = () => {};
  const drawn = new Promise<void>((r) => (drawnOrFailed = r));
  // `retryable` false: a reload cannot help (a browser without WebGPU).
  const fail = async (title: string, message: string, after: boolean, retryable = true, cause?: HostError) => {
    if (phase === "failed") return;
    phase = "failed";
    drawnOrFailed();
    emit("og:failed", { message });
    if (after) player.crash("EngineCrash", message, context ? crashFields(cause ?? { kind: "error", message, tick: null, lane: null, location: null }, context, lastMemory) : undefined);
    else player.fail("engine_start", message, retryable);
    let dump = "";
    if (shellRef) {
      const withTimeout = (p: Promise<unknown>) => {
        let timer: ReturnType<typeof setTimeout> | undefined;
        return Promise.race([p, new Promise((r) => (timer = setTimeout(() => r(null), 1500)))]).finally(() => clearTimeout(timer));
      };
      const world = await withTimeout(shellRef.probe(config.build === "publish" ? "state" : "dump", config.build === "publish" ? {} : { what: "world" }).catch(() => null));
      if (world) dump = JSON.stringify(world, null, 1).slice(0, 200_000);
    }
    const card = $("og-error");
    if (!card) return;
    if (mark) {
      mark.dataset["motion"] = "drift";
      mark.removeAttribute("data-complete");
    }
    boot?.removeAttribute("data-done");
    if (boot) boot.hidden = false;
    back?.removeAttribute("data-playing");
    $("og-gate")?.setAttribute("hidden", "");
    card.querySelector("h1")!.textContent = title;
    card.querySelector("p")!.textContent = message;
    // A dev build serves its maps (§Source maps): the card shows the game's .ts lines.
    if (config.build === "dev") void devStack(message).then((mapped) => mapped !== message && (card.querySelector("p")!.textContent = mapped));
    const pre = card.querySelector<HTMLElement>("#og-error-log");
    if (pre) pre.textContent = log.lines.join("\n") || "(no log lines)";
    const dumpPre = card.querySelector<HTMLElement>("#og-error-dump");
    const dumpBox = dumpPre?.closest("details") as HTMLElement | null;
    if (dumpPre && dumpBox) {
      dumpPre.textContent = dump;
      dumpBox.hidden = !dump;
    }
    card.hidden = false;
    const reload = card.querySelector<HTMLButtonElement>("[data-action=reload]");
    // A reload cannot bring WebGPU to a browser without it: no button that suggests one does.
    if (reload) reload.hidden = !retryable;
    reload?.addEventListener("click", () => location.reload());
    const copy = card.querySelector<HTMLButtonElement>("[data-action=copy]");
    copy?.addEventListener("click", async () => {
      const report = [`${config.name} — Origin Engine ${config.engine} (${config.build})`, navigator.userAgent, "", title, message, "", "log:", ...log.lines, "", dump].join("\n");
      try {
        await navigator.clipboard.writeText(report);
        copy.textContent = T.copied;
      } catch {
        /* the report stays on screen */
      }
    });
  };
  addEventListener("error", (e) => log.push([`page: ${e.message}`]));

  // ── layout ───────────────────────────────────────────────────────────
  const place = () => {
    const vv = globalThis.visualViewport;
    const viewport = { width: vv?.width ?? innerWidth, height: vv?.height ?? innerHeight };
    const box = layoutCanvas(viewport, safeAreaInsets(), config.fit, config.aspect, { min: config.aspectMin, max: config.aspectMax });
    canvas.style.position = "absolute";
    canvas.style.left = `${box.left}px`;
    canvas.style.top = `${box.top}px`;
    canvas.style.width = `${box.width}px`;
    canvas.style.height = `${box.height}px`;
    return box;
  };
  let box = place();
  // Where Back covers the canvas, in the canvas's CSS px (Viewport.covered):
  // the tucked control, as wide as it is tall (its name shows beside it only
  // for the first seconds of play); none while it is hidden or off the canvas.
  const backCover = (): { x: number; y: number; width: number; height: number } | undefined => {
    if (!back || back.hidden) return undefined;
    const r = back.getBoundingClientRect();
    const c = canvas.getBoundingClientRect();
    const x = r.left - c.left;
    const y = r.top - c.top;
    const side = r.height;
    if (side <= 0 || x + side <= 0 || y + side <= 0 || x >= c.width || y >= c.height) return undefined;
    return { x, y, width: side, height: side };
  };

  // ── the engine ───────────────────────────────────────────────────────
  const seedText = query.get(REPLAY_SEED_PARAM);
  const seed = seedText !== null && /^\d+$/.test(seedText) && Number(seedText) <= 0xffffffff ? Number(seedText) : undefined;
  const autoplayParam = query.get("autoplay");
  // `autoplay=0`: a harness drives the clock through the probe (no gate, no play).
  const harness = autoplayParam === "0";
  // WebGPU only (docs/design.md §8): without it the page says so and the engine never starts.
  const noWebGpu = await webGpuMissing();
  if (noWebGpu) {
    log.push([`webgpu: ${noWebGpu}`]);
    // How many players meet this page, and on what: the reason with the browser and the device
    // (no adapter to read), before the start failure that ends the session.
    if (context) player.sample({ phase: "unsupported", renderer: "none", reason: noWebGpu, ...context });
    // A browser with WebGPU but no adapter has it turned off (hardware acceleration, a blocked driver).
    const missing = noWebGpu.includes("navigator.gpu is missing");
    const detail = missing ? T.noWebGpuDetail : T.noWebGpuAdapter;
    await fail(missing ? T.noWebGpu : T.noWebGpuAdapterTitle, `${detail} (${noWebGpu})`, false, false);
    // play.ts catches it: the card is the page's answer.
    throw new Error(`this browser does not support WebGPU: ${noWebGpu}`);
  }
  if (config.isolation === "required" && !globalThis.crossOriginIsolated) {
    log.push(["warn: this game declares crossOriginIsolation required, but the page is not cross-origin isolated (serve it with COOP/COEP: see the headers file); it runs single-threaded"]);
  }

  // og.platform: a framed page's calls wait for the Player's hello (then the Player answers,
  // falling back to this browser's storage where it cannot); unframed, the browser's storage.
  // On the Stage (the overlay in this document) nothing goes to the shared Stage origin's
  // storage: where the overlay answers nothing, this load's memory does.
  const platformStore = stageSelf ? memoryPersistence() : storagePersistence(`og.platform:${location.pathname}`);
  let platformReady: (h: PlatformHost) => void = () => {};
  const hostReady = new Promise<PlatformHost>((r) => (platformReady = r));
  const helloForPlatform = () => platformReady(playerPlatform(player, platformStore));
  playerEvents.hello = helloForPlatform;
  if (player.hosted) helloForPlatform();
  // Without a Player the browser's storage answers; a dev page fakes the platform services
  // (every call succeeds), a publish page never does (they are `unavailable`).
  const fake = config.build === "dev" ? {} : undefined;
  if (!player.hosted && linked) setTimeout(() => platformReady(localPlatform(platformStore, fake)), HELLO_WAIT_MS);
  // A top-level platform page's calls wait for its play origin's hello; without one, the browser's storage.
  if (page)
    void page.hello.then((ok) => {
      if (!ok) return platformReady(localPlatform(platformStore, fake));
      helloForPlatform();
      // The cover and Back as soon as the play origin names them, while the engine still loads.
      if (page.page) paintPage(page.page);
    });
  const platform = linked || page ? deferredPlatform(hostReady, ["store.get", "store.set", "store.list", "telemetry"]) : localPlatform(platformStore, fake);

  let firstFrameAt = 0;
  // Ready is said once, never after the page failed; a crash before it is a start failure (Player protocol: fail before ready, crash after).
  let readied = false;
  const ready = () => {
    if (readied || phase === "failed") return;
    readied = true;
    player.ready();
  };

  // ── field telemetry's samples (docs/web-export.md §Telemetry) ─────────
  const within = <T>(p: Promise<T>, ms = 1500): Promise<T | null> => {
    let timer: ReturnType<typeof setTimeout> | undefined;
    return Promise.race([p.catch(() => null), new Promise<null>((r) => (timer = setTimeout(() => r(null), ms)))]).finally(() => clearTimeout(timer));
  };
  const readMemory = async () => {
    if (shellRef) lastMemory = memoryOf(await within(shellRef.call("memory"))) ?? lastMemory;
    return lastMemory;
  };
  /** Once, at the first frame: the boot's stages, the renderer, the lanes, the context, the memory and what did not arrive. */
  const bootSample = async () => {
    for (let i = 0; !shellRef && i < 100; i++) await new Promise((r) => setTimeout(r, 20));
    const memory = await readMemory();
    // The stages are milliseconds; the pipelines' counts (fork s60) stay on the probe's `boot`.
    const { at: _at, pipelines: _pipelines, ...stages } = shellRef?.boot ?? {};
    const lanes = shellRef?.ready.lanes;
    player.sample({
      phase: "boot",
      renderer: shellRef?.tier ?? "unknown",
      timings: { firstFrame: Math.round(firstFrameAt), boot: Math.round(firstFrameAt - t0) },
      stages,
      ...(lanes ? { lanes: { on: lanes.on, threads: lanes.threads } } : {}),
      ...context,
      ...(memory ? { memory } : {}),
      ...(failures.length ? { failures: failures.splice(0, 20) } : {}),
    });
  };
  let samples = 0;
  let lastSample = 0;
  let playingSince = 0;
  /** The frames since the last sample, when `gap` ms passed since it (SAMPLES_MAX a session). */
  const playSample = async (gap: number) => {
    const now = performance.now();
    if (!telemetry || !shellRef || !playingSince || samples >= SAMPLES_MAX || now - lastSample < gap) return;
    lastSample = now;
    const frames = (await within(shellRef.call("telemetry"))) as FrameStats | null;
    const memory = await readMemory();
    if ((!frames || frames.count === 0) && failures.length === 0) return;
    samples++;
    player.sample(playFields("play", frames, memory, failures.splice(0, 20), { ms: Math.round(now - playingSince), samples }));
  };
  /** The page goes: what is left, without waiting for the Worker (the hidden page already sent its frames). */
  const endSample = () => {
    if (!telemetry || !playingSince) return;
    player.sample(playFields("end", null, lastMemory, failures.splice(0, 20), { ms: Math.round(performance.now() - playingSince), samples }));
  };
  /** A dev build's stack mapped with the maps it serves beside its bundles; the text as it was when they are not there. */
  const devStack = async (text: string): Promise<string> => {
    const map = (f: string) => within(fetch(new URL(`./${f}.map`, location.href)).then((r) => (r.ok ? r.text() : null)));
    const [game, play] = await Promise.all([map("game.js"), map("play.js")]);
    return game || play ? mapStack(text, { "game.js": game, "play.js": play }) : text;
  };
  let shell: Shell;
  try {
    shell = await Shell.start({
      platform,
      canvas,
      worker: new Worker(at("game.js"), { type: "module" }),
      glue: at("origin-engine.js"),
      wasm: at("origin-engine_bg.wasm"),
      ...(config.expect.lanes > 0 ? { lanes: { glue: at("origin-engine-lanes.js"), wasm: at("origin-engine-lanes_bg.wasm"), bytes: config.expect.lanes } } : {}),
      root: new URL(".", location.href).href.replace(/\/$/, ""),
      basis: { js: at("basis/basis_transcoder.js"), wasm: at("basis/basis_transcoder.wasm") },
      av1: at("av1/og-av1.wasm"),
      autoplay: false,
      // Behind the gate, the first screen: one tick builds the game's UI (a harness keeps tick 0).
      prime: !harness,
      ...(seed === undefined ? {} : { seed }),
      // `?trace`: the run's trace from the boot on (the probe's `trace()` and `saveTrace()` read it).
      ...(query.has(TRACE_PARAM) ? { trace: true } : {}),
      profile: config.profile,
      ...(config.project === undefined ? {} : { project: config.project }),
      dprCap: config.dprCap,
      ...(config.pipelines ? { pipelines: config.pipelines } : {}),
      ...(config.expect.wasm + config.expect.cooked > 0 ? { expect: config.expect } : {}),
      onProgress: (v, stage, fraction) => setProgress(v, stage, fraction),
      ...(pageBoot ? { onStage: (id: string, fraction: number) => emit("og:stage", { id, fraction }) } : {}),
      onLog: (lines) => log.push(lines),
      onCrash: (message, error) => void fail(readied ? T.stopped : T.startFailed, message, readied, true, error),
      telemetry,
      onLoadFailure,
      onScenesSaved,
      onFirstFrame: () => {
        firstFrameAt = performance.now();
        performance.mark?.("og:first-frame");
        emit("og:first-frame");
        drawnOrFailed();
        ready();
        if (telemetry) void bootSample();
      },
    });
  } catch (e) {
    await fail(T.startFailed, e instanceof Error ? e.message : String(e), false);
    throw e;
  }
  shellRef = shell;
  // The device the engine renders with, as the Worker read it (never an adapter of the page's own).
  if (context) context = { ...context, gpu: shell.ready.gpu ? gpuOf(shell.ready.gpu) : null };
  performance.mark?.("og:ready");
  const lanes = shell.ready.lanes;
  log.push([lanes.on ? `lanes: on, ${lanes.threads} Workers` : `lanes: off (${lanes.why})`]);
  const resize = () => {
    box = place();
    shell.resize(box.width, box.height, box.safeArea, backCover());
  };
  let pending = 0;
  const onResize = () => {
    if (pending) cancelAnimationFrame(pending);
    pending = requestAnimationFrame(() => {
      pending = 0;
      resize();
    });
  };
  addEventListener("resize", onResize);
  globalThis.visualViewport?.addEventListener("resize", onResize);
  screen.orientation?.addEventListener?.("change", onResize);
  // The first layout's safe area reaches the UI too.
  shell.resize(box.width, box.height, box.safeArea, backCover());

  // ── the probe: in-page and over the og-probe wire ────────────────────
  // `lanes`: the mode the engine got at boot (on with its Workers, or why it runs single-threaded).
  const probe: Record<string, unknown> = { version: 5, substrate: "origin-engine", lanes: Object.freeze({ ...lanes }) };
  // `error`: what stopped the engine (a panic with its tick and lane, an error, a hang), null while it runs.
  Object.defineProperty(probe, "error", { get: () => shell.error, enumerable: true });
  // `boot`: where the Worker's boot went (src/web/worker.ts BootPhases), null before the first frame.
  Object.defineProperty(probe, "boot", { get: () => shell.boot, enumerable: true });
  for (const m of MEMBERS) probe[m] = (...args: unknown[]) => shell.probe(m, probeParams(m, args));
  // `saveScene`: the wire's save_scene; a page writes no file, so it answers the text.
  probe["saveScene"] = () => shell.probe("save_scene", { write: false });
  // `memory`: what the engine holds (ProbeMemory), from the Worker in every build (a publish probe has no `memory`).
  probe["memory"] = () => shell.call("memory");
  // `trace`: the run's Chrome trace, as `og run --trace` writes it ("take", "start", "stop");
  // `saveTrace`: the same, downloaded as a file (`?trace` records from the boot).
  probe["trace"] = (action?: "take" | "start" | "stop") => shell.trace(action);
  probe["saveTrace"] = (name?: string) => shell.saveTrace(name);
  (globalThis as Record<string, unknown>)[PROBE_GLOBAL] = Object.freeze(probe);
  // The played frames' display intervals and engine work, ms (`og run --device` reads them; `true` resets).
  (globalThis as Record<string, unknown>)[TIMINGS_GLOBAL] = (reset?: boolean) => shell.call("timings", reset === true);
  wire = mountWire({
    probe: () => (method, params) => shell.probe(method, params ?? {}),
    targetSize: () => ({ width: canvas.width || Math.round(box.width * Math.min(devicePixelRatio || 1, config.dprCap)), height: canvas.height || Math.round(box.height * Math.min(devicePixelRatio || 1, config.dprCap)) }),
    canvas: () => canvas,
  });

  // ── audio, wake lock, fullscreen, visibility ─────────────────────────
  const audio = new AudioShell({ worker: shell.worker, worklet: at("audio-worklet.js") });
  audio.listen();
  let wake: { release(): Promise<void> } | null = null;
  const lockWake = async () => {
    try {
      const nav = navigator as { wakeLock?: { request(kind: "screen"): Promise<{ release(): Promise<void> }> } };
      if (nav.wakeLock && document.visibilityState === "visible" && !wake) {
        wake = await nav.wakeLock.request("screen");
        (wake as unknown as EventTarget).addEventListener?.("release", () => (wake = null));
      }
    } catch {
      /* refused (battery saver, no gesture): play goes on */
    }
  };
  const fullscreen = async () => {
    if (config.fullscreen !== "gesture" || player.framed || document.fullscreenElement) return;
    try {
      await document.documentElement.requestFullscreen?.({ navigationUI: "hide" });
      if (config.orientation !== "any") await (screen.orientation as unknown as { lock?(o: string): Promise<void> }).lock?.(config.orientation);
    } catch {
      /* fullscreen or the lock refused: the page plays as laid out */
    }
  };
  let wanted = false;
  let pausedByPlayer = false;
  const start = () => {
    if (phase === "failed") return;
    phase = "playing";
    performance.mark?.("og:start");
    wanted = true;
    $("og-gate")?.setAttribute("hidden", "");
    badge?.setAttribute("data-playing", "");
    back?.setAttribute("data-playing", "");
    page?.start();
    if (telemetry && !playingSince) {
      playingSince = lastSample = performance.now();
      setInterval(() => void playSample(SAMPLE_EVERY_MS), SAMPLE_EVERY_MS / 4);
    }
    if (!harness) shell.play(!pausedByPlayer && document.visibilityState === "visible");
    void lockWake();
  };
  const pauseFor = (on: boolean) => {
    if (!wanted || harness || phase === "failed") return;
    const run = !on && document.visibilityState === "visible" && !pausedByPlayer;
    if (run === shell.playing) return;
    shell.play(run);
    void audio.suspend(!run);
    if (run) void lockWake();
    else {
      void wake?.release();
      wake = null;
    }
  };
  document.addEventListener("visibilitychange", () => {
    pauseFor(document.visibilityState !== "visible");
    // A hidden page may never come back: its frames go now (the clock is paused, so none are lost).
    if (document.visibilityState !== "visible") void playSample(SAMPLE_HIDDEN_MS);
  });
  addEventListener("pagehide", () => {
    pauseFor(true);
    endSample();
    page?.end({ ready: firstFrameAt > 0, failed: phase === "failed" });
  });
  pollGamepads((e) => shell.inject(e));

  // A Player owns the gate: start on its `start` (or a hello that says play began).
  // A top-level page owns its own gate; its hello only names what its play origin answers.
  let byPlayer = player.hosted && !page;
  if (page) {
    playerEvents.hello = () => {
      helloForPlatform();
      shell.setPlatform(playerPlatform(player, platformStore));
    };
    if (page.hosted) playerEvents.hello({ started: false, paused: false, capabilities: page.capabilities });
  } else playerEvents.hello = (m) => {
    byPlayer = true;
    helloForPlatform();
    // What it answers may have changed with this hello.
    shell.setPlatform(playerPlatform(player, platformStore));
    // Rooms ride the Player's postMessage link (pushed events): a framed Player's, never the HTTP page's.
    const rooms = playerRooms(player);
    shell.setRooms(rooms, roomLink);
    playerEvents.room = (e) => rooms?.push(e);
    pausedByPlayer = m.paused;
    if (m.started && phase === "gate") start();
  };
  // The usual case with a real Player: it answered the shell's hello while the engine loaded,
  // and the handler above was not installed yet (the first one only released og.platform).
  // What that hello says applies now: its rooms, and a Play link's room through them.
  if (!page && player.hosted) playerEvents.hello({ started: player.started, paused: player.paused, capabilities: player.capabilities });
  playerEvents.start = () => {
    byPlayer = true;
    if (phase === "gate") start();
  };
  playerEvents.gesture = () => void audio.unlock().catch(() => {});
  // A Player's notices (docs/platform-asks.md #19) and, on a dev page, the fake's ad pausing the game as the page's own pause does.
  playerEvents.notice = (n) => shell.notify(n as PlatformNotice);
  shell.onPlatformPause(pauseFor);
  playerEvents.pause = () => {
    pausedByPlayer = true;
    pauseFor(true);
  };
  playerEvents.resume = () => {
    pausedByPlayer = false;
    pauseFor(false);
  };
  pausedByPlayer = player.paused;
  // A framed page (or the Stage overlay's own document) waits briefly for a Player's hello before drawing its own gate.
  if (linked && !byPlayer) await new Promise((r) => setTimeout(r, 300));
  // A Play link to a room on a page no Player carries rooms for: the game hears why it is in none
  // (og.RoomJoined, ok false); a Player's hello later joins it.
  if (roomLink && !byPlayer) shell.setRooms(null, roomLink);

  // ── ready: reveal, then the gate ─────────────────────────────────────
  // Fork s61: the gate shows once the first frame is drawn (behind it: the first tick, the
  // textures' uploads, the pipelines), not when the bytes are in: Play then leads to a picture.
  // With no gate (og-config gate "none") the game starts at once instead: it runs its own loading
  // screen behind which the first frames draw (one bar from the page's first paint to the game's click).
  if (!harness && config.gate !== "none" && shell.ready.tier !== "none") await drawn;
  setProgress(1);
  boot?.setAttribute("data-done", "");
  setTimeout(() => boot && phase !== "failed" && (boot.hidden = true), 300);
  if (!firstFrameAt) {
    // The Worker drew the first frame behind the gate; a page without a renderer is ready now.
    setTimeout(() => {
      if (!firstFrameAt) ready();
    }, 2000);
  }
  if (harness) {
    phase = "playing";
  } else if (byPlayer) {
    phase = "gate";
    if (player.started) start();
  } else if (config.gate === "none") {
    start();
  } else {
    phase = "gate";
    const gate = $("og-gate");
    const button = gate?.querySelector("button");
    if (button) button.textContent = matchMedia?.("(pointer: coarse)").matches ? T.tapToPlay : T.play;
    if (gate) {
      gate.hidden = false;
      const go = (e: Event) => {
        e.preventDefault();
        gate.removeEventListener("pointerup", go);
        removeEventListener("keydown", key, true);
        void audio.unlock().catch((err) => log.push([`warn: audio: ${err instanceof Error ? err.message : String(err)}`]));
        void fullscreen();
        // The gate's click is the gesture the browser grants pointer lock to:
        // a game that wants it from its first screen locks now.
        shell.gesture();
        start();
        canvas.focus?.({ preventScroll: true });
      };
      // Enter or Space anywhere passes the gate too; the key does not reach the game.
      const key = (e: KeyboardEvent) => {
        if (e.code !== "Enter" && e.code !== "Space" && e.code !== "NumpadEnter") return;
        e.stopImmediatePropagation();
        go(e);
      };
      gate.addEventListener("pointerup", go);
      addEventListener("keydown", key, true);
    } else start();
  }

  // The og-probe wire's hello tells the host the page can be driven: said once the gate takes a tap
  // (or play began), so a host's tap after it reaches the gate. With the gate after the first frame
  // (fork s61), a hello said at ready came before the gate was up, and a tap then was lost.
  wire?.announce();

  // ── PWA ──────────────────────────────────────────────────────────────
  // The shared Stage origin installs no service worker.
  if (config.serviceWorker && !harness && !stageSelf) {
    // The variant the engine booted, which the service worker caches whole (sw.ts `og-cache`).
    const booted = lanes.on ? ["origin-engine-lanes.js", "origin-engine-lanes_bg.wasm"] : ["origin-engine.js", "origin-engine_bg.wasm"];
    void installPwa(player.framed, log, booted.map(at));
  }
  return { shell, config };
}

/**
 * Register the service worker and, once it is active, have it cache the
 * engine variant this page booted (`booted`: its files' URLs), whatever it
 * guessed when it installed; show the install hint only when the criteria hold.
 */
async function installPwa(framed: boolean, log: LogTail, booted: readonly string[]): Promise<void> {
  if (!("serviceWorker" in navigator) || !isSecureContext) return;
  try {
    await navigator.serviceWorker.register("./sw.js", { scope: "./" });
  } catch (e) {
    log.push([`warn: service worker: ${e instanceof Error ? e.message : String(e)}`]);
    return;
  }
  void navigator.serviceWorker.ready
    .then((reg) => reg.active?.postMessage({ type: "og-cache", files: booted }))
    .catch((e) => log.push([`warn: service worker: ${e instanceof Error ? e.message : String(e)}`]));
  const standalone = matchMedia?.("(display-mode: standalone)").matches || (navigator as { standalone?: boolean }).standalone === true;
  const manifest = document.querySelector('link[rel="manifest"]');
  let dismissed = false;
  try {
    dismissed = localStorage.getItem(INSTALL_DISMISSED) === "1";
  } catch {
    /* storage refused: ask once per load */
  }
  if (framed || standalone || !manifest || dismissed) return;
  const hint = $("og-install");
  if (!hint) return;
  const text = hint.querySelector("span");
  const [act, no] = Array.from(hint.querySelectorAll("button"));
  const dismiss = () => {
    hint.hidden = true;
    try {
      localStorage.setItem(INSTALL_DISMISSED, "1");
    } catch {
      /* not remembered */
    }
  };
  no?.addEventListener("click", dismiss);
  if (no) no.textContent = T.dismiss;
  // Chromium: the browser decides the criteria hold and fires beforeinstallprompt, often
  // before the engine has booted: index.html's head keeps that event in `__ogInstallPrompt`.
  // The label names the game as the manifest does (short_name, else name: the same as this meta).
  const game = document.querySelector<HTMLMetaElement>('meta[name="apple-mobile-web-app-title"]')?.content || document.title;
  const offer = (prompt: Event & { prompt(): Promise<void> }) => {
    if (text) text.textContent = T.installOffer.replace("{name}", game);
    if (act) {
      act.hidden = false;
      act.textContent = T.install;
      act.onclick = () => {
        hint.hidden = true;
        void prompt.prompt();
      };
    }
    hint.hidden = false;
  };
  const early = (window as { __ogInstallPrompt?: Event & { prompt(): Promise<void> } }).__ogInstallPrompt;
  if (early) offer(early);
  addEventListener("beforeinstallprompt", (e) => {
    e.preventDefault();
    offer(e as Event & { prompt(): Promise<void> });
  });
  // iOS Safari has no prompt: the hint names the Share sheet.
  const ua = navigator.userAgent;
  const ios = /iP(hone|ad|od)/.test(ua) || (/Macintosh/.test(ua) && navigator.maxTouchPoints > 1);
  if (ios && /Safari/.test(ua) && !/Chrome|Chromium|CriOS|FxiOS|EdgiOS|Android/.test(ua)) {
    if (text) text.textContent = T.installIos;
    if (act) act.hidden = true;
    hint.hidden = false;
  }
}
