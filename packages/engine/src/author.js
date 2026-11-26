// Cook installs this module in its closed realm, without importing a host.
export { procgen } from "@origingame/engine/generate";

const read = globalThis.__authorInput;

/** Inputs are captured before evaluation, never opened by the running script. */
export const input = Object.freeze({
  bytes(path) { return read(path, false); },
  text(path) { return read(path, true); },
});
