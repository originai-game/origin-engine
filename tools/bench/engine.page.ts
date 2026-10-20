// The engine's page in tools/bench/threejs-compare.ts: a publish build
// (served under /game/) on a 1280×720 canvas at DPR 1, started with no
// clock, so the harness drives it one `frame` (one 1/60 s tick and one
// render) at a time through `globalThis.__og_shell__`, as
// packages/engine/bench/lanes.ts does. `?webgl=1` takes the WebGL2 fallback.
// The shell decides lanes from crossOriginIsolated, as the product page does.
import { Shell } from "../../packages/engine/src/web/shell.ts";

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
    lanes: { glue: at("origin-engine-lanes.js"), wasm: at("origin-engine-lanes_bg.wasm") },
    root: at("").replace(/\/$/, ""),
    basis: { js: at("basis/basis_transcoder.js"), wasm: at("basis/basis_transcoder.wasm") },
    webgl: params.get("webgl") === "1",
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
