// The page side of the platform's `og-probe` wire
// (origingame/origin/packages/contracts/src/probe-wire.ts, wire 1, probe
// protocol 5) over the engine's in-page probe. The engine runs in a Worker,
// so every member is answered asynchronously through the shell's request
// ring (or postMessage without isolation); the wire's shapes are the
// contract's whatever the engine answers.
//
// Rules kept exactly as the contract states them:
// - a call is answered only when `event.source` is the host window
//   (`window.parent` when embedded, else `window.opener`) and `event.origin`
//   is the origin the page was told: `window.__OG_STAGE_HOST__`; a page that
//   was told no origin sends nothing;
// - every post names that origin; `"*"` is never used;
// - anything else (another `og`, another `v`, not a call, a foreign source or
//   origin) gets no result and no error.
//
// A publish build serves the 7 player members (hello, tick, state, pick,
// entity, input, capture): `pause`, `resume`, `advance`, `bounds` and
// `patch` answer `unsupported` there; a dev (Stage) build serves them all.

export const PROBE_WIRE = "probe";
export const PROBE_WIRE_VERSION = 1;
export const PROBE_WIRE_PROTOCOL = 5;
export const STAGE_HOST_GLOBAL = "__OG_STAGE_HOST__";
export const PROBE_WIRE_RECORD_MAX_SECONDS = 15;
export const PROBE_WIRE_SLOW_MS = 100;
export const PROBE_WIRE_METHODS = ["hello", "tick", "state", "pause", "resume", "advance", "entities", "bounds", "entity", "patch", "capture", "record"] as const;
/** The probe's `advance` takes at most this many steps. */
const MAX_STEPS = 3600;
/** JSON-RPC: a member the build does not serve. */
const METHOD_NOT_FOUND = -32601;

export type WireErrorCode = "unsupported" | "refused" | "failed";

export class WireError extends Error {
  readonly code: WireErrorCode;
  constructor(code: WireErrorCode, message: string) {
    super(message);
    this.code = code;
  }
}

/** One engine probe member by name (JSON-RPC params), resolving its result or rejecting with `{code}`. */
export type ProbeCall = (method: string, params?: Record<string, unknown>) => Promise<unknown>;

export interface WireOptions {
  /** The window allowed to call. */
  readonly host: () => Window | null | undefined;
  /**
   * A second caller: the page's own window, when a host in this same document (the
   * Stage overlay injected into a top-level engine page) posts to it. Answered like the host.
   */
  readonly inPage?: () => Window | null | undefined;
  /** The origin allowed to call and the only target of every post; null keeps the page silent. */
  readonly hostOrigin: () => string | null;
  /** The engine's probe, or null before the engine is ready. */
  readonly probe: () => ProbeCall | null;
  /** The render target in device pixels (what a full-size capture is). */
  readonly targetSize: () => { width: number; height: number };
  /** The canvas a recording is made from. */
  readonly canvas: () => HTMLCanvasElement | null;
  readonly frame?: (callback: () => void) => void;
  readonly now?: () => number;
}

export interface WireState {
  tick: number;
  paused: boolean;
  timings: { frameMs: number | null };
}

const finite = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v);
const plain = (v: unknown): Record<string, unknown> | null => (v && typeof v === "object" && !Array.isArray(v) ? (v as Record<string, unknown>) : null);

/** `ProbeWireState` from the engine's state head: `paused` and `timings.frameMs` are its own. */
export function wireState(head: unknown): WireState {
  const h = plain(head) ?? {};
  const t = plain(h["timings"]) ?? {};
  const frameMs = finite(t["frameMs"]) && t["frameMs"] >= 0 ? t["frameMs"] : finite(t["step"]) && finite(t["render"]) ? t["step"] + t["render"] : null;
  const tick = finite(h["tick"]) && Number.isSafeInteger(h["tick"]) && h["tick"] >= 0 ? h["tick"] : 0;
  return { tick, paused: h["paused"] === true, timings: { frameMs } };
}

