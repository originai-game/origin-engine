// Declaration-only inspection for og style check: no host, world, tick or GPU.
import { writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { declaredGameLook } from "../runtime.ts";

const [bundle, output] = process.argv.slice(2);
if (!bundle || !output) throw new Error("usage: game-look.ts <bundle> <output.json>");
const game = await import(pathToFileURL(bundle).href);
if (!game.default || typeof game.default !== "object") throw new Error("the entry must export default og.game(...)");
writeFileSync(output, JSON.stringify(declaredGameLook(game.default)));
