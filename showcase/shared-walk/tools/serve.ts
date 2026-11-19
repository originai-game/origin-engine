// Self-hosting Shared Walk with no platform: serve the built page and a token
// endpoint beside an `og world-server --auth hs256` that holds the same
// secret. The page's session provider (window.__OG_PROVIDER, set inline
// before play.js; a bundled page writes tokenEndpoint("/token") from
// @origingame/engine/provider, which does the same) asks /token: each
// connection POSTs { world, reconnect } here and gets the server's URL and a
// five-minute HS256 token back. A real game signs for the player its own
// login names; this one gives each page a random visitor id.
//
//   og world-server --auth hs256 --secret-file secret --audience shared-walk
//   og build .
//   bun tools/serve.ts --secret-file secret [--port 8080] [--server ws://127.0.0.1:47910/]
import { readFileSync } from "node:fs";
import { join, normalize } from "node:path";

const args = process.argv.slice(2);
const flag = (name: string, fallback?: string): string => {
  const i = args.indexOf(name);
  const v = i >= 0 ? args[i + 1] : fallback;
  if (v === undefined) throw new Error(`${name} is required`);
  return v;
};
const secret = readFileSync(flag("--secret-file")).toString().trim();
if (secret.length < 32) throw new Error("the secret is shorter than 32 bytes");
const port = Number(flag("--port", "8080"));
const server = flag("--server", "ws://127.0.0.1:47910/");
const root = normalize(flag("--root", join(import.meta.dir, "..", "dist")));
const AUDIENCE = "shared-walk";
const NAMES = ["Ash", "Birch", "Cedar", "Elm", "Fir", "Hazel", "Larch", "Oak", "Pine", "Rowan", "Willow", "Yew"];

const key = await crypto.subtle.importKey("raw", new TextEncoder().encode(secret), { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
/** The page's provider, inline: what tokenEndpoint("/token") is. */
const PROVIDER = `<script>window.__OG_PROVIDER = { sessions: { host: "endpoint", connect: (request) =>
  fetch("/token", { method: "POST", credentials: "include", headers: { "content-type": "application/json" }, body: JSON.stringify(request) })
    .then(async (r) => r.ok ? { ok: true, ...(await r.json()) } : { ok: false, error: { code: r.status === 401 || r.status === 403 ? "unauthorized" : "unavailable", message: "the token endpoint answered " + r.status } })
    .catch((e) => ({ ok: false, error: { code: "offline", message: String(e) } })) } };</script>`;

const b64url = (bytes: Uint8Array): string => Buffer.from(bytes).toString("base64url");

/** An HS256 JWT the world server's hs256 verifier takes: aud, sub, iat, exp (≤ 300 s), name. */
async function sign(claims: Record<string, unknown>): Promise<string> {
  const head = b64url(new TextEncoder().encode(JSON.stringify({ alg: "HS256", typ: "JWT" })));
  const body = b64url(new TextEncoder().encode(JSON.stringify(claims)));
  const mac = new Uint8Array(await crypto.subtle.sign("HMAC", key, new TextEncoder().encode(`${head}.${body}`)));
  return `${head}.${body}.${b64url(mac)}`;
}

Bun.serve({
  port,
  async fetch(req) {
    const url = new URL(req.url);
    if (url.pathname === "/token" && req.method === "POST") {
      const ask = (await req.json().catch(() => ({}))) as { world?: unknown };
      // The visitor keeps its id across reconnects through a cookie, so the server resumes its avatar.
      const known = /(?:^|;\s*)visitor=(v[0-9a-f]{12})/.exec(req.headers.get("cookie") ?? "")?.[1];
      const sub = known ?? `v${crypto.randomUUID().replaceAll("-", "").slice(0, 12)}`;
      const name = NAMES[parseInt(sub.slice(1, 5), 16) % NAMES.length]!;
      const now = Math.floor(Date.now() / 1000);
      const world = typeof ask.world === "string" ? ask.world : "main";
      const token = await sign({ aud: AUDIENCE, sub, name, world, iat: now, exp: now + 300 });
      return Response.json({ url: server, token }, { headers: { "set-cookie": `visitor=${sub}; Path=/; SameSite=Strict` } });
    }
    const path = normalize(join(root, url.pathname === "/" ? "index.html" : url.pathname));
    if (!path.startsWith(root)) return new Response("not found", { status: 404 });
    const file = Bun.file(path);
    if (!(await file.exists())) return new Response("not found", { status: 404 });
    if (path.endsWith("index.html")) {
      const html = (await file.text()).replace("<head>", `<head>${PROVIDER}`);
      return new Response(html, { headers: { "content-type": "text/html", "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp" } });
    }
    // A page with SharedArrayBuffer (the engine's Worker ring) needs cross-origin isolation.
    return new Response(file, { headers: { "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp" } });
  },
});
console.log(`shared-walk: http://127.0.0.1:${port}/ (tokens for ${server})`);
