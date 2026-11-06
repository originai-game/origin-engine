import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { idleGenerator, prestigePanel } from "./src/idle.ts";
export type * from "./types.ts";
export const idle = registerUiExtension("idle", { idleGenerator, prestigePanel });
