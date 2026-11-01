// Field telemetry (docs/web-export.md §Telemetry): what the shell tells the
// Player about a session beyond its lifecycle, and the fields of a crash
// report. Everything here is coarse on purpose: a browser's family and major
// version (never the user-agent string), a device class, the GPU's vendor,
// architecture and limits (never its driver description), the engine's
// memory figures, stack frames cut to file names (never a URL, a path or an
// error's message), frame-time counts. Nothing names a person, a page's
// address or what a player typed. A build whose project.ogt says
// `publish { telemetry off }` measures and sends none of it.

import type { FrameStats } from "../src/web/frames.ts";
import { stackFrames } from "../src/sourcemap.ts";

/** A browser: its family, major version and operating system family. */
export interface BrowserInfo {
  readonly name: "chrome" | "edge" | "safari" | "firefox" | "samsung" | "opera" | "other";
  readonly major: number | null;
  readonly os: "windows" | "macos" | "ios" | "android" | "linux" | "chromeos" | "other";
}

/** The device: its class and the coarse figures every browser exposes to any page. */
export interface DeviceInfo {
  readonly class: "phone" | "tablet" | "desktop";
  /** Logical processors (navigator.hardwareConcurrency), null where hidden. */
  readonly cores: number | null;
  /** navigator.deviceMemory, GiB (the browser rounds it to 0.25 to 8), null where hidden. */
  readonly memory: number | null;
  /** The device pixel ratio, to a quarter. */
  readonly dpr: number;
  /** Whether the primary pointer is coarse (touch). */
  readonly touch: boolean;
}

/** The WebGPU adapter: who made it and what it can do (the limits the renderer reads). */
export interface GpuInfo {
  readonly vendor: string;
  readonly architecture: string;
  readonly fallback: boolean;
  readonly limits: Readonly<Record<string, number>>;
  readonly features: readonly string[];
}

/** The engine's memory (the probe's ProbeMemory, flattened), bytes. */
export interface MemoryInfo {
  readonly linear: number | null;
  readonly heapLive: number | null;
  readonly heapPeak: number | null;
  readonly streamUsed: number | null;
  readonly streamBudget: number | null;
  readonly audio: number | null;
  readonly entities: number | null;
  /** The renderer's GPU memory (ProbeMemory.gpu, og-render's GpuMemory): its total and the textures and render targets within it. */
  readonly gpuTotal?: number | null;
  readonly gpuTextures?: number | null;
  readonly gpuTargets?: number | null;
}

/** The limits a report carries: the ones the renderer's paths depend on. */
export const GPU_LIMITS = [
  "maxTextureDimension2D",
  "maxTextureArrayLayers",
  "maxBindGroups",
  "maxSampledTexturesPerShaderStage",
  "maxStorageBuffersPerShaderStage",
  "maxStorageTexturesPerShaderStage",
  "maxUniformBufferBindingSize",
  "maxStorageBufferBindingSize",
  "maxBufferSize",
  "maxVertexBuffers",
  "maxColorAttachments",
  "maxColorAttachmentBytesPerSample",
  "maxComputeWorkgroupStorageSize",
  "maxComputeInvocationsPerWorkgroup",
] as const;
/** The features a report names when the adapter has them. */
export const GPU_FEATURES = [
  "texture-compression-bc",
  "texture-compression-astc",
  "texture-compression-etc2",
  "shader-f16",
  "timestamp-query",
  "float32-filterable",
  "rg11b10ufloat-renderable",
  "indirect-first-instance",
  "depth-clip-control",
  "subgroups",
] as const;

const word = (v: unknown, max = 40) => (typeof v === "string" ? v.toLowerCase().replace(/[^a-z0-9 ._-]/g, "").slice(0, max) : "");

