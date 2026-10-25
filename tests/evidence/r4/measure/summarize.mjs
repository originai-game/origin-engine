import { readFileSync, writeFileSync } from "node:fs";
const root = "tests/evidence/r4/measure";
const read = p => JSON.parse(readFileSync(`${root}/${p}`,"utf8"));
const require = (condition,message) => { if(!condition) throw new Error(message); };
const stats = values => { const s=[...values].sort((a,b)=>a-b); return {median:s[Math.floor(s.length/2)],min:s[0],max:s.at(-1)}; };
const terrain = [];
for(const tier of ["webgpu","webgl2"]) {
  const rows=Array.from({length:5},(_,i)=>read(`terrain-tick0-${tier}-${i+1}/result.json`));
  for(const r of rows) {
    const frame=r.samples[0], stream=JSON.parse(r.stream[0].data).terrain;
    require(frame.tick===0 && frame.ticks===0 && frame.stepMs===0 && r.before===r.after,"terrain changed simulation");
    require(stream.decoded===5 && stream.resident===5 && stream.waiting===0 && stream.inFlight===0,"terrain tiles did not settle");
  }
  terrain.push({tier,wallMs:stats(rows.map(r=>r.samples[0].wallMs)),renderMs:stats(rows.map(r=>r.samples[0].renderMs)),tileSettleMs:stats(rows.map(r=>JSON.parse(r.stream[0].data).terrain.worstFrameMs)),load:stats(rows.flatMap(r=>r.loads.map(s=>s.load[0]))),verdict:"pass: tile settle measured separately; no numerical terrain-settle budget specified"});
}
const runs=read("lanes/results.json");
require(runs.length===30,"missing lanes runs");
const lanes=[];
for(const bodies of [1000,2000]) {
  const rows=runs.filter(r=>r.bodies===bodies);
  require(new Set(rows.map(r=>JSON.stringify(r.result.hashes))).size===1,`whole-world parity ${bodies}`);
  for(const mode of ["non-isolated","isolated-off","isolated-on"]) {
    const selected=rows.filter(r=>r.mode===mode);
    for(const r of selected) for(const s of r.result.stats) {
      const p=s.facts.physics;
      require(p.ok && p.value.bodies===bodies && p.value.tick===s.tick-1,"physics stats do not describe the executed step");
      require(p.value.pool===(bodies===2000 && mode==="isolated-on"),"wrong physics pool use");
    }
    lanes.push({bodies,mode,hashes:selected[0].result.hashes,lastPhysicsMs:stats(selected.map(r=>r.result.stats.at(-1).facts.physics.value.ms)),parallelCalls:selected[0].result.lanes.parallelCalls,threads:selected[0].result.lanes.threads,load:stats(selected.flatMap(r=>r.loads.map(s=>s[0]))),verdict:"pass: whole-world hashes and direct physics stats"});
  }
}
const ticks=read("android/tick/results.json");
require(ticks.length===5 && new Set(ticks.map(r=>r.value.hash)).size===1,"Android repeatability");
for(const {value:v} of ticks) {
  const a=v.entityBefore.components["og.Transform"].position;
  const b=v.entityAfter.components["og.Transform"].position;
  require(v.entities===10000 && v.afterTick-v.beforeTick===2000,"Android tick count");
  require(a[0]===b[0] && a[1]===b[1] && Math.abs(b[2]-a[2]-2000/60)<0.001,"Move did not integrate velocity");
}
const first=Array.from({length:5},(_,i)=>read(`android/run-${i+1}.json`));
const android={firstFrameMs:stats(first.map(r=>r.firstFrameMs)),wholeTickMs:stats(ticks.map(r=>r.value.engineTickMs)),moveMs:stats(ticks.map(({value:v})=>{
  const b=v.systemsBefore.find(s=>s.name==="Move"),a=v.systemsAfter.find(s=>s.name==="Move");
  return (a.totalMs-b.totalMs)/(a.runs-b.runs);
})),load:stats(ticks.flatMap(r=>[r.before[0],r.after[0]])),verdict:"pass on Pixel 10 over USB localhost; not-measurable for the specified 2022 mid-range Android/4G acceptance"};
const summary={base:"1bb341e51b22bdcdaf69a3eb60aac5265366196f",terrain,lanes,android};
writeFileSync(`${root}/summary.json`,JSON.stringify(summary,null,2));
console.log(JSON.stringify(summary,null,2));
