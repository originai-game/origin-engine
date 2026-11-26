import { og } from "./index.ts";

/** Pure mechanisms shared by game code and deterministic source generators.
 * Cook supplies this module directly, without a world or host handles. */
export const procgen = og.procgen;
