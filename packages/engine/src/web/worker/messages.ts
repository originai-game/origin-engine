// What the shell sends the engine Worker (worker.ts), and the Worker's global
// scope as it uses it.

import type { PlatformAnswer, PlatformNotice } from "../../platform-local.ts";
import type { AudioLatencyMessage, AudioMessage } from "../audio.ts";
import type { InjectedInput } from "../input.ts";
import type { RpcRings } from "../rpc-ring.ts";

export interface Start {
  type: "start";
  glue: string;
  wasm: string;
  /**
   * The lanes variant (the atomics build, its own glue and module) and the
   * Worker pool's threads, when the shell chose lanes (laneChoice in
   * shell.ts); without it, or when it fails to start, `glue`/`wasm` run
   * single-threaded.
   */
  lanes?: { glue: string; wasm: string; threads: number; bytes?: number };
  /** Why the shell did not choose lanes, reported back in ready. */
  lanesWhy?: string;
  canvas?: OffscreenCanvas;
  seed?: number;
  ring?: SharedArrayBuffer;
  rpc?: RpcRings;
  autoplay?: boolean;
  /** Fork s61: the page draws its own loading screen and hears the game's stages (worker globals ogBootStage, ogBootIdle). */
  stages?: boolean;
  /**
   * Fork s60: project.ogt `publish { pipelines }`, absent for `auto` (pipelines.ts `choose`): `sync`; `async`, the frame pipelines with createRenderPipelineAsync;
   * `prewarm`, only the warm-up behind the gate's; absent: synchronously.
   */
  pipelines?: "sync" | "async" | "prewarm";
  /** The URL base whose `cooked/` bundle the engine opens (`.` next to the page). */
  root?: string;
  /** The Basis transcoder's script and module. */
  basis?: { js: string; wasm: string };
  /** The rav1d fallback decoder's module URL (dist/av1/og-av1.wasm by default). */
  av1?: string;
  /** The project's physics profile (project.ogt `profile`). */
  profile?: "2d" | "3d";
  /** project.ogt's text (CreateOptions.project). */
  project?: string;
  /** The locale og.i18n is asked for (the page's `navigator.language`). */
  locale?: string;
  /** The play surface in CSS pixels, its DPR and safe area: what the UI lays out in. */
  viewport?: Viewport;
  /** Bytes the boot will fetch (the wasm, the cooked bundle), for real progress; omitted: counted as they arrive. */
  expect?: { wasm?: number; cooked?: number };
  /**
   * Without autoplay: take the first tick before the first frame, so the frame behind a play gate shows the
   * game's first screen (its UI is built by ticks), then the ticks that apply the platform's answers to it (settle.ts).
   */
  prime?: boolean;
  /** Record the run's trace from the boot on (src/trace.ts; the shell's `trace()` reads it). */
  trace?: boolean;
  /** Count the played frames for field telemetry (frames.ts; the `telemetry` call takes them). */
  telemetry?: boolean;
  /** Ask for the low-power GPU adapter: the player's power preference from the last load (og.render.setDisplay). */
  lowPower?: boolean;
}

export type Viewport = { width: number; height: number; dpr: number; safeArea?: Record<string, number>; smallWidth?: number; smallHeight?: number; largeWidth?: number; largeHeight?: number; covered?: Record<string, number>; coarsePointer?: boolean; reducedMotion?: boolean; highContrast?: boolean; reducedTransparency?: boolean; macKeys?: boolean };

interface Call {
  type: "call";
  id: number;
  method: string;
  args: unknown[];
}

export type Message =
  | Start
  | Call
  | { type: "input"; event: InjectedInput }
  | { type: "play"; on: boolean }
  | { type: "rpc"; frame: string }
  | { type: "viewport"; viewport: Viewport }
  | { type: "resize"; width: number; height: number; viewport: Viewport }
  | { type: "platform-answer"; request: number; answer: PlatformAnswer }
  | { type: "platform-notice"; notice: PlatformNotice }
  | { type: "platform-host"; host: string; capabilities: string[]; time?: number }
  | { type: "room-events"; events: unknown[] }
  | { type: "room-host"; host: string; join?: string }
  | { type: "fullscreen"; on: boolean }
  | { type: "mouse-report"; active: boolean; unsupported: boolean }
  | { type: "keyboard-layout"; labels: { code: string; label: string }[] }
  | AudioMessage
  | AudioLatencyMessage;

export interface WorkerScope {
  onmessage: ((e: { data: Message }) => void) | null;
  addEventListener?(type: string, fn: (e: { message?: string; error?: unknown; reason?: unknown; preventDefault?(): void }) => void): void;
  postMessage(message: unknown): void;
  requestAnimationFrame?(fn: (t: number) => void): number;
  /** A page's Worker's script URL; absent in bun's Worker, which hosts this module in the tests. */
  readonly location?: { readonly href: string; readonly pathname: string };
}
