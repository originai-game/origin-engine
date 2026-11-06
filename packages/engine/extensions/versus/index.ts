import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { lobby, characterSelect, versus as versusScreen, draft } from "./src/roster.ts";
import { chat } from "./src/chat.ts";
export type * from "./types.ts";
export const versus = registerUiExtension("versus", { lobby, characterSelect, versus: versusScreen, draft, chat });
