import { mkdirSync, readFileSync, writeFileSync, unlinkSync } from "node:fs";
import { resolve } from "node:path";
import { decodePng } from "../../../../packages/engine/tests/support/png.ts";
const project = process.argv[2], out = resolve(process.argv[3]);
mkdirSync(out, { recursive:true });
const commands = Array.from({length:33}, (_,i)=>`${i+2}:screenshot:shot-${i}.png`).join(";");
const argv = [resolve("packages/engine/native/og.darwin-arm64"), "run", project, "--headless", "--ticks", "2042", "--size", "1280x720", "--commands", commands, "--out", out, "--json"];
writeFileSync(`${out}/command.json`, JSON.stringify(argv,null,2));
const child = Bun.spawn(argv, {env:{...process.env,OG_ENGINE_PACKAGE:resolve("packages/engine"),BUN_OPTIONS:`--preload=${resolve("tests/evidence/r4/measure/screenshot-preload.mjs")}`,R4_SCREENSHOT_TIMES:`${out}/frames.json`},stdout:"pipe",stderr:"pipe"});
const [stdout,stderr,exit] = await Promise.all([new Response(child.stdout).text(),new Response(child.stderr).text(),child.exited]);
writeFileSync(`${out}/stdout.txt`,stdout); writeFileSync(`${out}/stderr.txt`,stderr);
if(exit) throw new Error(`og run exit ${exit}`);
const result = JSON.parse(stdout.trim().split("\n").at(-1));
if(result.warnings.length || result.unchecked.length) throw new Error("unchecked screenshots");
const frames = JSON.parse(readFileSync(`${out}/frames.json`,"utf8"));
const samples = [];
for(let i=0;i<33;i++) {
  const image=decodePng(readFileSync(`${out}/shot-${i}.png`));
  if(image.width!==1280 || image.height!==720) throw new Error("wrong PNG size");
  // Commands pump at the pre-step tick; frame answers name the post-step tick.
  const frame=frames.find(f=>f.tick===(i+2)*60+1);
  if(!frame || !frame.ok || frame.ticks!==1) throw new Error("missing screenshot frame");
  if(i>=3) samples.push({...frame,captureResidualMs:frame.wallMs-frame.stepMs-frame.renderMs});
  if(i!==32) unlinkSync(`${out}/shot-${i}.png`);
}
const q = xs => { const s=xs.toSorted((a,b)=>a-b); return {p50:s[15],p95:s[28],max:s.at(-1)}; };
writeFileSync(`${out}/summary.json`,JSON.stringify({method:"33 real og run --commands screenshots at integer seconds 2..34; discard first 3. Time native frame ABI wall clock; includes screenshot plus one simulation tick and normal render, excludes output drain/disk writes. Residual subtracts reported step/render; it estimates internal capture+PNG cost, not an independently timed span.",wallMs:q(samples.map(s=>s.wallMs)),captureResidualMs:q(samples.map(s=>s.captureResidualMs)),samples},null,2));
