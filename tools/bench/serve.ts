// Static server for the spike pages with cross-origin isolation (COOP/COEP) so the
// wasm build can use SharedArrayBuffer, plus a result sink the pages POST to.
import { join, normalize } from "node:path";
const root = import.meta.dir;
const port = Number(process.env.PORT ?? 4173);
const results: Record<string, unknown>[] = [];
const types: Record<string, string> = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".mjs": "text/javascript", ".wasm": "application/wasm", ".glb": "model/gltf-binary", ".json": "application/json", ".ts": "text/javascript" };
Bun.serve({
  port,
  async fetch(req) {
    const url = new URL(req.url);
    if (url.pathname === "/result" && req.method === "POST") { const r = await req.json(); results.push(r); console.log("result:", JSON.stringify(r)); return new Response("ok"); }
    if (url.pathname === "/results") return Response.json(results);
    let path = normalize(decodeURIComponent(url.pathname)); if (path.endsWith("/")) path += "index.html";
    const file = Bun.file(join(root, path));
    if (!(await file.exists())) return new Response("not found", { status: 404 });
    const ext = path.slice(path.lastIndexOf("."));
    return new Response(file, { headers: { "content-type": types[ext] ?? "application/octet-stream", "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp", "cross-origin-resource-policy": "same-origin", "cache-control": "no-store" } });
  },
});
console.log(`spike server on http://127.0.0.1:${port}/ (COOP/COEP on)`);
