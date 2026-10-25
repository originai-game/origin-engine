// Bun preload for the real og run --commands process. No engine source edits.
import { writeFileSync } from "node:fs";
import { loadNativeHost } from "../../../../packages/engine/src/node/index.ts";
const host = loadNativeHost();
const frame = host.abi.og_engine_frame;
const samples = [];
host.abi.og_engine_frame = (id, elapsed) => {
  const start = performance.now();
  const raw = frame(id, elapsed);
  const wallMs = performance.now() - start;
  const answer = JSON.parse(raw);
  samples.push({ wallMs, ...answer });
  return raw;
};
process.on("exit", () => writeFileSync(process.env.R4_SCREENSHOT_TIMES, JSON.stringify(samples, null, 2)));