/** The browser of a user-agent string (and User-Agent Client Hints' brands when the browser has them). */
export function browserOf(ua: string, brands?: ReadonlyArray<{ brand: string; version: string }>): BrowserInfo {
  const os: BrowserInfo["os"] = /iPhone|iPad|iPod/.test(ua)
    ? "ios"
    : /Android/.test(ua)
      ? "android"
      : /CrOS/.test(ua)
        ? "chromeos"
        : /Windows/.test(ua)
          ? "windows"
          : /Macintosh|Mac OS X/.test(ua)
            ? "macos"
            : /Linux/.test(ua)
              ? "linux"
              : "other";
  const major = (re: RegExp) => {
    const m = re.exec(ua);
    return m ? Number(m[1]) : null;
  };
  const brand = (name: string) => {
    const b = brands?.find((x) => x.brand === name);
    return b ? Number.parseInt(b.version, 10) || null : undefined;
  };
  const edge = brand("Microsoft Edge");
  if (edge !== undefined || /Edg(e|A|iOS)?\//.test(ua)) return { name: "edge", major: edge ?? major(/Edg(?:e|A|iOS)?\/(\d+)/), os };
  if (/SamsungBrowser\//.test(ua)) return { name: "samsung", major: major(/SamsungBrowser\/(\d+)/), os };
  if (/OPR\/|Opera/.test(ua)) return { name: "opera", major: major(/OPR\/(\d+)/), os };
  if (/Firefox\/|FxiOS\//.test(ua)) return { name: "firefox", major: major(/(?:Firefox|FxiOS)\/(\d+)/), os };
  const chrome = brand("Google Chrome");
  if (chrome !== undefined || /Chrome\/|CriOS\//.test(ua)) return { name: "chrome", major: chrome ?? major(/(?:Chrome|CriOS)\/(\d+)/), os };
  if (/Safari\//.test(ua)) return { name: "safari", major: major(/Version\/(\d+)/), os };
  return { name: "other", major: null, os };
}

/** The device's class and figures: a phone or tablet by its touch screen's short side (and the OS), else a desktop. */
export function deviceOf(env: { ua: string; coarse: boolean; touchPoints: number; screen: { width: number; height: number }; dpr: number; cores?: number; memory?: number }): DeviceInfo {
  const short = Math.min(env.screen.width, env.screen.height);
  const mobileOs = /iPhone|iPod|Android.*Mobile/.test(env.ua);
  const tabletOs = /iPad|Android(?!.*Mobile)/.test(env.ua) || (/Macintosh/.test(env.ua) && env.touchPoints > 1);
  const cls: DeviceInfo["class"] = mobileOs || (env.coarse && short < 600) ? "phone" : tabletOs || (env.coarse && env.touchPoints > 0) ? "tablet" : "desktop";
  return {
    class: cls,
    cores: typeof env.cores === "number" && env.cores > 0 ? Math.min(env.cores, 64) : null,
    memory: typeof env.memory === "number" && env.memory > 0 ? env.memory : null,
    dpr: Math.round((env.dpr || 1) * 4) / 4,
    touch: env.coarse,
  };
}

/**
 * The device the engine renders with (the Worker's `ready.gpu`, worker.ts
 * deviceReport: never an adapter the page asks for itself, which could be
 * another GPU of the machine): its adapter's vendor and architecture, the
 * limits GPU_LIMITS names, the features GPU_FEATURES names.
 */
export function gpuOf(adapter: { info?: unknown; limits?: unknown; features?: readonly string[] | { has(f: string): boolean }; isFallbackAdapter?: boolean }): GpuInfo {
  const info = (adapter.info ?? {}) as { vendor?: unknown; architecture?: unknown; isFallbackAdapter?: unknown };
  const limits: Record<string, number> = {};
  const given = (adapter.limits ?? {}) as Record<string, unknown>;
  for (const k of GPU_LIMITS) if (typeof given[k] === "number") limits[k] = given[k] as number;
  return {
    vendor: word(info.vendor),
    architecture: word(info.architecture),
    fallback: info.isFallbackAdapter === true || adapter.isFallbackAdapter === true,
    limits,
    features: GPU_FEATURES.filter((f) => {
      const given = adapter.features;
      return Array.isArray(given) ? given.includes(f) : (given as { has(f: string): boolean } | undefined)?.has(f) === true;
    }),
  };
}

/** The probe's `memory` (ProbeMemory) as a report's figures; null when the engine did not answer. */
export function memoryOf(m: unknown): MemoryInfo | null {
  const p = m as { linear?: unknown; heap?: { live?: unknown; peak?: unknown } | null; stream?: { used?: unknown; budget?: unknown } | null; audio?: unknown; world?: { entities?: unknown }; gpu?: { total?: unknown; textures?: unknown; renderTargets?: unknown } | null } | null;
  if (!p || typeof p !== "object") return null;
  const n = (v: unknown) => (typeof v === "number" && Number.isFinite(v) ? v : null);
  return {
    linear: n(p.linear),
    heapLive: n(p.heap?.live),
    heapPeak: n(p.heap?.peak),
    streamUsed: n(p.stream?.used),
    streamBudget: n(p.stream?.budget),
    audio: n(p.audio),
    entities: n(p.world?.entities),
    gpuTotal: n(p.gpu?.total),
    gpuTextures: n(p.gpu?.textures),
    gpuTargets: n(p.gpu?.renderTargets),
  };
}

/** Where the session runs: sent once, with the boot sample, and with every crash. */
export interface Context {
  /** The engine's version (og-build.json `engine.version`). */
  readonly engine: string;
  /** The build's id (og-build.json `sourcemaps.build`), which names the maps a stack resolves with. */
  readonly build: string;
  readonly browser: BrowserInfo;
  readonly device: DeviceInfo;
  /** Null until the adapter answered, or when the page has no WebGPU. */
  readonly gpu: GpuInfo | null;
}

/** What stopped the engine, as shell.ts's HostError carries it. */
export interface CrashCause {
  readonly kind: "panic" | "error" | "hang";
  readonly message: string;
  readonly tick: number | null;
  readonly lane: string | null;
  readonly location: string | null;
}

/**
 * A crash report's fields beyond its name and fingerprint: the stack's frames
 * (the bundle's lines, mapped with the build's maps: `mapStack`), the tick,
 * what kind of stop it was (a panic's lanes site and Rust location), the
 * context and the engine's memory as last read.
 */
export function crashFields(cause: CrashCause, context: Context, memory: MemoryInfo | null): Record<string, unknown> {
  return {
    cause: cause.kind,
    tick: cause.tick,
    ...(cause.lane ? { lane: word(cause.lane) } : {}),
    // A Rust panic's `file:line:column` in the engine's crates (no machine's path: cargo's are relative).
    ...(cause.location ? { location: cause.location.replace(/^.*?(crates\/)/, "$1").slice(0, 160) } : {}),
    stack: stackFrames(cause.message),
    ...context,
    memory,
  };
}

/** A `sample` of the frames played: their counts, the memory and the files that did not arrive since the last sample. */
export function playFields(phase: "play" | "end", frames: FrameStats | null, memory: MemoryInfo | null, failures: ReadonlyArray<{ file: string; reason: string }>, session: { ms: number; samples: number }): Record<string, unknown> {
  return {
    phase,
    ...(frames ? { frames } : {}),
    ...(memory ? { memory } : {}),
    ...(failures.length ? { failures: failures.slice(0, 20) } : {}),
    session,
  };
}
