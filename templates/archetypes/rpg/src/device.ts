// This device's own, never in the world: the player's settings (the
// volumes, the language, reduced motion), loaded from og.platform's
// settings on the first ticks and saved when the settings close. They
// change presentation alone (the buses in og.Audio, each peer's own; the
// strings' locale; the screens' dissolve), so in a room each player keeps
// their own and no peer's settings enter the world or its hash (a platform
// answer is one peer's alone). src/ui.ts loads, applies and saves them.

/** The player's settings on this device, its settings request, and how often they were saved. */
export const prefs = { master: 1, music: 0.7, effects: 1, language: 0, reducedMotion: false, loadRequest: 0, saves: 0 };
