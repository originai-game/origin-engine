import { readFileSync, writeFileSync } from "node:fs";
import assert from "node:assert/strict";
const root = "tests/evidence/r4/final-metal";
const read = (path) => JSON.parse(readFileSync(`${root}/${path}`, "utf8"));
const load = (dir) => {
  assert.match(readFileSync(`${root}/${dir}/status.txt`, "utf8"), /command_exit=0\nidle=true/);
  const values = readFileSync(`${root}/${dir}/load.tsv`, "utf8").trim().split("\n").map((s) => Number(s.split("\t")[1]));
  assert(values.length > 0 && values.every((v) => v < 8));
  return { min: Math.min(...values), max: Math.max(...values) };
};
const capture = [];
for (const host of ["native", "web"]) for (const scene of ["game", "meridian"]) {
  const data = read(`capture/${host}-${scene}/${host === "native" ? "capture" : "result"}.json`);
  assert.equal(data.samples.length, 30);
  const limit = host === "native" ? 30 : 60;
  capture.push({ host, scene, ...data.summary, limit, verdict: data.summary.p95 <= limit ? "PASS" : "FAIL" });
}
const comparisons = [];
for (const scene of ["reference", "spheres"]) {
  const sides = ["engine", "three"].map((side) => {
    const dir = `threejs/${scene}-${side}`;
    const data = read(`${dir}/results/threejs-comparison.json`);
    assert.equal(data.rounds, 2); assert.equal(data.frames, 600); assert.equal(data.summary.length, 1);
    return { ...data.summary[0], load: load(`${dir}/run`) };
  });
  const ratio = sides[1].sustainedMs / sides[0].sustainedMs;
  comparisons.push({ scene, sides, ratio, target: scene === "reference" ? 1.5 : null, verdict: scene === "reference" ? (ratio >= 1.5 ? "PASS" : "FAIL") : "no separate numeric budget" });
}
const result = { base: "70c120c0b2653aab0d013403ab02641d9154a8d1", captureLoad: load("capture-run"), capture, comparisons, lanes3d: { brotliBytes: 3370739, budgetBytes: 4194304, percentage: 3370739 / 4194304 * 100 }, stopped: "Step 4 WebGL2 reference ratio below 1.5; Lantern and separate Chrome first-frame/frame-time checks not run" };
writeFileSync(`${root}/summary.json`, JSON.stringify(result, null, 2) + "\n");
console.log(JSON.stringify(result, null, 2));
