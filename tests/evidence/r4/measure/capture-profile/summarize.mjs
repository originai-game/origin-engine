import { readFileSync, writeFileSync } from "node:fs";
import assert from "node:assert/strict";
const root = "tests/evidence/r4/measure/capture-profile";
const q = (values) => {
  const xs = [...values].sort((a, b) => a - b);
  return { p50: xs[15], p95: xs[28], min: xs[0], max: xs[29] };
};
const result = [];
for (const mode of ["default", "fast-up", "fast-none", "store-none"]) {
  for (const game of ["game", "meridian"]) {
    const dir = `${root}/${mode}/${game}`;
    const capture = JSON.parse(readFileSync(`${dir}/capture.json`, "utf8"));
    const lines = readFileSync(`${dir}/stderr.txt`, "utf8").split("\n");
    const stages = (prefix) => lines.filter((s) => s.startsWith(prefix + " ")).map((s) => s.split(" ").slice(1).map(Number));
    const host = stages("CAPTURE_HOST");
    const runtime = stages("CAPTURE_RUNTIME");
    assert.equal(host.length, 33);
    assert.equal(runtime.length, 33);
    assert.equal(capture.samples.length, 30);
    const samples = capture.samples.map((s, i) => {
      const [render, readback] = host[i + 3];
      const [extract, png] = runtime[i + 3];
      const residual = s.ms - render - readback - extract - png;
      assert(s.load[0] < 8 && residual >= 0);
      return { total: s.ms, extractRender: extract + render, readback, png, residual, bytes: s.bytes };
    });
    result.push({ mode, game, stages: Object.fromEntries(Object.keys(samples[0]).map((k) => [k, q(samples.map((s) => s[k]))])), samples });
  }
}
const loads = readFileSync(`${root}/run/load.tsv`, "utf8").trim().split("\n").map((l) => Number(l.split("\t")[1]));
assert(loads.length > 0 && loads.every((v) => v < 8));
assert.match(readFileSync(`${root}/run/status.txt`, "utf8"), /command_exit=0\nidle=true/);
writeFileSync(`${root}/summary.json`, JSON.stringify({ load: { min: Math.min(...loads), max: Math.max(...loads) }, result }, null, 2) + "\n");
console.log("mode game total extract+render readback png residual bytes (p50/p95)");
for (const r of result) console.log(r.mode, r.game, ...Object.values(r.stages).map((v) => `${v.p50.toFixed(3)}/${v.p95.toFixed(3)}`));
