import { existsSync, realpathSync } from "node:fs";
import { isBuiltin } from "node:module";
import { dirname, extname, isAbsolute, relative, resolve } from "node:path";

declare const Bun: {
  Transpiler: new (options: { loader: "ts" | "tsx" }) => {
    scanImports(source: string): Array<{ path: string }>;
  };
  file(path: string): { text(): Promise<string> };
  resolveSync(specifier: string, parent: string): string;
};

declare module "node:module" {
  export function isBuiltin(specifier: string): boolean;
}
declare module "node:path" {
  export function extname(path: string): string;
}

/** Source imports, including package self-imports, before external filtering. */
export class ImportGraph {
  private readonly edges = new Map<string, Promise<readonly string[]>>();
  private readonly parsers = {
    ts: new Bun.Transpiler({ loader: "ts" }),
    tsx: new Bun.Transpiler({ loader: "tsx" }),
  };

  private readonly root: string;

  constructor(root: string) {
    this.root = realpathSync(root);
  }

  async inputs(entry: string): Promise<Set<string>> {
    const visited = new Set<string>();
    const pending = [realpathSync(resolve(this.root, entry))];
    while (pending.length) {
      const file = pending.pop();
      if (file === undefined || visited.has(file)) continue;
      visited.add(file);
      let edges = this.edges.get(file);
      if (!edges) {
        edges = this.imports(file);
        this.edges.set(file, edges);
      }
      pending.push(...await edges);
    }
    return new Set([...visited].map(file => relative(this.root, file)));
  }

  private async imports(file: string): Promise<readonly string[]> {
    const dependencies = new Set<string>();
    // Resolver configuration is an input even when this source has no imports.
    for (let dir = dirname(file); this.local(dir); dir = dirname(dir)) {
      for (const name of ["package.json", "tsconfig.json", "bun.lock", "bun.lockb", "bunfig.toml"]) {
        const config = resolve(dir, name);
        if (existsSync(config) && config !== file) dependencies.add(config);
      }
      if (dir === this.root) break;
    }
    const extension = extname(file);
    if (![".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".mts", ".cts"].includes(extension)) {
      return [...dependencies];
    }
    const parser = extension === ".tsx" || extension === ".jsx" ? this.parsers.tsx : this.parsers.ts;
    for (const imported of parser.scanImports(await Bun.file(file).text())) {
      if (imported.path === "bun" || /^(?:node:)?(?:fs(?:\/promises)?|child_process|worker_threads)$/.test(imported.path)) {
        throw new Error(`runtime dependencies of ${file} require execution`);
      }
      if (isBuiltin(imported.path) || imported.path === "bun" || imported.path.startsWith("bun:")) continue;
      // Resolving first is essential: @origingame/engine is usually this
      // package's own source, not an installed third-party dependency.
      const resolved = Bun.resolveSync(imported.path, dirname(file));
      if (!isAbsolute(resolved)) continue;
      const dependency = realpathSync(resolved);
      if (this.local(dependency) && !relative(this.root, dependency).split("/").includes("node_modules")) {
        dependencies.add(dependency);
      }
    }
    return [...dependencies];
  }

  private local(file: string): boolean {
    const path = relative(this.root, file);
    return path !== ".." && !path.startsWith("../") && !isAbsolute(path);
  }
}
