// Downloads every file of one asset-catalogue group (Poly Haven and the
// other curated sources behind the gateway) into a directory, and prints
// each file's name, size and sha256 for README.md's table.
//
//   bun tools/catalogue.ts "<search words>" <group id> <out dir> [source id]
import { createHash } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { call, data } from "./gateway.ts";

const [query, id, out, source] = process.argv.slice(2);
if (!query || !id || !out) throw new Error('usage: bun tools/catalogue.ts "<search words>" <group id> <out dir> [source id]');
const found = data(await call("assets_search", { query, limit: 20, ...(source ? { filters: { sourceId: source } } : {}) }));
const groups = (Array.isArray(found) ? found : (found.items ?? found.results ?? found.groups)) as any[];
const group = groups.find(g => g.id === id);
if (!group) throw new Error(`${id} is not among the results for "${query}"`);
console.log(`${group.title}: ${group.source.name}, ${group.source.license} (${group.source.licenseUrl}), release ${group.releaseId}`);
mkdirSync(out, { recursive: true });
for (const member of group.bundle.members) {
  const grant = data(await call("assets_download", { releaseId: group.releaseId, catalogDigest: group.catalogDigest, fileId: member.fileId }));
  const url = grant.url ?? grant.downloadUrl ?? grant.download_url;
  const name = String(grant.basename ?? member.fileId);
  const bytes = Buffer.from(await (await fetch(url)).arrayBuffer());
  const sha = createHash("sha256").update(bytes).digest("hex");
  const expected = grant.sha256 ?? grant.integrity?.sha256;
  if (expected && expected !== sha) throw new Error(`${name}: sha256 ${sha} is not the catalogue's ${expected}`);
  writeFileSync(join(out, name), bytes);
  console.log(`  ${name} ${bytes.length} bytes sha256 ${sha} (${member.role})${expected ? "" : ` [no catalogue hash; fields: ${Object.keys(grant).join(",")}]`}`);
}
