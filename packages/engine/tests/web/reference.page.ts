// The page of tests/web/reference.ts: the exported game's engine Worker
// (game.js, origin-engine.js and .wasm, cooked/, basis/ of an `og build`
// directory served under /game/) on a 1280×720 canvas, with no gate and no
// clock: the harness drives it as `og run --commands` does natively, one
// `frame` of one tick at a time, through `globalThis.__og_shell__`.
import { Shell } from "../../src/web/shell.ts";

const params = new URLSearchParams(location.search);
const lines: string[] = [];
(globalThis as Record<string, unknown>)["__og_log__"] = lines;
const at = (path: string) => new URL(`/game/${path}`, location.href).href;
try {
  const canvas = document.querySelector("canvas")!;
  const shell = await Shell.start({
    canvas,
    worker: new Worker(at("game.js"), { type: "module" }),
    glue: at("origin-engine.js"),
    wasm: at("origin-engine_bg.wasm"),
    root: at("").replace(/\/$/, ""),
    basis: { js: at("basis/basis_transcoder.js"), wasm: at("basis/basis_transcoder.wasm") },
    webgl: params.get("webgl") === "1",
    autoplay: false,
    prime: false,
    profile: params.get("profile") === "2d" ? "2d" : "3d",
    dprCap: 1,
    onLog: (l) => lines.push(...l),
    onCrash: (m) => lines.push(`crash: ${m}`),
  });
  (globalThis as Record<string, unknown>)["__og_shell__"] = shell;
} catch (e) {
  (globalThis as Record<string, unknown>)["__og_shell_error__"] = e instanceof Error ? `${e.message}\n${e.stack}` : String(e);
}
