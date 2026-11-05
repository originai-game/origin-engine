import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

function once(s: string, from: string, to: string): string {
  if (s.split(from).length !== 2) throw new Error(`expected one instrumentation site: ${from}`);
  return s.replace(from, to);
}

export function instrument(out: string): void {
  const game = join(out, "game.js");
  let s = readFileSync(game, "utf8");
  s = once(s, 'if (data.type === "play") {', `if (data.type === "play") {
    if (data.on) { globalThis.__diagBase = engine.probe.tick(); globalThis.__diagMoved = false; globalThis.__diagLog("worker-play", {tick: globalThis.__diagBase}); }`);
  s = once(s, "const answered = engine.frame(last ? now - last : 0);", `const answered = engine.frame(last ? now - last : 0);
    if (globalThis.__diagBase !== undefined && !globalThis.__diagMoved) {
      globalThis.__diagLog("frame-completed", {tick: answered.tick, began: performance.timeOrigin + began});
      if (answered.tick > globalThis.__diagBase) globalThis.__diagMoved = true;
    }`);
  if (process.env.COMPAT_WARM_CONTROL) {
    s = once(s, "more = warmPaced(() => warming(() => warm(engine.id)));", `
      const diagWarm = globalThis.__diagEvents.length;
      globalThis.__diagLog("warm-start", {id: diagWarm});
      globalThis.__diagQueueMark("queue-before-warm", diagWarm);
      more = warmPaced(() => warming(() => warm(engine.id)));
      globalThis.__diagLog("warm-return", {id: diagWarm, more});
      globalThis.__diagQueueMark("queue-after-warm", diagWarm);`);
    if (process.env.COMPAT_NO_WARM === "1") {
      s = once(s, "const warmBehindGate = () => {", 'const warmBehindGate = () => { globalThis.__diagLog("warm-disabled"); return;');
    }
  }
  const prefix = `
globalThis.__diagEvents = [];
globalThis.__diagLog = (event, detail = {}) => globalThis.__diagEvents.push({event, at: performance.timeOrigin + performance.now(), ...detail});
globalThis.__diagQueueMark = (event, id) => {
  globalThis.__diagLog(event + "-requested", {id});
  globalThis.__diagDevice?.queue.onSubmittedWorkDone().then(() => globalThis.__diagLog(event + "-done", {id}), error => globalThis.__diagLog(event + "-error", {id, error: String(error)}));
};
for (const method of ["createRenderPipeline", "createRenderPipelineAsync", "createComputePipeline", "createComputePipelineAsync"]) {
  const proto = globalThis.GPUDevice?.prototype;
  const native = proto?.[method];
  if (!native) continue;
  proto[method] = function(desc) {
    globalThis.__diagDevice = this;
    const id = globalThis.__diagEvents.length;
    globalThis.__diagLog("pipeline-start", {id, method, label: desc.label ?? "", entry: desc.fragment?.entryPoint ?? desc.compute?.entryPoint ?? ""});
    const result = native.call(this, desc);
    globalThis.__diagLog("pipeline-return", {id, method});
    if (method.endsWith("Async")) result.then(() => globalThis.__diagLog("pipeline-resolved", {id, method}), error => globalThis.__diagLog("pipeline-rejected", {id, method, error: String(error)}));
    return result;
  };
}
addEventListener("message", ({data}) => { if (data?.type === "compat-diag") self.postMessage({type: "compat-diag-result", events: globalThis.__diagEvents}); });
`;
  writeFileSync(game, prefix + s);
  const html = join(out, "index.html");
  const main = `<script>
globalThis.__diagEvents = [];
globalThis.__diagWorkers = [];
globalThis.__diagLog = (event, detail = {}) => globalThis.__diagEvents.push({event, at: performance.timeOrigin + performance.now(), ...detail});
for (const name of ["pointerdown", "pointerup", "click"]) addEventListener(name, e => globalThis.__diagLog(name, {trusted: e.isTrusted}), true);
const NativeWorker = Worker;
globalThis.Worker = class extends NativeWorker {
  constructor(...args) { super(...args); globalThis.__diagWorkers.push(this); this.addEventListener("message", ({data}) => { if (data?.type === "compat-diag-result") globalThis.__diagWorkerEvents = data.events; if (data?.type === "frame" && data.first) globalThis.__diagLog("first-frame-message", {boot: data.boot}); }); }
  postMessage(data, ...rest) { if (data?.type === "play") globalThis.__diagLog("page-play-send", {on: data.on}); return super.postMessage(data, ...rest); }
};
globalThis.__diagTake = async () => {
  for (const w of globalThis.__diagWorkers) w.postMessage({type: "compat-diag"});
  for (let n = 0; n < 100 && !globalThis.__diagWorkerEvents; n++) await new Promise(r => setTimeout(r, 50));
  return {page: globalThis.__diagEvents, worker: globalThis.__diagWorkerEvents ?? []};
};
</script>`;
  writeFileSync(html, once(readFileSync(html, "utf8"), "<head>", "<head>" + main));
}

