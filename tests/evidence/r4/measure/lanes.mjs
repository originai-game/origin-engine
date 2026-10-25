// Diagnostic Chrome whole-world hashes and direct og.physics.stats.
import { cpSync, mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { loadavg } from "node:os";
import { settingsOf, webBuild } from "../../../../packages/engine/src/node/web-build.ts";
import { Browser, chromePath } from "../../../../packages/engine/tests/web/cdp.ts";
import { serve } from "../../../../packages/engine/tests/web/serve.ts";
const out = resolve("tests/evidence/r4/measure/lanes");
const project = resolve("crates/og-tests/reference-scene");
const work = resolve("target/tmp/r4-lanes");
mkdirSync(work,{recursive:true});
mkdirSync(out,{recursive:true});
const runs = [];
for (const bodies of [1000,2000]) {
  const entry = `${work}/pile-${bodies}.ts`;
  writeFileSync(entry, `
import { og } from "@origingame/engine";
import { pile } from ${JSON.stringify(resolve("packages/engine/tests/support/lanes-pile-scene.ts"))};
export default og.game({seed:1,render:{antialias:"taa"},start(world){pile(world,${bodies},true)},facts(world){return {physics:og.physics.stats(world)}}});
`);
  const dist = `${work}/web-${bodies}`;
  const built = await webBuild(project,entry,dist,false,settingsOf(project));
  if (!built.ok) throw new Error(built.reason);
  cpSync(resolve("target/r4-lanes-dev"),dist,{recursive:true});
  const page = await Bun.build({entrypoints:[resolve("packages/engine/bench/lanes.page.ts")],target:"browser",format:"esm"});
  if (!page.success) throw new Error(String(page.logs));
  writeFileSync(`${dist}/lanes-page.js`,await page.outputs[0].text());
  for (let round=0;round<5;round++) {
    for (const [mode,isolate,offered] of [["non-isolated",false,true],["isolated-off",true,false],["isolated-on",true,true]]) {
      const server = serve({dist,isolate,extra:{"/r4.html":'<style>body{margin:0}canvas{width:1280px;height:720px}</style><canvas></canvas><script type="module" src="/game/lanes-page.js"></script>'}});
      const browser = await Browser.launch({chrome:chromePath([]),width:1280,height:720});
      const log = [], loads = [loadavg()];
      const timer = setInterval(()=>loads.push(loadavg()),1000);
      try {
        const tab = await browser.open("about:blank",log);
        await tab.send("Emulation.setDeviceMetricsOverride",{width:1280,height:720,deviceScaleFactor:1,mobile:false});
        await tab.send("Page.navigate",{url:`http://127.0.0.1:${server.port}/r4.html${offered?"":"?lanes=0"}`});
        await tab.until("globalThis.__og_shell__ || globalThis.__og_shell_error__",120000);
        const error = await tab.eval("globalThis.__og_shell_error__");
        if (error) throw new Error(error);
        const result = await tab.eval(`(async()=>{
          const shell=globalThis.__og_shell__, hashes=[], stats=[];
          for(let i=0;i<660;i++) {
            const frame=await shell.call("frame",1000/60);
            if(!frame.ok) throw new Error(JSON.stringify(frame));
            if(i===59 || i===359 || i===659) {
              hashes.push({tick:frame.tick,hash:await shell.call("hash")});
              const state=await shell.probe("state");
              stats.push({tick:frame.tick,facts:state.facts,timings:state.timings});
            }
          }
          return {hashes,stats,lanes:await shell.call("lanes"),ready:shell.ready,tier:shell.tier};
        })()`);
        if (result.ready.lanes.on !== (isolate && offered)) throw new Error("wrong lanes mode");
        if (result.hashes.some(h=>! /^[0-9a-f]{16}$/.test(h.hash))) throw new Error("missing whole-world hash");
        runs.push({bodies,round,mode,loads,result,log});
        writeFileSync(`${out}/results.json`,JSON.stringify(runs,null,2));
      } finally {
        clearInterval(timer);
        await browser.close();
        server.stop();
      }
    }
  }
}
for (const bodies of [1000,2000]) {
  const selected=runs.filter(r=>r.bodies===bodies);
  if (new Set(selected.map(r=>JSON.stringify(r.result.hashes))).size!==1) throw new Error(`whole-world hashes differ for ${bodies} bodies`);
}
