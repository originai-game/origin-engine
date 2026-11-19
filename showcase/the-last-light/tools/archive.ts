// Keeps this project's generated sources as private objects in the
// platform's engine asset store (docs/ci.md §Generated sources): for each
// Meshy task of tools/assets.json, its model.glb, the reference images it
// was made from and a task.json receipt (prompt, ids, sizes and checksums).
// The receipt's sha256 is recorded on the task as `archive` in
// tools/assets.json; the receipt names every other object by its sha256.
//
//   bun tools/archive.ts [name...]
//
// Needs OG_ASSETS_KEY or OG_PLATFORM_DEVELOPMENT_KEY (a key the store lets write).
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const STORE = `${(process.env.OG_ASSETS_URL ?? "https://origingame.ai").replace(/\/+$/, "")}/api/engine-assets/sha256`;
const key = process.env.OG_ASSETS_KEY || process.env.OG_PLATFORM_DEVELOPMENT_KEY;
if (!key) throw new Error("set OG_ASSETS_KEY or OG_PLATFORM_DEVELOPMENT_KEY");

const sha256 = (data: Buffer | string) => createHash("sha256").update(data).digest("hex");

/** Stores `body` privately (a stored object stays as it is); answers its address. */
async function keep(what: string, body: Buffer): Promise<string> {
  const sha = sha256(body);
  const put = await fetch(`${STORE}/${sha}?visibility=private`, {
    method: "PUT", body, headers: { authorization: `Bearer ${key}`, "content-type": "application/octet-stream", "content-length": String(body.length) },
  });
  if (!put.ok) throw new Error(`${what}: the store answered ${put.status} ${(await put.text()).slice(0, 300)}`);
  console.log(`  ${what}: ${put.status === 201 ? "kept" : "already kept"} ${body.length} bytes sha256 ${sha}`);
  return sha;
}

interface Task {
  operation: string; operation_id: string; task_id: string; out: string;
  reference?: string; reference_media?: string[]; image_model?: string; input?: string; action_ids?: number[];
}
const root = join(import.meta.dir, "..");
const ledgerFile = join(root, "tools/assets.json");
const ledger = JSON.parse(readFileSync(ledgerFile, "utf8")) as { tasks: Record<string, Task & { archive?: string }> };
const names = process.argv.slice(2);
for (const [name, task] of Object.entries(ledger.tasks)) {
  if (names.length && !names.includes(name)) continue;
  console.log(`${name}: ${task.operation_id}`);
  const model = readFileSync(join(root, task.out));
  const references = (task.reference ?? "").split(" ").filter(Boolean);
  const concepts = references.map((file, i) => {
    const bytes = readFileSync(join(root, file));
    const prompt = file.replace(/\.png$/, ".txt");
    return {
      key: references.length === 1 ? "concept.png" : `concept-${file.includes("back") ? "back" : "front"}.png`,
      tool: "generate_image", image_model: task.image_model, media_id: task.reference_media?.[i],
      prompt: existsSync(join(root, prompt)) ? readFileSync(join(root, prompt), "utf8").trim() : undefined,
      bytes: bytes.length, sha256: sha256(bytes), body: bytes,
    };
  });
  const receipt = {
    asset: name,
    purpose: "origin-engine hero showcase A, The Last Light (showcase/the-last-light)",
    licence: "generated through the OriginGame gateway for origin-engine; Meshy output from gpt-image concepts",
    operation_id: task.operation_id,
    gateway_task_id: task.task_id,
    meshy: { operation: task.operation, input: task.input ? ledger.tasks[task.input]!.operation_id : undefined, action_ids: task.action_ids },
    model: { key: "model.glb", bytes: model.length, sha256: sha256(model) },
    concepts: concepts.map(({ body: _body, ...concept }) => concept),
    shipped: "showcase/the-last-light/tools/pack.ts packs the models for the game (pinned in showcase/the-last-light/og-assets.json)",
  };
  for (const concept of concepts) await keep(concept.key, concept.body);
  await keep("model.glb", model);
  task.archive = await keep("task.json", Buffer.from(JSON.stringify(receipt, null, 1) + "\n"));
}
writeFileSync(ledgerFile, JSON.stringify(ledger, null, 2) + "\n");
