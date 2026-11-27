// The Web boot's delivery DAG: shared dependencies arrive once, unrelated
// regions stay lazy, and conflicting/cyclic manifests cannot start a game.
import { expect, test } from "bun:test";
import { preloadBundle } from "../src/web/boot.ts";
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
