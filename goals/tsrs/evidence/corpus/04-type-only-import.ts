// Corpus 04-type-only-import: verbatimModuleSyntax, a type imported as a value
import { World } from "@origingame/engine";
export function f(w: World): number { return w.tick; }
