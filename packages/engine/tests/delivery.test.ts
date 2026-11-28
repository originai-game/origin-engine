// The Web boot's delivery DAG: shared dependencies arrive once, unrelated
// regions stay lazy, and conflicting/cyclic manifests cannot start a game.
import { expect, test } from "bun:test";
import { installFetcher, preloadBundle } from "../src/web/boot.ts";
import type { WebExtras } from "../src/web/host.ts";
import { downloadsOf } from "../src/node/web-build.ts";

declare const Bun: {
  serve(options: { port: number; fetch(request: Request): Response }): { port: number; stop(force: boolean): void };
};

test("delivery boot follows pinned dependencies without fetching other regions", async () => {
  type File = { id: string; file: string; type: string; integrity?: string; group?: string };
  const digest = async (text: string) => `sha384-${btoa(String.fromCharCode(...new Uint8Array(await crypto.subtle.digest("SHA-384", new TextEncoder().encode(text)))))}`;
  const shared = JSON.stringify([{ id: "shared", file: "shared.ogb", type: "og.Scene" }]);
  const dependency: File = { id: "shared", file: "delivery/shared/.cook.ogt", type: "og.Bundle", integrity: await digest(shared) };
  const other = JSON.stringify([{ ...dependency, file: "../shared/.cook.ogt" }, { id: "other", file: "other.ogb", type: "og.Scene", group: "other/cells" }]);
  const base = JSON.stringify([dependency, { id: "other", file: "delivery/other/.cook.ogt", type: "og.Bundle", integrity: await digest(other) }, { id: "boot", file: "boot.ogb", type: "og.Scene" }]);
  const files = new Map([
    ["/cooked/.cook.ogt", base],
    ["/cooked/boot.ogb", "boot payload"],
    ["/cooked/delivery/shared/.cook.ogt", shared],
    ["/cooked/delivery/shared/shared.ogb", "shared payload"],
    ["/cooked/delivery/other/.cook.ogt", other],
  ]);
  const requests: string[] = [];
  const server = Bun.serve({ port: 0, fetch(req) {
    const path = new URL(req.url).pathname;
    requests.push(path);
    const file = files.get(path);
    return new Response(file ?? "missing", { status: file === undefined ? 404 : 200 });
  } });
  try {
    const puts = new Map<string, string>();
    const failures: string[] = [];
    const sharedIntegrity = await digest("shared payload");
    const glue = {
      og_web_bundle_files: (manifest: string) => manifest,
      og_web_bundle_integrity: (manifest: string) => JSON.stringify(Object.fromEntries((JSON.parse(manifest) as File[]).map((f) => [f.file, sharedIntegrity]))),
      og_web_put_file: (path: string, bytes: Uint8Array) => { puts.set(new URL(path).pathname, new TextDecoder().decode(bytes)); },
      og_web_fail_file: (_path: string, reason: string) => { failures.push(reason); },
    };
    const root = `http://localhost:${server.port}`;
    const loaded = await preloadBundle(glue, root, () => {});
    expect(failures).toEqual([]);
    expect(loaded).toMatchObject({ files: 2, failed: 0, streamed: 1 });
    expect(puts.get("/cooked/delivery/shared/shared.ogb")).toBe("shared payload");
    expect(requests.filter((p) => p.endsWith("shared/.cook.ogt"))).toHaveLength(1);
    expect(requests.some((p) => p.includes("other.ogb") || p.includes("expansion"))).toBeFalse();
    files.set("/cooked/.cook.ogt", JSON.stringify([dependency, { ...dependency, integrity: await digest("wrong version") }]));
    await expect(preloadBundle(glue, root, () => {})).rejects.toThrow("conflicting delivery manifest pins");
    files.set("/cooked/.cook.ogt", JSON.stringify([{ id: "self", type: "og.Bundle", file: "./.cook.ogt" }]));
    await expect(preloadBundle(glue, root, () => {})).rejects.toThrow("delivery dependency cycle");
    const sizes = downloadsOf([
      { path: "cooked/base.ogb", bytes: 13, compressed: 7, codec: "brotli-5" },
      { path: "cooked/delivery/north/.cook.ogt", bytes: 19, compressed: 11, codec: "brotli-5" },
      { path: "cooked/delivery/north/stream/north/cells.ogb", bytes: 23, compressed: 17, codec: "brotli-5" },
    ], new Set(), new Set(), new Set(["cooked/delivery/north/.cook.ogt", "cooked/delivery/north/stream/north/cells.ogb"]));
    expect(sizes.later.packs).toEqual({ files: 2, bytes: 42, compressed: 28 });
    expect(sizes.boot.single.cooked).toEqual({ files: 1, bytes: 13, compressed: 7 });
  } finally {
    server.stop(true);
  }
});

