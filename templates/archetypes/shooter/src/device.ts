// This device's own, never in the world: the player's settings (the
// volumes, the language, reduced motion, the look's sensitivity on touch),
// loaded from og.platform's settings on the first ticks and saved by the
// settings' Done. They change presentation and this device's input alone
// (the buses in og.Audio, each peer's own; the strings' locale; the
// camera's shake and the screens' dissolve; how far a drag turns the
// view, which reaches the world only as this seat's input), so in a room
// each player keeps their own and no peer's settings enter the world or
// its hash. src/ui.ts loads, applies and saves them.

/** The player's settings on this device, its settings request, and how often Done saved them. */
export const prefs = { master: 1, music: 0.6, effects: 1, language: 0, reducedMotion: false, sensitivity: 1, loadRequest: 0, saves: 0 };
