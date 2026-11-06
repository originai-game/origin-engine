import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { storeFront } from "./src/store.ts";
import { battlePass } from "./src/pass.ts";
export type * from "./types.ts";
export const liveops = registerUiExtension("liveops", { storeFront, battlePass });