/** `{items}` of `bounds {id}`: the probe's order, ranges only where declared, the current value where it is a number. */
export function wireBounds(declared: unknown, values: unknown): { items: Array<Record<string, unknown>> } {
  const d = plain(declared) ?? {};
  const v = plain(values) ?? {};
  return {
    items: Object.entries(d).map(([parameter, row]) => {
      const r = plain(row) ?? {};
      const { label, path, min, max, step } = r;
      const value = v[parameter];
      return {
        parameter,
        ...(typeof label === "string" && label ? { label } : {}),
        ...(typeof path === "string" && path ? { path } : {}),
        ...(finite(min) && finite(max) && max >= min ? { min, max } : {}),
        ...(finite(step) && step > 0 ? { step } : {}),
        ...(finite(value) ? { value } : {}),
      };
    }),
  };
}

/** The size a capture fits into `width`×`height` (either may be absent), never larger than the target. */
export function fitCapture(target: { width: number; height: number }, width?: number, height?: number): { width: number; height: number } {
  const w = width && width > 0 ? width : 0;
  const h = height && height > 0 ? height : 0;
  if (!w && !h) return { width: 0, height: 0 };
  const scale = Math.min(w ? w / target.width : Infinity, h ? h / target.height : Infinity, 1);
  return { width: Math.max(1, Math.round(target.width * scale)), height: Math.max(1, Math.round(target.height * scale)) };
}

function blobBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(new WireError("failed", "could not read the recording"));
    reader.onload = () => {
      const text = String(reader.result);
      resolve(text.slice(text.indexOf(",") + 1));
    };
    reader.readAsDataURL(blob);
  });
}

