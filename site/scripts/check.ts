// What still stands in for the final page: interim and pending media slots,
// numbers not yet measured, live scenes not yet built.
//
//   bun scripts/check.ts            list them
//   bun scripts/check.ts --final    exit 1 if any remains (run before a deploy)
import index from "../src/generated/media.json";
import { live } from "../src/live.ts";

const final = process.argv.includes("--final");
const open: string[] = [];
for (const [slot, r] of Object.entries(index as Record<string, { status: string; from: { pending?: string } }>)) {
  if (r.status !== "final") open.push(`media ${slot}: ${r.status}${r.from.pending ? ` (${r.from.pending})` : ""}`);
}
// numbers.ts reads git and the manifest relative to the site directory.
process.chdir(new URL("..", import.meta.url).pathname);
const { numbers } = await import("../src/numbers.ts");
for (const n of numbers) if (n.pending) open.push(`number ${n.key}: ${n.label.en}`);
open.push("number benchmark: the comparison with three.js and other web engines");
for (const [name, path] of Object.entries(live)) if (!path) open.push(`live ${name}: not built yet`);
process.stdout.write(open.length ? `${open.length} open:\n${open.map((l) => `  ${l}`).join("\n")}\n` : "nothing open\n");
if (final && open.length) process.exit(1);
