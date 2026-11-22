// Native cold time to the first frame with Han text: usage
//   bun cold-native.ts <og> <project> <out-dir>
// Cooks <project> afresh, spawns `og run --headless` on this machine's GPU with
// screenshots at ticks 0, 1, 3 and 30 and the UI dump at tick 3, and stamps each
// output line with the ms since spawn. Prints one JSON line: the ms of each
// screenshot and whether its Han lines have ink. Writes <out-dir>/f<tick>.png,
// stamped.log and cold-native.json.
// Exits 1 when no frame by tick 30 has ink in every text line (a blank or tofu
// line: a real finding), 2 on a usage or run error.
import { mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { decodePng } from "../../packages/engine/src/node/png.ts";

const [og, projectArg, outArg] = process.argv.slice(2);
if (!og || !projectArg || !outArg) {
  console.error("usage: bun cold-native.ts <og> <project> <out-dir>");
  process.exit(2);
}
const project = resolve(projectArg), out = resolve(outArg);
mkdirSync(out, { recursive: true });
rmSync(join(project, "cooked"), { recursive: true, force: true });
if (Bun.spawnSync([og, "cook", project], { stdout: "inherit", stderr: "inherit" }).exitCode !== 0) process.exit(2);

const TICKS = [0, 1, 3, 30];
const script = [...TICKS.map((t) => `${(t / 60).toFixed(4)}:screenshot:f${t}.png`), `${(3 / 60).toFixed(4)}:dump_ui`, "0.6:quit"].sort((a, b) => parseFloat(a) - parseFloat(b));
const t0 = performance.now();
const p = Bun.spawn([og, "run", project, "--headless", "--no-check", "--size", "1280x720", "--out", out, "--commands", script.join(";")], { stdout: "pipe", stderr: "pipe" });
const stamped: string[] = [];
const read = async (s: ReadableStream<Uint8Array>) => {
  const dec = new TextDecoder();
  let buf = "";
  for await (const chunk of s) {
    const ms = Math.round(performance.now() - t0);
    buf += dec.decode(chunk);
    const parts = buf.split("\n");
    buf = parts.pop()!;
    for (const l of parts) stamped.push(`${String(ms).padStart(6)} ${l}`);
  }
};
await Promise.all([read(p.stdout as ReadableStream<Uint8Array>), read(p.stderr as ReadableStream<Uint8Array>)]);
await p.exited;
writeFileSync(join(out, "stamped.log"), stamped.join("\n") + "\n");
if (p.exitCode !== 0) {
  console.error(`cold-native: og run exited ${p.exitCode}; see ${join(out, "stamped.log")}`);
  process.exit(2);
}

const dumpFile = readdirSync(out).find((f) => f.startsWith("dump_ui") && f.endsWith(".json"));
const elements = dumpFile ? (JSON.parse(readFileSync(join(out, dumpFile), "utf8")).elements ?? []) : [];
const lines: Array<[string, number[]]> = elements.flatMap((e: { text?: string; textLines?: number[][] }) => (e.textLines ?? []).map((l) => [e.text ?? "", l] as [string, number[]]));
/** Pixels in each text line far from the ground beside it. */
function ink(file: string): Record<string, number> {
  const img = decodePng(readFileSync(file));
  const at = (x: number, y: number) => (Math.min(img.height - 1, Math.max(0, Math.round(y))) * img.width + Math.min(img.width - 1, Math.max(0, Math.round(x)))) * 4;
  const r: Record<string, number> = {};
  for (const [text, [x, y, w, h]] of lines) {
    const bg = at(x! - 3, y! + h! / 2);
    let n = 0;
    for (let py = y!; py < y! + h!; py++)
      for (let px = x!; px < x! + w!; px++) {
        const i = at(px, py);
        if (Math.abs(img.rgba[i]! - img.rgba[bg]!) + Math.abs(img.rgba[i + 1]! - img.rgba[bg + 1]!) + Math.abs(img.rgba[i + 2]! - img.rgba[bg + 2]!) > 120) n++;
      }
    r[text] = (r[text] ?? 0) + n;
  }
  return r;
}
const frames = TICKS.map((t) => {
  const line = stamped.find((l) => l.includes(`screenshot: wrote`) && l.includes(`f${t}.png`));
  const ms = line ? Number(line.trim().split(/\s+/)[0]) : null;
  return { tick: t, ms, ink: ink(join(out, `f${t}.png`)) };
});
const whole = frames.find((f) => lines.length > 0 && Object.values(f.ink).every((n) => n > 0));
const result = { firstWholeTextTick: whole?.tick ?? null, firstWholeTextMs: whole?.ms ?? null, frames };
writeFileSync(join(out, "cold-native.json"), JSON.stringify(result, null, 1));
console.log(JSON.stringify(result));
if (!whole) {
  console.error(`cold-native: no frame by tick 30 draws every text line (${lines.map((l) => l[0]).join(" | ")})`);
  process.exit(1);
}
