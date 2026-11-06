import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { flightHud } from "./src/vehicles.ts";
export type * from "./types.ts";
export const flight = registerUiExtension("flight", { flightHud });
