// The page of the browser run: the shell starts the engine Worker, spawns the
// hello world's 10,000 entities through its own command, advances 600 ticks,
// renders a few frames and posts what it saw to the harness.
import { Shell } from "../../src/web/shell.ts";

const params = new URLSearchParams(location.search);
const post = (body: unknown) => fetch("/result", { method: "POST", body: JSON.stringify(body) });

try {
  const canvas = document.querySelector("canvas")!;
  const worker = new Worker("./hello.worker.js", { type: "module" });
  const shell = await Shell.start({
    canvas,
    worker,
    glue: "/web/origin-engine.js",
    wasm: "/web/origin-engine_bg.wasm",
    webgl: params.get("webgl") === "1",
    autoplay: false,
  });
  const t0 = performance.now();
  const spawned = (await shell.call("command", "spawn_wave", ["10000"])) as { ok: boolean; reason?: string };
  const t1 = performance.now();
  const advanced = (await shell.call("advance", 600)) as { ok: boolean; value?: number; reason?: string };
  const t2 = performance.now();
  const hash = await shell.call("hash");
  const dump = (await shell.call("probe", "dump", { what: "world" })) as { entities: number; systems: Array<{ ms: number }> };
  // Input through the page's own listeners: the ring when isolated, messages otherwise.
  const before = (await shell.call("probe", "accepted")) as number;
  for (const type of ["keydown", "keyup"]) dispatchEvent(new KeyboardEvent(type, { code: "KeyW" }));
  await new Promise((r) => setTimeout(r, 50));
  const after = (await shell.call("probe", "accepted")) as number;
  // tools/bench/views.ts's raw loop on this page, then the same work as the
  // hello world's Move system through the engine, 2000 ticks.
  const N = 10000;
  const mem = new WebAssembly.Memory({ initial: 256 });
  const pos = new Float32Array(mem.buffer, 0, N * 3);
  const vel = new Float32Array(mem.buffer, N * 12 + 16, N * 3);
  for (let i = 0; i < N; i++) vel[i * 3 + 2] = 1;
  const integrate = (p: Float32Array, v: Float32Array, n: number, dt: number) => {
    for (let i = 0; i < n * 3; i += 3) { p[i]! += v[i]! * dt; p[i + 1]! += v[i + 1]! * dt; p[i + 2]! += v[i + 2]! * dt; }
  };
  for (let i = 0; i < 100; i++) integrate(pos, vel, N, 1 / 60);
  let tb = performance.now();
  for (let i = 0; i < 2000; i++) integrate(pos, vel, N, 1 / 60);
  const rawLoopMs = (performance.now() - tb) / 2000;
  await shell.call("advance", 100);
  tb = performance.now();
  await shell.call("advance", 2000);
  const engineTickMs = (performance.now() - tb) / 2000;
  const frames: unknown[] = [];
  for (let i = 0; i < 3; i++) frames.push(await shell.call("frame", 0));
  // og render's every-frame capture on the web (the probe's Capture::frame):
  // the first ask turns it on, the next frame is kept, its readback lands.
  await shell.call("renderFrame");
  await shell.call("frame", 0);
  let kept: Uint8Array | null = null;
  for (let i = 0; i < 100 && !kept; i++) {
    kept = (await shell.call("renderFrame")) as Uint8Array | null;
    if (!kept) await new Promise((r) => setTimeout(r, 10));
  }
  let lit = 0;
  for (let i = 0; kept && i < kept.length; i += 4) if (kept[i]! + kept[i + 1]! + kept[i + 2]! > 24) lit++;
  const renderFrame = { bytes: kept?.length ?? 0, expected: canvas.width * canvas.height * 4, lit };
  await post({
    host: "web",
    userAgent: navigator.userAgent,
    crossOriginIsolated,
    transport: shell.transport,
    tier: shell.tier,
    spawned,
    advanced,
    tick: advanced.value,
    hash,
    entities: dump.entities,
    moveSystemMs: dump.systems[0]?.ms,
    spawnMs: +(t1 - t0).toFixed(1),
    advanceMs: +(t2 - t1).toFixed(1),
    inputAccepted: after - before,
    views: { rawLoopMs: +rawLoopMs.toFixed(4), engineTickMs: +engineTickMs.toFixed(4), ratio: +(engineTickMs / rawLoopMs).toFixed(2) },
    frames,
    renderFrame,
    ...(renderFrame.bytes !== renderFrame.expected || renderFrame.lit === 0
      ? { error: `renderFrame: ${renderFrame.bytes} bytes for ${renderFrame.expected}, ${renderFrame.lit} lit pixels` }
      : {}),
  });
} catch (e) {
  await post({ host: "web", error: e instanceof Error ? `${e.message}\n${e.stack}` : String(e) });
}