export function createWire(options: WireOptions) {
  const now = options.now ?? (() => performance.now());
  const frame = options.frame ?? ((cb: () => void) => requestAnimationFrame(cb));
  let recording = false;

  const need = (): ProbeCall => {
    const p = options.probe();
    if (!p) throw new WireError("unsupported", "the engine is not ready: no probe yet");
    return p;
  };
  /** One engine member; a member this build does not serve is `unsupported`, any other error `failed`. */
  const member = async (method: string, params: Record<string, unknown> = {}): Promise<unknown> => {
    try {
      return await need()(method, params);
    } catch (e) {
      if (e instanceof WireError) throw e;
      const code = (e as { code?: unknown }).code;
      if (code === METHOD_NOT_FOUND) throw new WireError("unsupported", `${method} is not served by this build (a publish build serves the player members only)`);
      throw new WireError("failed", e instanceof Error ? e.message : String(e));
    }
  };
  const text = (value: unknown, name: string): string => {
    if (typeof value !== "string" || !value) throw new WireError("refused", `${name} must be a non-empty string`);
    return value;
  };
  const state = async () => wireState(await member("state"));

  function hello(): { protocol: 5 | null; substrate: "origin-engine" | null } {
    return options.probe() ? { protocol: PROBE_WIRE_PROTOCOL, substrate: "origin-engine" } : { protocol: null, substrate: null };
  }

  async function answer(method: string, params: Record<string, unknown>, received: number): Promise<unknown> {
    switch (method) {
      case "hello":
        return hello();
      case "tick": {
        const tick = await member("tick");
        if (!finite(tick)) throw new WireError("failed", "the probe answered no tick");
        return tick;
      }
      case "state":
        return state();
      case "pause":
      case "resume": {
        if ((await member(method)) !== true) throw new WireError("refused", `the probe refused ${method}`);
        return state();
      }
      case "advance": {
        const steps = params["steps"];
        if (typeof steps !== "number" || !Number.isInteger(steps) || steps < 0 || steps > MAX_STEPS) throw new WireError("refused", `steps must be a whole number from 0 to ${MAX_STEPS}`);
        await member("advance", { steps });
        return state();
      }
      case "entities": {
        const head = plain(await member("state"));
        const list = Array.isArray(head?.["entities"]) ? (head!["entities"] as unknown[]) : [];
        return {
          items: list.flatMap((e) => {
            const row = plain(e);
            const id = row?.["id"];
            if (typeof id !== "string" || !id) return [];
            const label = row!["label"];
            return [{ id, ...(typeof label === "string" && label ? { label } : {}) }];
          }),
        };
      }
      case "bounds": {
        const id = text(params["id"], "id");
        const declared = await member("bounds", { id });
        if (!declared) throw new WireError("refused", `no entity ${id}`);
        return wireBounds(declared, await member("entity", { id }));
      }
      case "entity": {
        const id = text(params["id"], "id");
        const values = await member("entity", { id });
        if (!plain(values)) throw new WireError("refused", `no entity ${id}`);
        return { id, params: values };
      }
      case "patch": {
        const id = text(params["id"], "id");
        const parameter = text(params["parameter"], "parameter");
        if (!("value" in params)) throw new WireError("refused", "patch needs a value");
        if ((await member("patch", { id, parameter, value: params["value"] })) !== true) throw new WireError("refused", `${parameter} of ${id} refused`);
        // Applied before the next tick: the time to the first animation frame after the answer.
        return new Promise((resolve) => {
          let done = false;
          const settle = () => {
            if (done) return;
            done = true;
            resolve({ applied: true, roundTripMs: Math.max(0, Math.round((now() - received) * 10) / 10) });
          };
          frame(settle);
          setTimeout(settle, PROBE_WIRE_SLOW_MS);
        });
      }
      case "capture": {
        const width = typeof params["width"] === "number" ? params["width"] : undefined;
        const height = typeof params["height"] === "number" ? params["height"] : undefined;
        if ((width !== undefined && !(width >= 0 && width <= 4096)) || (height !== undefined && !(height >= 0 && height <= 4096))) throw new WireError("refused", "width and height must be within 0–4096");
        const size = fitCapture(options.targetSize(), width, height);
        const shot = plain(await member("capture", size));
        if (!shot || typeof shot["png"] !== "string" || !shot["png"]) throw new WireError("unsupported", "this page has no renderer to capture");
        return { tick: finite(shot["tick"]) ? shot["tick"] : 0, width: Number(shot["width"]) || 0, height: Number(shot["height"]) || 0, png: shot["png"] };
      }
      case "record": {
        const seconds = params["seconds"];
        if (typeof seconds !== "number" || !(seconds > 0) || seconds > PROBE_WIRE_RECORD_MAX_SECONDS) throw new WireError("refused", `seconds must be within (0, ${PROBE_WIRE_RECORD_MAX_SECONDS}]`);
        if (recording) throw new WireError("refused", "a recording is in progress");
        const canvas = options.canvas() as (HTMLCanvasElement & { captureStream?: (fps: number) => MediaStream }) | null;
        if (!canvas || !canvas.captureStream || typeof MediaRecorder === "undefined") throw new WireError("unsupported", "recording is not supported here");
        recording = true;
        try {
          let recorder: MediaRecorder;
          try {
            recorder = new MediaRecorder(canvas.captureStream(30), MediaRecorder.isTypeSupported("video/webm") ? { mimeType: "video/webm" } : {});
          } catch (e) {
            throw new WireError("unsupported", `recording is not supported here: ${e instanceof Error ? e.message : String(e)}`);
          }
          const made = await new Promise<{ blob: Blob; mime: string; durationMs: number }>((resolve, reject) => {
            const chunks: Blob[] = [];
            const started = now();
            recorder.ondataavailable = (e) => {
              if (e.data.size) chunks.push(e.data);
            };
            recorder.onerror = () => reject(new WireError("failed", "the recorder failed"));
            recorder.onstop = () => {
              recorder.stream.getTracks().forEach((t) => t.stop());
              const mime = recorder.mimeType || "video/webm";
              resolve({ blob: new Blob(chunks, { type: mime }), mime, durationMs: Math.round(now() - started) });
            };
            recorder.start(1000);
            setTimeout(() => recorder.state !== "inactive" && recorder.stop(), seconds * 1000);
          });
          return { mime: made.mime, base64: await blobBase64(made.blob), durationMs: made.durationMs };
        } finally {
          recording = false;
        }
      }
      default:
        throw new WireError("unsupported", `unknown method ${method}`);
    }
  }

  /** The windows that may call: the host, and the page itself when an in-page host is allowed. */
  const callers = (): Window[] => {
    const out: Window[] = [];
    const host = options.host();
    if (host) out.push(host);
    const self = options.inPage?.();
    if (self && self !== host) out.push(self);
    return out;
  };

  function post(message: Record<string, unknown>, to?: Window): void {
    const origin = options.hostOrigin();
    if (!origin || origin === "*") return;
    for (const target of to ? [to] : callers()) {
      try {
        target.postMessage({ og: PROBE_WIRE, v: PROBE_WIRE_VERSION, ...message }, origin);
      } catch {
        /* the host went away */
      }
    }
  }

  function onMessage(event: MessageEvent): void {
    const origin = options.hostOrigin();
    const from = callers().find((w) => w === event.source);
    if (!from || !origin || event.origin !== origin) return;
    const data = plain(event.data);
    if (!data || data["og"] !== PROBE_WIRE || data["v"] !== PROBE_WIRE_VERSION || data["kind"] !== "call") return;
    const id = data["id"];
    const params = plain(data["params"]);
    if (typeof id !== "string" || typeof data["method"] !== "string" || !params) return;
    const received = now();
    answer(data["method"], params, received).then(
      (result) => {
        post({ kind: "result", id, result: result === undefined ? null : result }, from);
        if (data["method"] === "hello") post({ kind: "hello", ...hello() }, from);
      },
      (error) => post({ kind: "error", id, code: error instanceof WireError ? error.code : "failed", message: String((error as Error)?.message ?? error) }, from),
    );
  }

  /** Hello on load and whenever the told origin or the probe's availability changes; returns the stop function. */
  function watch(intervalMs = 500): () => void {
    let last: string | null = null;
    const check = () => {
      const origin = options.hostOrigin();
      if (!callers().length || !origin) {
        last = null;
        return;
      }
      const h = hello();
      const key = `${origin}|${h.protocol}|${h.substrate}`;
      if (key === last) return;
      last = key;
      post({ kind: "hello", ...h });
    };
    check();
    const timer = setInterval(check, intervalMs);
    return () => clearInterval(timer);
  }

  return { onMessage, watch, hello, answer };
}

