// Scratch: fx-showcase's test-reference export (dev, as reference.ts builds it) from the webboot worktree, with one `pipelines` value.
//   bun export-cmp.ts <sync|async|prewarm> <out>
import { join } from "node:path";
import { settingsOf, webBuild } from "../../../../origin-engine-webboot/packages/engine/src/node/web-build.ts";
import { cookProject } from "../../../../origin-engine-webboot/packages/engine/tests/support/og-bin.ts";

const repo = "C:/Users/win/src/origin-engine-webboot";
const project = join(repo, "templates", "fx-showcase");
const value = process.argv[2] as "auto" | "sync" | "async" | "prewarm";
cookProject(project, repo);
const s = settingsOf(project);
const built = await webBuild(project, join(project, "src", "main.ts"), process.argv[3]!, false, { ...s, publish: { ...s.publish, pipelines: value } });
console.log(value, JSON.stringify(built).slice(0, 300));
