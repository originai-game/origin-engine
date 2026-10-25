import { readFileSync, writeFileSync } from "node:fs";
import assert from "node:assert/strict";
const root = "tests/evidence/r4/final-metal/outstanding";
const json = (path) => JSON.parse(readFileSync(path, "utf8"));
const stats = (a) => {
  const s = [...a].sort((a, b) => a - b);
  return { median: s[Math.floor(s.length / 2)], min: s[0], max: s.at(-1), runs: a };
};
const load = (name, code = 0) => {
  const status = readFileSync(`${root}/${name}/status.txt`, "utf8");
  assert.equal(status, `command_exit=${code}\nidle=true\n`);
  const values = readFileSync(`${root}/${name}/load.tsv`, "utf8").trim().split("\n").map(x => Number(x.split("\t")[1]));
  assert(values.length && values.every(x => x < 8));
  return { min: Math.min(...values), max: Math.max(...values) };
};
const games = json(`${root}/perf-games/runs.json`);
assert.equal(games.length, 15);
const perfGames = [...new Set(games.map(x => x.game))].map(game => {
  const r = games.filter(x => x.game === game).map(x => x.result);
  assert.equal(r.length, 5);
  assert.equal(new Set(r.map(x => x.hash)).size, 1);
  for (const x of r) {
    assert.equal(x.tick, 600);
    assert.equal(x.tier, "metal");
    assert.equal(x.warnings.length, 0);
    assert.equal(x.unchecked.length, 0);
    assert.equal(x.frames.rendered, 595);
  }
  return { game, hash: r[0].hash, stepP50: stats(r.map(x => x.frames.stepMedianMs)), stepP95: stats(r.map(x => x.frames.stepP95Ms)), renderP50: stats(r.map(x => x.frames.renderMedianMs)), renderP95: stats(r.map(x => x.frames.renderP95Ms)) };
});
const chrome = [];
for (const game of ["game", "meridian-loop"]) for (const tier of ["webgpu", "webgl2"]) {
  const r = [1, 2, 3, 4, 5].map(i => json(`${root}/chrome/${i}/${game}-${tier}/device.json`).measured);
  for (const x of r) {
    assert.equal(x.tier, tier);
    assert.equal(x.errors.length, 0);
    assert(x.frames > 0 && x.ticks > 0);
    assert(x.device.crossOriginIsolated);
  }
  chrome.push({ game, tier, canvas: r[0].device.canvas, firstFrame: stats(r.map(x => x.firstFrameMs)), intervalP95: stats(r.map(x => x.interval.p95)), intervalMax: stats(r.map(x => x.interval.max)), workP50: stats(r.map(x => x.work.p50)), workP95: stats(r.map(x => x.work.p95)), ticks: r.map(x => x.ticks), frames: r.map(x => x.frames) });
}
const spheres = [1, 2, 3].map(i => {
  const stdout = readFileSync(`${root}/spheres-${i}/stdout.txt`, "utf8");
  const ms = Number(stdout.match(/perf extract-spheres-20k: best ([\d.]+) ms/)[1]);
  assert(stdout.includes("1 failed;"));
  assert(stdout.includes("baseline 0.531 ms, limit 0.611 ms"));
  return { ms, load: load(`spheres-${i}`, 101) };
});
const result = { head: readFileSync(`${root}/chrome/head.txt`, "utf8").trim(), load: { perfGames: load("perf-games"), chrome: load("chrome") }, perfGames, chrome, spheres: { runs: spheres, frameMs: stats(spheres.map(x => x.ms)), baselineMs: 0.531, limitMs: 0.531 * 1.15 } };
writeFileSync(`${root}/summary.json`, JSON.stringify(result, null, 2) + "\n");
console.log(JSON.stringify(result, null, 2));
