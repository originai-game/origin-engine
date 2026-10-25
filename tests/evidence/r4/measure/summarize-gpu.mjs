import { readFileSync, writeFileSync } from "node:fs";
const root = "tests/evidence/r4/measure";
const read = path => JSON.parse(readFileSync(`${root}/${path}`, "utf8"));
const check = (ok, message) => { if (!ok) throw new Error(message); };
const quantiles = values => {
  const s = values.toSorted((a,b)=>a-b);
  check(s.length > 0 && s.every(Number.isFinite), "missing finite samples");
  return { p50:s[Math.floor(s.length/2)], p95:s[Math.floor(s.length*.95)], max:s.at(-1) };
};
const idle = path => {
  check(readFileSync(`${root}/${path}/status.txt`, "utf8") === "command_exit=0\nidle=true\n", `${path}: not an idle successful run`);
  const before = readFileSync(`${root}/${path}/idle-before.tsv`, "utf8").trim().split("\n").map(s=>s.split("\t"));
  const lastBad = before.findLastIndex(([,n])=>Number(n)>=8);
  const ready = before.slice(lastBad+1);
  check(ready.length>1 && Date.parse(ready.at(-1)[0])-Date.parse(ready[0][0])>=60000, `${path}: no minute below 8`);
  const loads = readFileSync(`${root}/${path}/load.tsv`, "utf8").trim().split("\n").map(s=>Number(s.split("\t")[1]));
  check(loads.every(n=>Number.isFinite(n)&&n<8), `${path}: load cap exceeded`);
  return { min:Math.min(...loads), max:Math.max(...loads) };
};
const captureLoad = idle("capture-idle-run");
const captures = [];
for (const game of ["game", "meridian"]) for (const host of ["native", "web"]) {
  const r = read(`capture-idle/${host}-${game}/${host === "native" ? "capture.json" : "result.json"}`);
  check(r.samples.length === 30 && r.tier === (host === "native" ? "metal" : "webgpu"), "wrong capture count/tier");
  const ms = quantiles(r.samples.map(s=>s.ms)), budgetMs = host === "native" ? 30 : 60;
  captures.push({game,host,ms,budgetMs,p95Verdict:ms.p95<=budgetMs?"pass":"fail",hardCapVerdict:ms.max<=budgetMs?"pass":"fail",load:captureLoad});
}
const screenshots = ["game", "meridian"].map(game=>({game,...read(`capture-idle/screenshot-${game}/summary.json`),load:captureLoad}));
const threejs = [];
for (let n=1;n<=9;n++) {
  const load = idle(`threejs-separated/${n}/run`);
  const r = read(`threejs-separated/${n}/results/threejs-comparison.json`);
  check(r.frames===600 && r.warm===60 && r.rounds===2 && r.runs.length===2 && r.summary.length===1, "wrong comparison sample count");
  threejs.push({...r.summary[0],load,roundSustainedMs:r.runs.map(s=>s.sustainedMs)});
}
const ref = threejs.filter(r=>r.scene==="reference");
const instanced = ref.find(r=>r.mode==="three WebGL2, InstancedMesh").sustainedMs;
const referenceBudgets = ref.filter(r=>r.mode.startsWith("engine")).map(r=>({mode:r.mode,ratio:instanced/r.sustainedMs,target:1.5,verdict:instanced/r.sustainedMs>=1.5?"pass":"fail"}));
const perf = [], polish = [];
for (const profile of ["release", "dev"]) {
  const load = idle(`perf-games-${profile}-run`), rows = read(`perf-games/runs-${profile}.json`);
  check(rows.length===30, `${profile}: expected 30 play runs`);
  for (const game of ["hollowbell", "kestrel-stair", "lantern-vale"]) {
    for (const paced of [false,true]) {
      const selected = rows.filter(r=>r.game===game && r.paced===paced);
      check(selected.length===5 && selected.every(r=>r.result.tier==="metal" && r.result.ticks===600 && r.result.tick>=600 && r.result.entities>0 && r.result.frames.stepped>0 && r.result.frames.rendered>0), "wrong perf workload");
      const frames = Object.fromEntries(["stepMedianMs","stepP95Ms","renderMedianMs","renderP95Ms"].map(key=>[key,quantiles(selected.map(r=>r.result.frames[key])).p50]));
      const systems = selected[0].result.systems.map(s=>({name:s.name,meanMs:quantiles(selected.map(r=>r.result.systems.find(t=>t.name===s.name).meanMs)).p50})).sort((a,b)=>b.meanMs-a.meanMs).slice(0,5);
      perf.push({profile,game,paced,frames,systems,endTicks:selected.map(r=>r.result.tick),hashes:[...new Set(selected.map(r=>r.result.hash))],load,verdict:profile==="release"?(frames.stepP95Ms<8&&frames.renderP95Ms<8?"pass":"fail"):(frames.stepP95Ms<16?"pass":"fail")});
    }
    const r = read(`perf-games/${profile}/polish/${game}/polish.json`);
    check(r.playtests.length>0 && !r.findings.some(f=>f.measure==="run") && r.playtests.every(t=>t.frame.rendered&&t.frame.tier==="metal"), "polish did not run every playtest on Metal");
    polish.push({profile,game,scores:r.scores,worstFrameP95Ms:Math.max(...r.playtests.map(t=>t.frame.frameP95Ms)),stepP95MedianMs:quantiles(r.playtests.map(t=>t.frame.stepP95Ms)).p50,stepP95WorstMs:Math.max(...r.playtests.map(t=>t.frame.stepP95Ms)),renderP95WorstMs:Math.max(...r.playtests.map(t=>t.frame.renderP95Ms)),load});
  }
}
const summary = {base:"1bb341e51b22bdcdaf69a3eb60aac5265366196f",captures,screenshots,threejs,referenceBudgets,perf,polish};
writeFileSync(`${root}/gpu-summary.json`, JSON.stringify(summary,null,2)+"\n");
console.log(JSON.stringify({...summary,screenshots:screenshots.map(({samples,...rest})=>rest)},null,2));
