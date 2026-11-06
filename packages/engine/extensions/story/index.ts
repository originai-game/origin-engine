import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { backlog, barks } from "./src/story.ts";
import { cgGallery, routeChart } from "./src/gallery.ts";
export type * from "./types.ts";
export const story = registerUiExtension("story", { backlog, barks, cgGallery, routeChart });
