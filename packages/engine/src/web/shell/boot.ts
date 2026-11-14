// The shell's boot (shell.ts): the canvas transferred to the engine Worker,
// the `start` message, and the Worker's answer, `ready` or why it did not
// start; and what the shell reports when the engine stops.

import { createRing } from "../input.ts";
import { laneChoice } from "../lanes.ts";
import { createRpcRings, type RpcRings } from "../rpc-ring.ts";
import type { ShellOptions } from "../shell.ts";
import { pagePreferences, pixelSize, storedLowPower } from "./viewport.ts";

/**
 * Why the engine stopped, as the Worker or the shell's frame deadline
 * reported it (docs/web-export.md §Errors): `panic`, a Rust panic on the
 * engine's thread or inside a lane (`lane` names the lanes site; the wasm
 * recorded it before it trapped); `error`, a failing system, a trap without
 * a panic or an uncaught error; `hang`, a playing engine Worker that sent
 * no frame for FRAME_DEADLINE_MS.
 */
export interface HostError {
  readonly kind: "panic" | "error" | "hang";
  readonly message: string;
  /** The tick the world was running, when known. */
  readonly tick: number | null;
  /** The lanes site a panic happened in (`anim`, `extract`, `cull`, `physics`); null on the engine's thread. */
  readonly lane: string | null;
  /** A panic's `file:line:column`. */
  readonly location: string | null;
}

/** What the Worker reported when the engine was ready. */
export interface Ready {
  /** The mode the engine got: lanes on with the pool's threads, or why it runs single-threaded. */
  readonly lanes: { readonly on: boolean; readonly threads: number; readonly why: string };
  readonly tier: string;
  readonly transport: "ring" | "message";
  readonly rpc: "ring" | "message";
  /** The encodings the bundle's Basis textures reached the renderer in (`bc7+bc5`, `astc4x4`, `etc2+eacrg11`, `rgba8`), or null without a bundle. */
  readonly textureFormat: string | null;
  readonly files: number;
  readonly textures: number;
  /** With `telemetry`: the device the renderer runs on (boot.ts deviceReport; shell/telemetry.ts gpuOf reads it). */
  readonly gpu?: { readonly info?: unknown; readonly limits?: unknown; readonly features?: readonly string[] };
}

/**
 * How long the boot may go without a word from the Worker (progress, a log
 * line) before it fails: generous, since a slow network moves the progress
 * and only a hung Worker says nothing for this long.
 */
export const BOOT_DEADLINE_MS = 120_000;

/**
 * Why this browser cannot run the engine, or null when it can. The engine
 * draws on WebGPU only, with no fallback (docs/design.md §8): a page with
 * no `navigator.gpu` (an older browser, or an insecure context: WebGPU
 * needs https or localhost) or no adapter is not supported, and the player
 * shell says so instead of starting the engine. A device the adapter then
 * refuses is the engine Worker's start failure (`og_web_attach_canvas`).
 */
export async function webGpuMissing(): Promise<string | null> {
  const gpu = (globalThis.navigator as { gpu?: { requestAdapter(): Promise<unknown> } } | undefined)?.gpu;
  if (!gpu) {
    return globalThis.isSecureContext === false
      ? "this page is not a secure context (WebGPU needs https or localhost), so navigator.gpu is missing"
      : "navigator.gpu is missing";
  }
  try {
    return (await gpu.requestAdapter()) ? null : "the browser found no WebGPU adapter";
  } catch (e) {
    return `the browser's WebGPU did not start: ${e instanceof Error ? e.message : String(e)}`;
  }
}

/** What the boot left the shell: the Worker's ready, the rings when the page is cross-origin isolated, the canvas's DPR and where the power preference is kept. */
export interface Booted {
  readonly ready: Ready;
  readonly ring: SharedArrayBuffer | undefined;
  readonly rpc: RpcRings | undefined;
  readonly dpr: number;
  readonly displayKey: string;
}

