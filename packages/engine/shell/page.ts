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
//   the UI), the device pixel ratio capped, resize and orientation changes
//   resized in the Worker;
// - visibility: a hidden page pauses the clock and the audio and resumes both;
// - input: keyboard, pointers (touch ids carried), gamepads as keys;
// - the error card (§6): the reason, the last 200 log lines and the world
//   dump (state in a publish build), with Reload and Copy report;
// - the probe: `globalThis.__runtime_probe__` (protocol 5, every member a
//   Promise) and the platform's `og-probe` postMessage wire;
// - PWA: the service worker (publish builds) and the add-to-home-screen hint
//   only when the install criteria hold;
// - "Made with Origin Engine" in the corner unless the project turns it off.

import { Shell } from "../src/web/shell.ts";
import { AudioShell } from "../src/web/audio.ts";
import { pollGamepads } from "../src/web/gamepad.ts";
import { mountWire } from "./wire.ts";
import { connectPlayer } from "./player.ts";
import { connectPage, pageGame, type PageConnection, type PageInfo } from "./transport.ts";
import { deferredPlatform, HELLO_WAIT_MS, playerPlatform, storagePersistence } from "./platform.ts";
import { localPlatform, memoryPersistence, type PlatformHost } from "../src/platform-local.ts";

/** The page's configuration (`<script type="application/json" id="og-config">`, written by og build). */
export interface ShellConfig {
  readonly engine: string;
  readonly build: "publish" | "dev";
  readonly name: string;
  readonly profile: "2d" | "3d";
  readonly isolation: "required" | "preferred" | "off";
  readonly fit: "letterbox" | "fill";
  /** Width over height for letterbox; null takes the screen's. */
  readonly aspect: number | null;
  readonly orientation: "any" | "landscape" | "portrait";
  readonly dprCap: number;
  readonly fullscreen: "gesture" | "off";
  readonly gate: "tap" | "none";
  readonly madeWith: boolean;
  readonly serviceWorker: boolean;
  readonly expect: { readonly wasm: number; readonly cooked: number };
}

const PROBE_GLOBAL = "__runtime_probe__";
const REPLAY_SEED_PARAM = "replay-seed";
const LOG_LINES = 200;
const INSTALL_DISMISSED = "og-install-dismissed";

const MEMBERS = [
  "hello", "tick", "state", "advance", "pause", "resume", "input", "accepted", "pick", "entity",
  "bounds", "patch", "run", "capture", "sweep", "reload", "snapshot", "restore", "dump", "commands",
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
    case "sweep":
    case "dump":
      return (args[0] as Record<string, unknown>) ?? {};
    default:
      return {};
  }
}

const zh = /^zh\b/i.test(globalThis.navigator?.language ?? "");
const T = zh
  ? { loading: "正在加载", play: "点击开始", tapToPlay: "点击开始", startFailed: "游戏无法启动", stopped: "游戏已停止", reload: "重新加载", copy: "复制报告", copied: "已复制", log: "日志", dump: "世界快照", install: "添加到主屏幕", installIos: "点按分享，再选“添加到主屏幕”", dismiss: "不用了", back: "返回", madeWith: "Made with Origin Engine" }
  : { loading: "Loading", play: "Play", tapToPlay: "Tap to play", startFailed: "The game could not start", stopped: "The game stopped", reload: "Reload", copy: "Copy report", copied: "Copied", log: "Log", dump: "World", install: "Install", installIos: "Tap Share, then Add to Home Screen", dismiss: "Not now", back: "Back", madeWith: "Made with Origin Engine" };

export function readConfig(doc: Document = document): ShellConfig {
  const text = doc.getElementById("og-config")?.textContent ?? "{}";
  const c = JSON.parse(text) as Partial<ShellConfig>;
  return {
    engine: c.engine ?? "0.0.0",
    build: c.build === "publish" ? "publish" : "dev",
    name: c.name ?? "Origin Engine",
    profile: c.profile === "2d" ? "2d" : "3d",
    isolation: c.isolation ?? "preferred",
    fit: c.fit === "fill" ? "fill" : "letterbox",
    aspect: typeof c.aspect === "number" && c.aspect > 0 ? c.aspect : null,
    orientation: c.orientation ?? "any",
    dprCap: typeof c.dprCap === "number" && c.dprCap > 0 ? c.dprCap : 2,
    fullscreen: c.fullscreen === "gesture" ? "gesture" : "off",
    gate: c.gate === "none" ? "none" : "tap",
    madeWith: c.madeWith !== false,
    serviceWorker: c.serviceWorker === true,
    expect: { wasm: c.expect?.wasm ?? 0, cooked: c.expect?.cooked ?? 0 },
  };
}

