// The play page `og build` ships (dist/index.html loads dist/play.js): the
// main-thread shell on the page's canvas, the engine Worker (dist/game.js),
// the cooked bundle next to the page, the Basis transcoder under basis/.
//
// - A `replay-seed` query parameter (REPLAY_SEED_PARAM) starts a deterministic
//   replay from that uint32 seed.
// - The probe is published on `globalThis.__runtime_probe__` (PROBE_GLOBAL).
//   In a page every member answers a Promise: the call crosses the request
//   ring to the Worker (postMessage without cross-origin isolation), and a
//   `capture` resolves once its readback has landed. Protocol-5 refusals are
//   `false` or `null`, as on the native socket.
// - Until the engine is ready the page shows the boot state: ink #14161b with
//   a real progress value in white.
//   TODO(brand): replace the bar with packages/brand's `genesis` (boot) and
//   `progress` (loading) motions from the platform repository, driven by the
//   same value; the engine introduces no second palette or mark.

import { Shell } from "./shell.ts";

const PROBE_GLOBAL = "__runtime_probe__";
const REPLAY_SEED_PARAM = "replay-seed";

const MEMBERS = [
  "hello", "tick", "state", "advance", "pause", "resume", "input", "accepted", "pick", "entity",
  "bounds", "patch", "run", "capture", "sweep", "reload", "snapshot", "restore", "dump", "commands",
] as const;

/** Positional protocol-5 arguments to JSON-RPC params, as og-probe reads them. */
function params(member: string, args: unknown[]): Record<string, unknown> {
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

function bootState(): { set(value: number): void; done(): void; fail(message: string): void } {
  const veil = document.createElement("div");
  veil.setAttribute("role", "progressbar");
  veil.setAttribute("aria-valuemin", "0");
  veil.setAttribute("aria-valuemax", "100");
  veil.style.cssText = "position:fixed;inset:0;display:flex;align-items:center;justify-content:center;background:#14161b;color:#fff;font:14px system-ui,sans-serif";
  const track = document.createElement("div");
  track.style.cssText = "width:min(240px,60vw);height:2px;background:rgba(255,255,255,0.2)";
  const bar = document.createElement("div");
  bar.style.cssText = "height:100%;width:0;background:#fff;transition:width 120ms linear";
  track.append(bar);
  veil.append(track);
  document.body.append(veil);
  return {
    set(value) {
      const pct = Math.round(Math.max(0, Math.min(1, value)) * 100);
      bar.style.width = `${pct}%`;
      veil.setAttribute("aria-valuenow", String(pct));
    },
    done: () => veil.remove(),
    fail(message) {
      veil.textContent = message;
      veil.style.padding = "16px";
    },
  };
}

export async function play(): Promise<Shell> {
  const canvas = document.querySelector("canvas") ?? document.body.appendChild(document.createElement("canvas"));
  const boot = bootState();
  const query = new URLSearchParams(location.search);
  const seedText = query.get(REPLAY_SEED_PARAM);
  const seed = seedText !== null && /^\d+$/.test(seedText) && Number(seedText) <= 0xffffffff ? Number(seedText) : undefined;
  try {
    const shell = await Shell.start({
      canvas,
      worker: new Worker(new URL("./game.js", location.href), { type: "module" }),
      glue: new URL("./origin-engine.js", location.href).href,
      wasm: new URL("./origin-engine_bg.wasm", location.href).href,
      root: new URL(".", location.href).href.replace(/\/$/, ""),
      basis: { js: new URL("./basis/basis_transcoder.js", location.href).href, wasm: new URL("./basis/basis_transcoder.wasm", location.href).href },
      webgl: query.get("webgl") === "1" || !("gpu" in navigator),
      autoplay: query.get("autoplay") !== "0",
      ...(seed === undefined ? {} : { seed }),
      profile: document.querySelector<HTMLMetaElement>('meta[name="og-profile"]')?.content === "2d" ? "2d" : "3d",
      onProgress: (v) => boot.set(v),
    });
    const probe: Record<string, unknown> = { version: 5 };
    for (const m of MEMBERS) probe[m] = (...args: unknown[]) => shell.probe(m, params(m, args));
    (globalThis as Record<string, unknown>)[PROBE_GLOBAL] = Object.freeze(probe);
    boot.done();
    return shell;
  } catch (e) {
    boot.fail(`The game could not start: ${e instanceof Error ? e.message : String(e)}`);
    throw e;
  }
}

void play();
