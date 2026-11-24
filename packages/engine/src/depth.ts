// The `custom` member of `debug` (IDL): a publish build's debug view is none.

import type { Custom } from "./generated/calls.ts";
import { abi } from "./runtime/host.ts";
import { parse } from "./runtime/result.ts";

export const debugCustom: Custom["debug"] = {
  // A publish build answers every debug member with a refusal: the view is none there.
  view: (world) => {
    const v = parse<unknown>(abi().og_debug_view(world.id));
    return typeof v === "string" ? v : "none";
  },
};
