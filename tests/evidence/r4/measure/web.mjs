// Bun runner: web.mjs capture|terrain PROJECT OUT [--webgl].
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { loadavg } from "node:os";
import { settingsOf, webBuild } from "../../../../packages/engine/src/node/web-build.ts";
import { Browser, chromePath } from "../../../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../../../packages/engine/tests/web/serve.ts";
import { cookProject } from "../../../../packages/engine/tests/support/og-bin.ts";
import { decodePng } from "../../../../packages/engine/tests/support/png.ts";

const [kind, relative, output] = process.argv.slice(2);
const project = resolve(relative), out = resolve(output);
const dist = resolve(`target/tmp/r4-web-${kind}`);
mkdirSync(out, { recursive: true });
cookProject(project, process.cwd());
const entry = resolve("target/tmp/r4-web-entry.mjs");
mkdirSync(resolve("target/tmp"), { recursive: true });
writeFileSync(entry, `
import game from ${JSON.stringify(`${project}/src/main.ts`)};
import { registerCommand, boundHost } from ${JSON.stringify(resolve("packages/engine/src/runtime.ts"))};
registerCommand("r4outputs", (world) => {
  self.postMessage({ type:"r4outputs", value:JSON.parse(boundHost().abi.og_engine_take_outputs(world.id)) });
});
export default game;
`);
const built = await webBuild(project, entry, dist, false, settingsOf(project));
if (!built.ok) throw new Error(built.reason);
const page = await Bun.build({ entrypoints: [resolve("packages/engine/tests/web/reference.page.ts")], target: "browser", format: "esm" });
if (!page.success) throw new Error(String(page.logs));
writeFileSync(`${dist}/reference-page.js`, await page.outputs[0].text());
const html = '<!doctype html><style>html,body{margin:0}canvas{width:1280px;height:720px}</style><canvas></canvas><script type="module" src="/game/reference-page.js"></script>';
const server = serve({ dist, isolate: true, extra: { "/r4.html": html } });
const browser = await Browser.launch({ chrome: chromePath([]), width: 1280, height: 720 });
const log = [], loads = [];
const sampleLoad = () => { loads.push({ at: new Date().toISOString(), load: loadavg() }); };
sampleLoad();
const timer = setInterval(sampleLoad, 1000);
try {
  const tab = await browser.open("about:blank", log);
  await tab.send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 720, deviceScaleFactor: 1, mobile: false });
  await tab.send("Page.navigate", { url: `http://127.0.0.1:${server.port}/r4.html?profile=${settingsOf(project).profile}${process.argv.includes("--webgl") ? "&webgl=1" : ""}` });
  await tab.until("globalThis.__og_shell__ || globalThis.__og_shell_error__", 120000);
  const error = await tab.eval("globalThis.__og_shell_error__");
  if (error) throw new Error(error);
  const result = await tab.eval(`(async () => {
    const shell = globalThis.__og_shell__;
    const frame = async (elapsed = 1000/60) => {
      const start = performance.now();
      const answer = await shell.call("frame", elapsed);
      if (!answer || !answer.ok) throw new Error(JSON.stringify(answer));
      return { wallMs: performance.now()-start, ...answer };
    };
    const capture = async () => {
      const start = performance.now();
      let shot = null, polls = 0;
      while (!shot && polls < 2000) {
        shot = await shell.probe("capture", { width:1280, height:720 });
        polls++;
        if (!shot) await new Promise(r => setTimeout(r, 0));
      }
      if (!shot?.png) throw new Error("capture timed out or returned no PNG");
      return { ms: performance.now()-start, polls, png:shot.png };
    };
    const samples = [];
    if (${JSON.stringify(kind)} === "terrain") {
      for (const [action,args] of [["terrain_stream",["stream","0.25"]],["station",["terrain"]]]) {
        const answer=await shell.call("command",action,args);
        if(!answer.ok) throw new Error(JSON.stringify(answer));
      }
      const before=await shell.call("hash");
      samples.push(await frame(0));
      const after=await shell.call("hash");
      if (samples[0].tick!==0 || samples[0].ticks!==0 || before!==after) throw new Error("terrain first render advanced or changed the world");
      await shell.call("probe", "commands", "0:dump_stream:terrain.json");
      await frame();
      const stream = await new Promise((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error("outputs timeout")), 5000);
        const receive = event => {
          if (event.data.type !== "r4outputs") return;
          clearTimeout(timeout);
          shell.worker.removeEventListener("message", receive);
          resolve(event.data.value);
        };
        shell.worker.addEventListener("message", receive);
        shell.call("command", "r4outputs", []).catch(reject);
      });
      const shot = await capture();
      return { tier:shell.tier, samples, stream, before, after, png:shot.png };
    }
    for (let i=0;i<60;i++) await frame();
    let png;
    for (let i=-3;i<30;i++) {
      await frame();
      const shot = await capture();
      png = shot.png;
      if (i>=0) samples.push({ ms:shot.ms, polls:shot.polls });
    }
    return { tier:shell.tier, samples, png };
  })()`);
  const bytes = Buffer.from(result.png, "base64");
  const image = decodePng(bytes);
  if (image.width !== 1280 || image.height !== 720) throw new Error("wrong capture dimensions");
  writeFileSync(`${out}/capture.png`, bytes);
  delete result.png;
  sampleLoad();
  result.loads = loads;
  result.idle = loads.every(s => s.load[0] < 8);
  result.log = log;
  result.engineLog = await tab.eval("globalThis.__og_log__");
  const sorted = result.samples.map(s => s.ms ?? s.wallMs).sort((a,b)=>a-b);
  result.summary = { p50: sorted[Math.floor(sorted.length/2)], p95: sorted[Math.floor(sorted.length*0.95)], max: sorted.at(-1) };
  writeFileSync(`${out}/result.json`, JSON.stringify(result,null,2));
} finally {
  clearInterval(timer);
  await browser.close();
  server.stop();
}
