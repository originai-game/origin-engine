import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { matchStats, goalTimeline, playerPlate, substitution } from "./src/sports.ts";
import { scorebug } from "./src/scorebug.ts";
export type * from "./types.ts";
export const sports = registerUiExtension("sports", { scorebug, matchStats, goalTimeline, playerPlate, substitution });
