// Downloads the finished Meshy tasks of tools/assets.json to their `out`
// paths and prints each one's state, size and sha256 (README.md's table).
//
//   bun tools/fetch.ts [name ...]
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { call, data } from "./gateway.ts";

const root = join(import.meta.dir, "..");
const ledger = JSON.parse(readFileSync(join(root, "tools/assets.json"), "utf8")) as { tasks: Record<string, { task_id: string; out: string }> };
const names = process.argv.length > 2 ? process.argv.slice(2) : Object.keys(ledger.tasks);
const wanted = names.map(name => ({ name, ...ledger.tasks[name]! }));
const status = data(await call("generate_get_tasks", { tasks: wanted.map(t => ({ kind: "meshy", task_id: t.task_id })) })).items as any[];
for (const task of wanted) {
  const item = status.find(i => i.task_id === task.task_id);
  const state = String(item?.data?.status ?? item?.status ?? "unknown");
  const path = join(root, task.out);
  if (!existsSync(path) && /success/i.test(state)) {
    const grant = data(await call("generate_download", { task_id: task.task_id, kind: "meshy" }));
    const url = grant.url ?? grant.download_url ?? grant.href;
    if (!url) throw new Error(`${task.name}: no download URL in ${Object.keys(grant).join(", ")}`);
    const bytes = Buffer.from(await (await fetch(url)).arrayBuffer());
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, bytes);
  }
  const file = existsSync(path) ? readFileSync(path) : null;
  console.log(`${task.name}: ${state} ${item?.data?.progress ?? ""}${file ? ` ${task.out} ${file.length} bytes sha256 ${createHash("sha256").update(file).digest("hex")}` : ""}`);
}