/**
 * Whether the platform's Stage overlay is drawn in this very document: the page
 * is not framed and was told its own origin as the Stage host. The og-probe wire
 * then also answers this window, and the Player bridge (./player.ts) speaks
 * Player protocol 2 to it.
 */
export function stageInPage(w: Window & Record<string, unknown> = globalThis as unknown as Window & Record<string, unknown>): boolean {
  try {
    return w.parent === w && w[STAGE_HOST_GLOBAL] === w.location?.origin;
  } catch {
    return false;
  }
}

/**
 * The wire on this page: host = parent when embedded, else opener; origin =
 * `window.__OG_STAGE_HOST__`. A top-level page told its own origin also answers
 * itself: the Stage overlay drawn in this document posts to this window.
 */
export function mountWire(options: Omit<WireOptions, "host" | "hostOrigin">): () => void {
  const w = globalThis as unknown as Window & Record<string, unknown>;
  const wire = createWire({
    ...options,
    host: () => (w.parent && w.parent !== w ? w.parent : (w.opener as Window | null)),
    inPage: () => (stageInPage(w) ? w : null),
    hostOrigin: () => {
      const told = w[STAGE_HOST_GLOBAL];
      return typeof told === "string" && told && told !== "*" ? told : null;
    },
  });
  addEventListener("message", wire.onMessage);
  const stop = wire.watch();
  return () => {
    stop();
    removeEventListener("message", wire.onMessage);
  };
}
