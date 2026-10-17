// The engine Worker: owns the OffscreenCanvas and the wasm instance; the page only shows numbers.
import init, { init_log, WebSpike } from "./pkg/og_spike.js";

self.onmessage = async (e) => {
  if (e.data.type !== "start") return;
  const { canvas, count, webgl, models, seconds, isolated, sphere, moving } = e.data;
  try {
    const t0 = performance.now();
    await init();
    init_log();
    const tWasm = performance.now();
    const glbs = await Promise.all(models.map(async (m) => new Uint8Array(await (await fetch(`./models/${m}`)).arrayBuffer())));
    const tFetch = performance.now();
    // A freshly launched browser can answer requestAdapter with nothing for a moment; retry.
    let spike, attempt = 0;
    for (;;) {
      try { spike = await WebSpike.create(canvas, glbs, count, webgl, !!sphere, !!moving); break; }
      catch (err) { if (++attempt >= 8 || !/no adapter/i.test(String(err))) throw err; await new Promise((r) => setTimeout(r, 1000)); }
    }
    const tReady = performance.now();
    postMessage({ type: "hud", text: `wasm ${(tWasm - t0).toFixed(0)} ms, models ${(tFetch - tWasm).toFixed(0)} ms, renderer ${(tReady - tFetch).toFixed(0)} ms` });
    const samples = [];
    let first = 0;
    const start = performance.now();
    let last = start;
    const loop = () => {
      const now = performance.now();
      spike.frame((now - start) / 1000);
      if (!first) first = now;
      samples.push(now - last);
      last = now;
      if (now - start < seconds * 1000) requestAnimationFrame(loop);
      else finish();
    };
    const finish = () => {
      const s = samples.slice(5).sort((a, b) => a - b);
      const q = (p) => s[Math.min(s.length - 1, Math.floor(p * s.length))];
      postMessage({
        type: "result",
        backend: webgl ? "webgl2" : "webgpu",
        scene: sphere ? "sphere" : "gltf", moving: !!moving,
        isolated,
        count,
        triangles: spike.triangles(),
        draws: spike.draws(),
        width: canvas.width,
        height: canvas.height,
        wasmInitMs: +(tWasm - t0).toFixed(1),
        rendererMs: +(tReady - tFetch).toFixed(1),
        firstFrameMs: +(first - t0).toFixed(1),
        frames: samples.length,
        frameMs: { p50: +q(0.5).toFixed(2), p95: +q(0.95).toFixed(2), mean: +(s.reduce((a, b) => a + b, 0) / s.length).toFixed(2) },
      });
    };
    requestAnimationFrame(loop);
  } catch (err) {
    postMessage({ type: "error", message: String(err && err.message || err) });
  }
};