/** Starts the engine in `options.worker` on `options.canvas`; resolves once it is ready, rejects (the Worker stopped) when it cannot start. */
export async function bootWorker(options: ShellOptions): Promise<Booted> {
  const { canvas, worker } = options;
  const size = pixelSize(canvas.clientWidth || 1, canvas.clientHeight || 1, globalThis.devicePixelRatio || 1, options.dprCap);
  const dpr = size.dpr;
  canvas.width = size.width;
  canvas.height = size.height;
  // Logs and a crash during the boot reach the page as well as after it.
  const early = (e: MessageEvent) => {
    const m = e.data as { type: string; lines?: string[] };
    if (m.type === "log" && m.lines) options.onLog?.(m.lines);
    if (m.type === "load-failure") options.onLoadFailure?.(m as unknown as { file: string; reason: string });
  };
  worker.addEventListener("message", early);
  const offscreen = canvas.transferControlToOffscreen();
  const ring = globalThis.crossOriginIsolated ? createRing() : undefined;
  const rpc = globalThis.crossOriginIsolated ? createRpcRings() : undefined;
  // The boot fails (and the Worker stops) when the Worker says so, when it
  // does not load or throws while loading (game.js missing, a syntax error,
  // a top-level throw: an `error` event and no message), when its message
  // cannot be read, or when it says nothing for BOOT_DEADLINE_MS.
  const ready = new Promise<Ready>((resolve, reject) => {
    let deadline: ReturnType<typeof setTimeout> | undefined;
    const settle = () => {
      clearTimeout(deadline);
      worker.removeEventListener("message", on);
      worker.removeEventListener("error", onError);
      worker.removeEventListener("messageerror", onMessageError);
    };
    const failed = (message: string, error?: HostError) => {
      settle();
      worker.terminate();
      reject(Object.assign(new Error(message), { error: error ?? { kind: "error", message, tick: null, lane: null, location: null } }));
    };
    const arm = () => {
      clearTimeout(deadline);
      deadline = setTimeout(() => failed(`the engine Worker said nothing for ${BOOT_DEADLINE_MS / 1000} s while starting`), BOOT_DEADLINE_MS);
    };
    const onError = (e: Event) => {
      const message = (e as ErrorEvent).message;
      failed(message ? `the engine Worker failed: ${message}` : "the engine Worker did not load (game.js is missing, or it failed while loading)");
    };
    const onMessageError = () => failed("a message from the engine Worker could not be read");
    const on = (e: MessageEvent) => {
      arm();
      const m = e.data as { type: string; value?: number; stage?: string; fraction?: number; message?: string } & Partial<Ready>;
      if (m.type === "progress") options.onProgress?.(m.value ?? 0, m.stage ?? "", m.fraction);
      else if (m.type === "ready") {
        settle();
        resolve({
          lanes: m.lanes ?? { on: false, threads: 1, why: "the Worker did not report lanes" },
          tier: m.tier ?? "none",
          transport: m.transport ?? "message",
          rpc: m.rpc ?? "message",
          textureFormat: m.textureFormat ?? null,
          files: m.files ?? 0,
          textures: m.textures ?? 0,
          ...(m.gpu ? { gpu: m.gpu } : {}),
        });
      } else if (m.type === "error") failed(m.message ?? "the engine did not start", (m as { error?: HostError }).error);
    };
    worker.addEventListener("message", on);
    worker.addEventListener("error", onError);
    worker.addEventListener("messageerror", onMessageError);
    arm();
  });
  const choice = laneChoice(!!options.lanes, !!globalThis.crossOriginIsolated, globalThis.navigator?.hardwareConcurrency ?? 0);
  const displayKey = options.displayKey ?? `og.display:${globalThis.location?.pathname ?? "/"}`;
  const lowPower = options.lowPower ?? storedLowPower(displayKey);
  worker.postMessage(
    {
      type: "start",
      lowPower,
      glue: options.glue,
      wasm: options.wasm,
      ...("threads" in choice ? { lanes: { ...options.lanes!, threads: choice.threads } } : { lanesWhy: choice.why }),
      canvas: offscreen,
      seed: options.seed,
      ring,
      rpc,
      autoplay: options.autoplay ?? true,
      root: options.root,
      basis: options.basis,
      ...(options.av1 ? { av1: options.av1 } : {}),
      profile: options.profile,
      ...(options.project === undefined ? {} : { project: options.project }),
      locale: options.locale ?? globalThis.navigator?.language,
      ...(options.expect ? { expect: options.expect } : {}),
      ...(options.prime ? { prime: true } : {}),
      ...(options.trace ? { trace: true } : {}),
      ...(options.telemetry ? { telemetry: true } : {}),
      ...(options.onStage ? { stages: true } : {}),
      ...(options.pipelines ? { pipelines: options.pipelines } : {}),
      viewport: { width: canvas.clientWidth || canvas.width / dpr, height: canvas.clientHeight || canvas.height / dpr, dpr, coarsePointer: globalThis.matchMedia?.("(pointer: coarse)").matches ?? false, ...pagePreferences() },
    },
    [offscreen],
  );
  let answered: Ready;
  try {
    answered = await ready;
  } finally {
    worker.removeEventListener("message", early);
  }
  return { ready: answered, ring, rpc, dpr, displayKey };
}