if (import.meta.main) {
  const root = "E:/src/origin-engine-dev";
  const dir = join(root, "packages/engine/tests/web");
  let s = readFileSync(join(dir, "wire.ts"), "utf8");
  s = `import { instrument } from "${root}/.amp/in/scratch/wire-instrument.ts";\n` + s;
  s = once(s, 'const work = join(pkg, "dist", "test-wire");', 'const work = join(pkg, "dist", "test-wire-compat-diag");');
  s = once(s, '    dists[`${kind} ${pipelines}`] = out;', '    if (kind === "dev") instrument(out);\n    dists[`${kind} ${pipelines}`] = out;');
  if (process.env.COMPAT_PREWARM) {
    s = once(s, 'for (const kind of ["dev", "publish"] as const)\n  for (const pipelines of kind === "publish" ? (["auto", "sync", "async", "prewarm"] as const) : (["auto", "async"] as const)) {', 'for (const kind of ["dev"] as const)\n  for (const pipelines of ["sync", "prewarm"] as const) {');
  }
  const assertion = '    check(typeof t3.result === "number" && t3.result > (tHeld.result as number), `${label}: a tap on the gate starts the clock`, { before: tHeld.result, after: t3.result });';
  s = once(s, assertion, assertion + `
    // Only AFTER the unchanged assertion: collect completed-frame/pipeline timestamps.
    await new Promise(r => setTimeout(r, 3000));
    const diagnostic = await page.evalChild("localhost", "__diagTake()");
    const boot = await page.evalChild("localhost", "__runtime_probe__.boot");
    transcript.push({run: label, diagnostic, boot, before: tHeld.result, after: t3.result});
    console.log("DIAG " + label + " " + JSON.stringify(diagnostic));
    return;
`);
  const loops = `  for (const pipelines of ["auto", "async"] as const) for (const kind of ["dev", "publish"] as const) for (const isolate of [true, false]) await run(kind, isolate, pipelines, browser);
  for (const pipelines of ["sync", "prewarm"] as const) for (const isolate of [true, false]) await run("publish", isolate, pipelines, browser);
  await inPage(browser);`;
  s = once(s, loops, process.env.COMPAT_PREWARM
    ? '  for (let rep = 0; rep < 3; rep++) for (const pipelines of ["sync", "prewarm"] as const) for (const isolate of [true, false]) await run("dev", isolate, pipelines, browser);'
    : process.env.COMPAT_WARM_CONTROL
    ? '  for (const isolate of [true, false]) await run("dev", isolate, "auto", browser);'
    : '  for (const pipelines of ["auto", "async"] as const) for (const isolate of [true, false]) await run("dev", isolate, pipelines, browser);');
  writeFileSync(join(dir, "wire-compat-diag.local.ts"), s);
  s = once(s, 'import { androidOf, Browser, chromePath, type Page as CdpPage } from "./cdp.ts";', 'import { Firefox as Browser, firefoxPath as chromePath, type FirefoxPage as CdpPage } from "./bidi.ts";\nconst androidOf = (_: unknown) => false;');
  s = s.replaceAll("  browser.reverse(live.port);", "");
  s = once(s, 'await Browser.from(args, { chrome, reverse: [] })', 'await Browser.launch({ firefox: chrome, headless: false })');
  writeFileSync(join(dir, "wire-compat-diag-firefox.local.ts"), s);
}
