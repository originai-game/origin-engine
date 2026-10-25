// Existing hello-world browser benchmark, served to the authorized Android.
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve, extname } from "node:path";
import { loadavg } from "node:os";
const serial = "58051FDCR007QL";
const out = resolve("tests/evidence/r4/measure/android/tick");
const dist = resolve("target/tmp/r4-android-tick");
mkdirSync(out, { recursive:true });
mkdirSync(dist, { recursive:true });
for (const entry of ["tests/evidence/r4/measure/android-tick.page.mjs", "packages/engine/tests/web/hello.worker.ts"]) {
  const b = await Bun.build({ entrypoints:[resolve(entry)], outdir:dist, target:"browser", format:"esm" });
  if (!b.success) throw new Error(String(b.logs));
}
const adb = (...args) => {
  const r = Bun.spawnSync(["adb","-s",serial,...args]);
  if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr));
  return new TextDecoder().decode(r.stdout);
};
const results = [];
let answer;
const server = Bun.serve({ port:0, hostname:"127.0.0.1", async fetch(req) {
  const path = new URL(req.url).pathname;
  const headers = { "cache-control":"no-store", "cross-origin-opener-policy":"same-origin", "cross-origin-embedder-policy":"require-corp", "cross-origin-resource-policy":"cross-origin" };
  if (path === "/result") { answer(await req.json()); return new Response("ok", { headers }); }
  if (path === "/") return new Response('<!doctype html><style>canvas{width:320px;height:180px}</style><canvas></canvas><script type="module" src="/android-tick.page.js"></script>', { headers:{...headers,"content-type":"text/html"} });
  const file = Bun.file(path.startsWith("/web/") ? resolve(`packages/engine/dist${path}`) : `${dist}${path}`);
  if (!(await file.exists())) return new Response("not found", {status:404,headers});
  return new Response(file, { headers:{...headers,"content-type":extname(path)===".wasm"?"application/wasm":"text/javascript"} });
} });
adb("reverse", `tcp:${server.port}`, `tcp:${server.port}`);
try {
  for (let round=0;round<5;round++) {
    const before = loadavg();
    let timeout;
    const result = new Promise((resolve,reject)=>{
      answer = resolve;
      timeout = setTimeout(()=>reject(new Error("Android benchmark timed out")),120000);
    });
    adb("shell","am","start","-n","com.android.chrome/com.google.android.apps.chrome.Main","-d",`http://localhost:${server.port}/?round=${round}`);
    let value;
    try { value = await result; } finally { clearTimeout(timeout); }
    results.push({round,before,after:loadavg(),value});
    writeFileSync(`${out}/results.json`,JSON.stringify(results,null,2));
    if (value.error || value.entities !== 10000 || !value.advanced?.ok || !Number.isFinite(value.engineTickMs)) throw new Error(JSON.stringify(value));
  }
  if (new Set(results.map(r=>r.value.hash)).size !== 1) throw new Error("Android worlds differ between rounds");
} finally {
  adb("reverse","--remove",`tcp:${server.port}`);
  server.stop();
}
