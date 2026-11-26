// Cook installs this module in its closed realm, without importing a host.
export { procgen } from "@origingame/engine/generate";

const read = globalThis.__authorInput;

/** Explicit streams use the same Rust algorithm as the simulation. */
export const rng = globalThis.__authorRng;

/** Inputs are captured before evaluation, never opened by the running script. */
export const input = Object.freeze({
  bytes(path) { return read(path, false); },
  text(path) { return read(path, true); },
});

// Kind names come from the Rust schema registry. A descriptor has no side
// effects: constructing one never evaluates or cooks another content entry.
export const file = Object.freeze(Object.fromEntries(
  globalThis.__authorKinds.map(kind => [kind, value => ({ $ogFile: kind, value })]),
));

/** Identity only. Existence, access and target kinds belong to the content graph. */
export function ref(id) {
  if (typeof id !== "string" || id.length === 0) {
    throw new TypeError("ref requires a nonempty content ID, not a runtime handle");
  }
  return Object.freeze({ $ogRef: id });
}
