import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { photoMode } from "./src/photo.ts";
export type * from "./types.ts";
export const photo = registerUiExtension("photo", { photoMode });
