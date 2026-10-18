// A static server for an exported game (og build's dist/) as a host serves
// it: the build's own `_headers` applied (COOP/COEP/CORP when `isolate`),
// content types, and two test-only knobs: `throttle` streams the wasm and
// the cooked files slowly so the boot's splash and progress can be seen,
// and `told` injects `window.__OG_STAGE_HOST__` (what the Stage proxy does)
// into index.html. Also serves `extra` pages by path and a `/result` sink.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

declare module "node:path" {
  export function normalize(path: string): string;
}
import { normalize } from "node:path";

declare const Bun: {
  serve(o: { port: number; hostname?: string; fetch(req: Request): Response | Promise<Response> }): { port: number; stop(): void };
  file(path: string): Blob & { exists(): Promise<boolean> };
};

export const TYPES: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript",
  ".wasm": "application/wasm",
  ".json": "application/json",
  ".webmanifest": "application/manifest+json",
  ".png": "image/png",
  ".ogt": "text/plain; charset=utf-8",
};

export interface ServeOptions {
  /** Mounted at /game/. */
  readonly dist: string;
  /** Apply the build's COOP/COEP (its `_headers`); false serves without isolation. */
  readonly isolate: boolean;
  /** Bytes per second for the wasm and cooked/ (0: as fast as possible). */
  readonly throttle?: number;
  /** The host origin told to the page (`window.__OG_STAGE_HOST__`), or null. */
  readonly told?: () => string | null;
  /** Other pages by path (with the same isolation headers). */
  readonly extra?: Record<string, string>;
  readonly onResult?: (body: unknown) => void;
}

/** The `_headers` of a build, as path pattern → headers (Netlify/Cloudflare format, `*` suffixes). */
export function readHeaders(dist: string): Array<{ pattern: string; headers: Record<string, string> }> {
  const file = join(dist, "_headers");
  if (!existsSync(file)) return [];
  const rules: Array<{ pattern: string; headers: Record<string, string> }> = [];
  for (const line of readFileSync(file, "utf8").split("\n")) {
    if (!line.trim()) continue;
    if (!line.startsWith(" ")) rules.push({ pattern: line.trim(), headers: {} });
    else {
      const [k, ...v] = line.trim().split(":");
      rules[rules.length - 1]!.headers[k!.trim()] = v.join(":").trim();
    }
  }
  return rules;
}

const matches = (pattern: string, path: string) =>
  pattern === path || (pattern.endsWith("*") && path.startsWith(pattern.slice(0, -1))) || (pattern.startsWith("/*.") && path.endsWith(pattern.slice(2)));

export function serve(o: ServeOptions): { port: number; stop(): void } {
  const rules = readHeaders(o.dist);
  return Bun.serve({
    port: 0,
    hostname: "127.0.0.1",
    async fetch(req) {
      const url = new URL(req.url);
      const headers: Record<string, string> = { "cache-control": "no-store" };
      const isolation = o.isolate
        ? { "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp", "cross-origin-resource-policy": "cross-origin" }
        : {};
      if (url.pathname === "/result") {
        o.onResult?.(await req.json());
        return new Response("ok", { headers });
      }
      const page = o.extra?.[url.pathname];
      if (page !== undefined) return new Response(page, { headers: { ...headers, ...isolation, "content-type": TYPES[".html"]! } });
      if (!url.pathname.startsWith("/game/")) return new Response("not found", { status: 404, headers });
      let rel = normalize(decodeURIComponent(url.pathname.slice("/game".length)));
      if (rel.endsWith("/")) rel += "index.html";
      const path = join(o.dist, rel);
      if (!path.startsWith(o.dist) || !(await Bun.file(path).exists())) return new Response("not found", { status: 404, headers });
      for (const r of rules) if (matches(r.pattern, rel)) Object.assign(headers, Object.fromEntries(Object.entries(r.headers).map(([k, v]) => [k.toLowerCase(), v])));
      if (o.isolate) Object.assign(headers, isolation);
      else {
        delete headers["cross-origin-opener-policy"];
        delete headers["cross-origin-embedder-policy"];
      }
      headers["cache-control"] = "no-store";
      const ext = rel.slice(rel.lastIndexOf("."));
      headers["content-type"] = TYPES[ext] ?? "application/octet-stream";
      if (rel === "/index.html" && o.told) {
        const told = o.told();
        let html = readFileSync(path, "utf8");
        if (told !== null) html = html.replace("<head>", `<head><script>window.__OG_STAGE_HOST__=${JSON.stringify(told)}</script>`);
        return new Response(html, { headers });
      }
      const slow = o.throttle && (ext === ".wasm" || rel.startsWith("/cooked/"));
      if (!slow) return new Response(Bun.file(path), { headers });
      const bytes = new Uint8Array(readFileSync(path));
      const chunk = 32 * 1024;
      const delay = (chunk / o.throttle!) * 1000;
      let at = 0;
      const body = new ReadableStream<Uint8Array>({
        async pull(c) {
          if (at >= bytes.length) return c.close();
          await new Promise((r) => setTimeout(r, delay));
          c.enqueue(bytes.subarray(at, at + chunk));
          at += chunk;
        },
      });
      headers["content-length"] = String(bytes.length);
      return new Response(body, { headers });
    },
  });
}
