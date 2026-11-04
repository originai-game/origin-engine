import { existsSync, readdirSync } from "node:fs";

// Bun evaluates this at build time on both hosts; no filesystem in game code.
export function stylePaths() {
  const paths = Object.fromEntries(readdirSync(new URL("../assets/looks/", import.meta.url))
    .filter((name) => name.startsWith("kit-") && name.endsWith(".ogt"))
    .map((name) => name.slice(4, -4))
    .filter((name) => name !== "default")
    .map((name) => [name, `@styles/${name}/style.ogt`]));
  const installed = new URL("../assets/styles/", import.meta.url);
  if (existsSync(installed)) {
    for (const name of readdirSync(installed)) {
      if (existsSync(new URL(`${name}/style.ogt`, installed))) paths[name] = `styles/${name}/style.ogt`;
    }
  }
  return paths;
}

export function styleNames() {
  return ["default", ...Object.keys(stylePaths()).sort()];
}