/** The canvas's CSS box for a viewport (CSS px) and its safe-area insets. */
export function layoutCanvas(
  viewport: { width: number; height: number },
  insets: { top: number; right: number; bottom: number; left: number },
  fit: "letterbox" | "fill",
  aspect: number | null,
): { left: number; top: number; width: number; height: number; safeArea: { top: number; right: number; bottom: number; left: number } } {
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
  const query = new URLSearchParams(location.search);
  const t0 = performance.now();
  const log = new LogTail();
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
      back.onclick = () => {
        let fromPortal = false;
        try {
          fromPortal = !!info.portal && new URL(document.referrer).origin === info.portal;
        } catch {
          /* no referrer */
        }
        if (fromPortal && history.length > 1) history.back();
        else location.assign(target);
      };
    }
  };

  // ── boot: genesis once, then progress with the real value ────────────
  let progress = 0;
  let genesisDone = !mark || matchMedia?.("(prefers-reduced-motion: reduce)").matches;
  const showProgress = () => {
    if (!mark || !genesisDone || phase !== "loading") return;
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
  const setProgress = (value: number) => {
    if (value <= progress) return;
    progress = Math.min(1, value);
    boot?.setAttribute("aria-valuenow", String(Math.round(progress * 100)));
    if (label) label.textContent = `${T.loading} ${Math.round(progress * 100)}%`;
    player.loading(progress);
    showProgress();
  };
  if (label) label.textContent = T.loading;

  // ── the error card (§6) ──────────────────────────────────────────────
  let shellRef: Shell | null = null;
  const fail = async (title: string, message: string, after: boolean) => {
    if (phase === "failed") return;
    phase = "failed";
    if (after) player.crash("EngineCrash", message);
    else player.fail("engine_start", message, true);
    let dump = "";
    if (shellRef) {
      const withTimeout = (p: Promise<unknown>) => Promise.race([p, new Promise((r) => setTimeout(() => r(null), 1500))]);
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
    const pre = card.querySelector<HTMLElement>("#og-error-log");
    if (pre) pre.textContent = log.lines.join("\n") || "(no log lines)";
    const dumpPre = card.querySelector<HTMLElement>("#og-error-dump");
    const dumpBox = dumpPre?.closest("details") as HTMLElement | null;
    if (dumpPre && dumpBox) {
      dumpPre.textContent = dump;
      dumpBox.hidden = !dump;
    }
    card.hidden = false;
    card.querySelector<HTMLButtonElement>("[data-action=reload]")?.addEventListener("click", () => location.reload());
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
    const box = layoutCanvas(viewport, safeAreaInsets(), config.fit, config.aspect);
    canvas.style.position = "absolute";
    canvas.style.left = `${box.left}px`;
    canvas.style.top = `${box.top}px`;
    canvas.style.width = `${box.width}px`;
    canvas.style.height = `${box.height}px`;
    return box;
  };
  let box = place();

  // ── the engine ───────────────────────────────────────────────────────
  const seedText = query.get(REPLAY_SEED_PARAM);
  const seed = seedText !== null && /^\d+$/.test(seedText) && Number(seedText) <= 0xffffffff ? Number(seedText) : undefined;
  const autoplayParam = query.get("autoplay");
  // `autoplay=0`: a harness drives the clock through the probe (no gate, no play).
  const harness = autoplayParam === "0";
  let webgl = query.get("webgl") === "1";
  if (!webgl) {
    try {
      const gpu = (navigator as { gpu?: { requestAdapter(): Promise<unknown> } }).gpu;
      webgl = !gpu || !(await gpu.requestAdapter());
    } catch {
      webgl = true;
    }
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
  else if (linked) setTimeout(() => platformReady(localPlatform(platformStore)), HELLO_WAIT_MS);
  // A top-level platform page's calls wait for its play origin's hello; without one, the browser's storage.
  if (page)
    void page.hello.then((ok) => {
      if (!ok) return platformReady(localPlatform(platformStore));
      helloForPlatform();
      // The cover and Back as soon as the play origin names them, while the engine still loads.
      if (page.page) paintPage(page.page);
    });
  const platform = linked || page ? deferredPlatform(hostReady, ["store.get", "store.set", "store.list", "telemetry"]) : localPlatform(platformStore);

  let firstFrameAt = 0;
  let shell: Shell;
  try {
    shell = await Shell.start({
      platform,
      canvas,
      worker: new Worker(new URL("./game.js", location.href), { type: "module" }),
      glue: new URL("./origin-engine.js", location.href).href,
      wasm: new URL("./origin-engine_bg.wasm", location.href).href,
      root: new URL(".", location.href).href.replace(/\/$/, ""),
      basis: { js: new URL("./basis/basis_transcoder.js", location.href).href, wasm: new URL("./basis/basis_transcoder.wasm", location.href).href },
      webgl,
      autoplay: false,
      // Behind the gate, the first screen: one tick builds the game's UI (a harness keeps tick 0).
      prime: !harness,
      ...(seed === undefined ? {} : { seed }),
      profile: config.profile,
      dprCap: config.dprCap,
      ...(config.expect.wasm + config.expect.cooked > 0 ? { expect: config.expect } : {}),
      onProgress: (v) => setProgress(v),
      onLog: (lines) => log.push(lines),
      onCrash: (message) => void fail(T.stopped, message, true),
      onFirstFrame: () => {
        firstFrameAt = performance.now();
        performance.mark?.("og:first-frame");
        player.ready();
        player.sample({ renderer: shell?.tier ?? "unknown", timings: { firstFrame: Math.round(firstFrameAt), boot: Math.round(firstFrameAt - t0) } });
      },
    });
  } catch (e) {
    await fail(T.startFailed, e instanceof Error ? e.message : String(e), false);
    throw e;
  }
  shellRef = shell;
  performance.mark?.("og:ready");
  const resize = () => {
    box = place();
    shell.resize(box.width, box.height, box.safeArea);
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
  shell.resize(box.width, box.height, box.safeArea);

  // ── the probe: in-page and over the og-probe wire ────────────────────
  const probe: Record<string, unknown> = { version: 5, substrate: "origin-engine" };
  for (const m of MEMBERS) probe[m] = (...args: unknown[]) => shell.probe(m, probeParams(m, args));
  (globalThis as Record<string, unknown>)[PROBE_GLOBAL] = Object.freeze(probe);
  mountWire({
    probe: () => (method, params) => shell.probe(method, params ?? {}),
    targetSize: () => ({ width: canvas.width || Math.round(box.width * Math.min(devicePixelRatio || 1, config.dprCap)), height: canvas.height || Math.round(box.height * Math.min(devicePixelRatio || 1, config.dprCap)) }),
    canvas: () => canvas,
  });

  // ── audio, wake lock, fullscreen, visibility ─────────────────────────
  const audio = new AudioShell({ worker: shell.worker, worklet: new URL("./audio-worklet.js", location.href).href });
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
  document.addEventListener("visibilitychange", () => pauseFor(document.visibilityState !== "visible"));
  addEventListener("pagehide", () => {
    pauseFor(true);
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
    pausedByPlayer = m.paused;
    if (m.started && phase === "gate") start();
  };
  playerEvents.start = () => {
    byPlayer = true;
    if (phase === "gate") start();
  };
  playerEvents.gesture = () => void audio.unlock().catch(() => {});
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

  // ── ready: reveal, then the gate ─────────────────────────────────────
  setProgress(1);
  boot?.setAttribute("data-done", "");
  setTimeout(() => boot && phase !== "failed" && (boot.hidden = true), 300);
  if (!firstFrameAt) {
    // The Worker drew the first frame behind the gate; a page without a renderer is ready now.
    setTimeout(() => {
      if (!firstFrameAt) player.ready();
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

  // ── PWA ──────────────────────────────────────────────────────────────
  // The shared Stage origin installs no service worker.
  if (config.serviceWorker && !harness && !stageSelf) void installPwa(player.framed, log);
  return { shell, config };
}

/** Register the service worker; show the install hint only when the criteria hold. */
async function installPwa(framed: boolean, log: LogTail): Promise<void> {
  if (!("serviceWorker" in navigator) || !isSecureContext) return;
  try {
    await navigator.serviceWorker.register("./sw.js", { scope: "./" });
  } catch (e) {
    log.push([`warn: service worker: ${e instanceof Error ? e.message : String(e)}`]);
    return;
  }
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
  // Chromium: the browser decides the criteria hold and fires beforeinstallprompt.
  addEventListener("beforeinstallprompt", (e) => {
    e.preventDefault();
    const prompt = e as Event & { prompt(): Promise<void> };
    if (text) text.textContent = T.install;
    if (act) {
      act.hidden = false;
      act.textContent = T.install;
      act.onclick = () => {
        hint.hidden = true;
        void prompt.prompt();
      };
    }
    hint.hidden = false;
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
