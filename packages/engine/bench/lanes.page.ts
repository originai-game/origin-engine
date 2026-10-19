// The page of bench/lanes.ts in Chrome: the reference scene's publish build
// (served under /game/) on a 1280×720 canvas, started with no clock so the
// bench drives it one `frame` at a time through `globalThis.__og_shell__`.
// `?lanes=0` starts it without the lanes variant (single-threaded on an
// isolated page); otherwise the shell decides from crossOriginIsolated, as
// the product page does.
import { Shell } from "../src/web/shell.ts";

const params = new URLSearchParams(location.search);
const lines: string[] = [];
(globalThis as Record<string, unknown>)["__og_log__"] = lines;
const at = (path: string) => new URL(`/game/${path}`, location.href).href;
try {
  const shell = await Shell.start({
    canvas: document.querySelector("canvas")!,
    worker: new Worker(at("game.js"), { type: "module" }),
    glue: at("origin-engine.js"),
    wasm: at("origin-engine_bg.wasm"),
    ...(params.get("lanes") === "0" ? {} : { lanes: { glue: at("origin-engine-lanes.js"), wasm: at("origin-engine-lanes_bg.wasm") } }),
    root: at("").replace(/\/$/, ""),
    basis: { js: at("basis/basis_transcoder.js"), wasm: at("basis/basis_transcoder.wasm") },
    autoplay: false,
    prime: false,
    profile: "3d",
    dprCap: 1,
    onLog: (l) => lines.push(...l),
    onCrash: (m) => lines.push(`crash: ${m}`),
  });
  (globalThis as Record<string, unknown>)["__og_shell__"] = shell;
} catch (e) {
  (globalThis as Record<string, unknown>)["__og_shell_error__"] = e instanceof Error ? `${e.message}\n${e.stack}` : String(e);
}
