// Every vulkan-cpu frame against metal, for scripts/wave-end.sh: one row per
// frame with og-tests' figures (outliers over 32/255 per channel, RGB RMSE;
// templates/fx-showcase/tests/frames.ts's `difference`, the rule the
// playtests and og-tests judge by) against its metal reference and, when
// given, against the committed vulkan-cpu one.
//
//   bun scripts/frames-vs-metal.ts --out <file.tsv>
//       --set <name> <frames dir> <metal dir> [<committed vulkan-cpu dir>] ...
//
// Prints the table, writes it as TSV to --out, exits 0 (a measurement, not a
// gate; a frame with no metal counterpart is listed as such).
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { decodePng, difference } from "../templates/fx-showcase/tests/frames.ts";

const argv = process.argv.slice(2);
let out = "";
const sets: { name: string; frames: string; metal: string; committed?: string }[] = [];
for (let i = 0; i < argv.length; i++) {
  if (argv[i] === "--out") out = argv[++i]!;
  else if (argv[i] === "--set") {
    const [name, frames, metal] = [argv[++i]!, argv[++i]!, argv[++i]!];
    const committed = argv[i + 1] && !argv[i + 1]!.startsWith("--") ? argv[++i] : undefined;
    sets.push({ name, frames, metal, committed });
  } else {
    process.stderr.write(`frames-vs-metal: unknown argument ${argv[i]}\n`);
    process.exit(2);
  }
}
if (!out || sets.length === 0) {
  process.stderr.write("usage: bun scripts/frames-vs-metal.ts --out <file.tsv> --set <name> <frames> <metal> [<committed>] ...\n");
  process.exit(2);
}

const load = (p: string) => decodePng(new Uint8Array(readFileSync(p)));
const fmt = (d: { outliers: number; rmse: number } | null) =>
  d ? [(d.outliers * 100).toFixed(3), d.rmse.toFixed(5)] : ["-", "-"];
const rows: string[][] = [["set", "frame", "outliers % vs metal", "RMSE vs metal", "outliers % vs committed", "RMSE vs committed"]];
let worst = { rmse: -1, at: "" };
for (const s of sets) {
  if (!existsSync(s.frames)) {
    rows.push([s.name, `(no ${s.frames})`, "-", "-", "-", "-"]);
    continue;
  }
  for (const f of readdirSync(s.frames).filter((f) => f.endsWith(".png")).sort()) {
    const frame = load(join(s.frames, f));
    const m = join(s.metal, f);
    const vsMetal = existsSync(m) ? difference(frame, load(m)) : null;
    const c = s.committed ? join(s.committed, f) : "";
    const vsCommitted = c && existsSync(c) ? difference(frame, load(c)) : null;
    if (vsMetal && vsMetal.rmse > worst.rmse) worst = { rmse: vsMetal.rmse, at: `${s.name}/${f}` };
    rows.push([s.name, f, ...fmt(vsMetal), ...fmt(vsCommitted)]);
  }
}
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, rows.map((r) => r.join("\t")).join("\n") + "\n");
const widths = rows[0]!.map((_, k) => Math.max(...rows.map((r) => r[k]!.length)));
for (const r of rows) console.log(r.map((v, k) => (k < 2 ? v.padEnd(widths[k]!) : v.padStart(widths[k]!))).join("  "));
console.log(`${rows.length - 1} frames; the largest RMSE against metal: ${worst.rmse < 0 ? "-" : `${worst.rmse.toFixed(5)} (${worst.at})`}; table in ${out}`);
