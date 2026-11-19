// Keeps this project's generated sources in the private bucket
// origin-generated-assets (docs/ci.md §The model bucket; its README.md has
// the layout): each Meshy task of tools/assets.json as
// meshy/model/<operation id>/model.glb with the reference images it was
// made from and a task.json receipt (prompt, ids, sizes and checksums).
// The bucket is write-once: what is already there is left alone.
//
//   bun tools/archive.ts [name...]
//
// Needs OG_SHOWCASE_S3_ACCESS_KEY and OG_SHOWCASE_S3_SECRET_KEY.
import { createHash, createHmac } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const ENDPOINT = "s3.bhs.io.cloud.ovh.net", REGION = "bhs", BUCKET = "origin-generated-assets";
const access = process.env.OG_SHOWCASE_S3_ACCESS_KEY, secret = process.env.OG_SHOWCASE_S3_SECRET_KEY;
if (!access || !secret) throw new Error("set OG_SHOWCASE_S3_ACCESS_KEY and OG_SHOWCASE_S3_SECRET_KEY");

const sha256 = (data: Buffer | string) => createHash("sha256").update(data).digest("hex");
const hmac = (key: Buffer | string, data: string) => createHmac("sha256", key).update(data).digest();

/** One signed S3 request (signature version 4); a put is encrypted at rest, as the bucket asks. */
async function s3(method: "HEAD" | "PUT", key: string, body = Buffer.alloc(0), type = "application/octet-stream"): Promise<Response> {
  const path = `/${BUCKET}/${key.split("/").map(encodeURIComponent).join("/")}`;
  const stamp = new Date().toISOString().replace(/[-:]|\.\d{3}/g, ""), day = stamp.slice(0, 8);
  const headers: Record<string, string> = { host: ENDPOINT, "x-amz-content-sha256": sha256(body), "x-amz-date": stamp };
  if (method === "PUT") { headers["content-type"] = type; headers["x-amz-server-side-encryption"] = "AES256"; }
  const names = Object.keys(headers).sort();
  const request = [method, path, "", ...names.map(n => `${n}:${headers[n]}`), "", names.join(";"), headers["x-amz-content-sha256"]].join("\n");
  const scope = `${day}/${REGION}/s3/aws4_request`;
  const signing = hmac(hmac(hmac(hmac(`AWS4${secret}`, day), REGION), "s3"), "aws4_request");
  const signature = hmac(signing, ["AWS4-HMAC-SHA256", stamp, scope, sha256(request)].join("\n")).toString("hex");
  const { host: _host, ...sent } = headers;
  return fetch(`https://${ENDPOINT}${path}`, {
    method, body: method === "PUT" ? body : undefined,
    headers: { ...sent, authorization: `AWS4-HMAC-SHA256 Credential=${access}/${scope}, SignedHeaders=${names.join(";")}, Signature=${signature}` },
  });
}

async function keep(key: string, body: Buffer, type: string): Promise<void> {
  const there = await s3("HEAD", key);
  if (there.status === 200) { console.log(`  ${key}: already kept (${there.headers.get("content-length")} bytes)`); return; }
  if (there.status !== 404) throw new Error(`${key}: HEAD answered ${there.status}`);
  const put = await s3("PUT", key, body, type);
  if (!put.ok) throw new Error(`${key}: PUT answered ${put.status} ${(await put.text()).slice(0, 300)}`);
  console.log(`  ${key}: kept ${body.length} bytes sha256 ${sha256(body)}`);
}

interface Task {
  operation: string; operation_id: string; task_id: string; out: string;
  reference?: string; reference_media?: string[]; image_model?: string; input?: string; action_ids?: number[];
}
const root = join(import.meta.dir, "..");
const ledger = JSON.parse(readFileSync(join(root, "tools/assets.json"), "utf8")) as { tasks: Record<string, Task> };
const names = process.argv.slice(2);
for (const [name, task] of Object.entries(ledger.tasks)) {
  if (names.length && !names.includes(name)) continue;
  const prefix = `meshy/model/${task.operation_id}`;
  console.log(`${name}: ${prefix}`);
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
  for (const concept of concepts) await keep(`${prefix}/${concept.key}`, concept.body, "image/png");
  await keep(`${prefix}/model.glb`, model, "model/gltf-binary");
  await keep(`${prefix}/task.json`, Buffer.from(JSON.stringify(receipt, null, 1) + "\n"), "application/json");
}
