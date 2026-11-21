// Holds types/ui.d.ts, what a game's check reads for @origingame/engine/ui, to
// this module: the engine's typecheck fails when a declared export is missing
// or no longer fits, or when a runtime export a game may use is undeclared.

import type * as Declared from "../../types/ui.d.ts";
import * as ui from "./index.ts";

type Values<M> = { [K in keyof M as M[K] extends (...args: never[]) => unknown ? K : never]: M[K] };

export const declared: Values<typeof Declared> = ui;

// The engine's own (the runtime's flush, tests) are not for games.
type Undeclared = Exclude<keyof Values<typeof ui>, keyof typeof Declared | "renderers" | "rendererFor">;
export const undeclared: [Undeclared] extends [never] ? true : Undeclared = true;