test("web boot and lazy fonts share body slots and recover Retry-After refusals", async () => {
  const boot = Array.from({ length: 12 }, (_, i) => ({ id: `asset${i}`, file: `asset${i}.ogb`, type: "og.Scene" }));
  const face = "required.ttf.face.ogb";
  const manifest = JSON.stringify([...boot, { id: "required", file: face, type: "og.Font", preload: true }]);
  const seen = new Map<string, number[]>();
  let active = 0, peak = 0, overflow = 0;
  const server = Bun.serve({ port: 0, fetch(req) {
    const path = new URL(req.url).pathname;
    const times = seen.get(path) ?? [];
    times.push(Date.now());
    seen.set(path, times);
    if (path.endsWith(".cook.ogt")) return new Response(manifest);
    if (active >= 6) {
      overflow++;
      return new Response("busy", { status: 429, headers: { "retry-after": "1" } });
    }
    if (times.length === 1 && (path.endsWith(face) || path.includes("lazy"))) {
      return new Response("busy", { status: path.endsWith(face) ? 429 : 503, headers: { "retry-after": "1" } });
    }
    active++;
    peak = Math.max(peak, active);
    // Headers and the first byte arrive immediately; the slot lives until
    // the rest of the body, not merely until fetch() resolves.
    const bytes = new TextEncoder().encode(`payload:${path}`);
    return new Response(new ReadableStream<Uint8Array>({
      async start(controller) {
        controller.enqueue(bytes.slice(0, 1));
        await new Promise((r) => setTimeout(r, 40));
        controller.enqueue(bytes.slice(1));
        controller.close();
        active--;
      },
    }), { headers: { "content-length": String(bytes.length) } });
  } });
  try {
    const root = `http://localhost:${server.port}`;
    const puts = new Map<string, string>();
    const failures: string[] = [];
    let ask!: (path: string) => void;
    let arrived!: () => void;
    const lazy = new Promise<void>((r) => { arrived = r; });
    const glue = {
      og_web_bundle_files: (text: string) => text,
      og_web_bundle_integrity: () => "{}",
      og_web_set_fetcher: (fetch: (path: string) => void) => { ask = fetch; },
      og_web_put_file: (path: string, bytes: Uint8Array) => {
        puts.set(new URL(path).pathname, new TextDecoder().decode(bytes));
        if (puts.has("/cooked/lazy0.face.ogb") && puts.has("/cooked/lazy1.face.ogb")) arrived();
      },
      og_web_fail_file: (_path: string, reason: string) => { failures.push(reason); arrived(); },
    } as unknown as WebExtras;
    installFetcher(glue);
    const loading = preloadBundle(glue, root, () => {});
    for (let i = 0; i < 2; i++) ask(`${root}/cooked/lazy${i}.face.ogb`);
    const [loaded] = await Promise.all([loading, lazy]);
    expect(failures).toEqual([]);
    expect(loaded).toMatchObject({ files: 13, failed: 0 });
    for (const file of [...boot.map((f) => f.file), face, "lazy0.face.ogb", "lazy1.face.ogb"]) {
      expect(puts.get(`/cooked/${file}`)).toBe(`payload:/cooked/${file}`);
    }
    expect(overflow).toBe(0);
    expect(peak).toBeLessThanOrEqual(6);
    for (const file of [face, "lazy0.face.ogb", "lazy1.face.ogb"]) {
      const times = seen.get(`/cooked/${file}`)!;
      expect(times).toHaveLength(2);
      expect(times[1]! - times[0]!).toBeGreaterThanOrEqual(1000);
    }
  } finally {
    server.stop(true);
  }
}, 15_000);
