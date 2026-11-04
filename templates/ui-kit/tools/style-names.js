import { readdirSync } from "node:fs";

// Bun evaluates this at build time on both hosts; no filesystem in game code.
export function styleNames() {
  return ["default", ...readdirSync(new URL("../assets/looks/", import.meta.url))
    .filter((name) => name.startsWith("kit-") && name.endsWith(".ogt"))
    .map((name) => name.slice(4, -4))
    .filter((name) => name !== "default")
    .sort()];
}
