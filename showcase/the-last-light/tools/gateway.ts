// Calls one tool of the OriginGame asset gateway (MCP over HTTP) with
// OG_PLATFORM_DEVELOPMENT_KEY, for fetching and generating this project's
// art (README.md records what each call produced). Nothing here is cooked.
//
//   bun tools/gateway.ts <tool> '<json arguments>' [--save <file>] [--timeout <seconds>]
//
// Prints the result with inline image bytes replaced by their size; --save
// writes the first image the result carries (base64 content or media_url).
import { writeFileSync } from "node:fs";

const ENDPOINT = "https://mcp.origingame.dev/mcp";

export async function call(tool: string, args: unknown, timeoutSeconds = 600): Promise<any> {
  const key = process.env.OG_PLATFORM_DEVELOPMENT_KEY;
  if (!key) throw new Error("OG_PLATFORM_DEVELOPMENT_KEY is not set");
  const response = await fetch(ENDPOINT, {
    method: "POST",
    headers: { Authorization: `Bearer ${key}`, "Content-Type": "application/json", Accept: "application/json, text/event-stream" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/call", params: { name: tool, arguments: args } }),
    signal: AbortSignal.timeout(timeoutSeconds * 1000),
  });
  const body = await response.text();
  if (!response.ok) throw new Error(`${tool}: HTTP ${response.status} ${body.slice(0, 400)}`);
  const lines = body.split("\n").filter(line => line.startsWith("data: "));
  const message = JSON.parse(lines.length ? lines[lines.length - 1]!.slice(6) : body);
  if (message.error) throw new Error(`${tool}: ${JSON.stringify(message.error)}`);
  return message.result;
}

/** A result's data: its structured content, else its first text block parsed. */
export function data(result: any): any {
  return result.structuredContent ?? JSON.parse(result.content[0].text);
}

if (import.meta.main) {
  const [tool, json = "{}", ...rest] = process.argv.slice(2);
  if (!tool) throw new Error("usage: bun tools/gateway.ts <tool> '<json>' [--save <file>] [--timeout <seconds>]");
  const flag = (name: string) => (rest.includes(name) ? rest[rest.indexOf(name) + 1] : undefined);
  const result = tool === "tools/list" ? null : await call(tool, JSON.parse(json), Number(flag("--timeout") ?? 600));
  const save = flag("--save");
  if (save && result) {
    const image = (result.content ?? []).find((c: any) => c.type === "image");
    const url = result.structuredContent?.media_url ?? result.structuredContent?.url;
    if (image) writeFileSync(save, Buffer.from(image.data, "base64"));
    else if (url) writeFileSync(save, Buffer.from(await (await fetch(url)).arrayBuffer()));
    else throw new Error("the result carries no image");
  }
  console.log(JSON.stringify(result, (k, v) => (k === "data" && typeof v === "string" && v.length > 400 ? `<${v.length} base64 chars>` : v), 1));
}
