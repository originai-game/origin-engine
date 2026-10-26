// The page of tests/web/gpu-timing.ts: the shell on the tier the query
// asks for (`?webgl=1`, the WebGL2 fallback), the renderer's capabilities
// left in the engine Worker (the `caps` command), then frames, each
// followed by a pause that returns to the event loop, as a played frame
// does: WebGL makes a query's result available only after the task that
// ended it. Posts the probe's timings.
import { Shell } from "../../src/web/shell.ts";

const params = new URLSearchParams(location.search);
const post = (body: unknown) => fetch("/result", { method: "POST", body: JSON.stringify(body) });

try {
  // Whether this browser's WebGL2 has the timer extension of its own: the
  // page's context, which the harness never gives a stand-in.
  const gl = document.createElement("canvas").getContext("webgl2");
  const native = !!gl?.getSupportedExtensions()?.includes("EXT_disjoint_timer_query_webgl2");
  const shell = await Shell.start({
    canvas: document.querySelector("canvas")!,
    worker: new Worker("./gpu-timing.worker.js", { type: "module" }),
    glue: "/web/origin-engine.js",
    wasm: "/web/origin-engine_bg.wasm",
    webgl: params.get("webgl") === "1",
    autoplay: false,
  });
  await shell.call("command", "caps", []);
  const frames = Number(params.get("frames") ?? "40");
  for (let i = 0; i < frames; i++) {
    await shell.call("frame", 1000 / 60);
    await new Promise((r) => setTimeout(r, 16));
  }
  const state = (await shell.call("probe", "state")) as { timings: Record<string, unknown> };
  await post({ tier: shell.tier, native, timings: state.timings });
} catch (e) {
  await post({ error: e instanceof Error ? `${e.message}\n${e.stack}` : String(e) });
}
