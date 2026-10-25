import { Shell } from "../../../../packages/engine/src/web/shell.ts";
try {
  const shell = await Shell.start({
    canvas:document.querySelector("canvas"),
    worker:new Worker("/hello.worker.js", {type:"module"}),
    glue:"/web/origin-engine.js", wasm:"/web/origin-engine_bg.wasm",
    autoplay:false, prime:false,
  });
  const spawned = await shell.call("command","spawn_wave",["10000"]);
  if (!spawned.ok) throw new Error(JSON.stringify(spawned));
  const warm = await shell.call("advance",600);
  if (!warm.ok) throw new Error(JSON.stringify(warm));
  const before = await shell.call("probe","dump",{what:"world"});
  const entityBefore = await shell.call("probe","dump",{what:"entity",id:"enemy-0-0"});
  const start = performance.now();
  const advanced = await shell.call("advance",2000);
  const elapsed = performance.now()-start;
  const after = await shell.call("probe","dump",{what:"world"});
  const entityAfter = await shell.call("probe","dump",{what:"entity",id:"enemy-0-0"});
  if (!advanced.ok || after.tick-before.tick !== 2000 || after.entities !== 10000 || before.hash === after.hash) throw new Error("tick or entity/hash validation failed");
  await fetch("/result", {method:"POST",body:JSON.stringify({
    userAgent:navigator.userAgent,crossOriginIsolated,transport:shell.transport,
    entities:after.entities,advanced,beforeTick:before.tick,afterTick:after.tick,
    hash:after.hash,engineTickMs:elapsed/2000,elapsedMs:elapsed,
    systemsBefore:before.systems,systemsAfter:after.systems,
    entityBefore,entityAfter,
  })});
} catch (error) {
  await fetch("/result",{method:"POST",body:JSON.stringify({error:String(error)})});
}
