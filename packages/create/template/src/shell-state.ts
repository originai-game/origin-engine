// Navigation only. Bind gameplay systems to state: "session", value: "playing".
// UI callbacks record intents; only this system changes the session/world.
import { og, defineEvent, defineSystem, str, type World, type Result } from "@origingame/engine";
import { shellConfig } from "./shell-config.ts";

export const ShellAction = defineEvent("ShellAction", { action: str(32), value: str(256) });
export const shellStates = [{ name: "session", values: ["title", "menu", "playing", "paused", "settings", "save", "load"], initial: "title" }];
const defaults = () => ({ volume: 1, textScale: 1, reducedMotion: false, language: shellConfig.lang[0]! });

export function shellStores(world: World) {
  const scope = og.ui.scope(world, "shell");
  return {
    scope,
    view: scope.writable("view", "title"),
    back: scope.writable("back", "menu"),
    saved: scope.writable("saved", false),
    settings: scope.writable("settings", defaults()),
    notice: scope.writable("notice", ""),
    pending: scope.writable("pending", { request: 0, kind: "" }),
    preferences: scope.writable("preferences", 0),
    peek: scope.writable("peek", 0),
    // Seconds in play: saved with the world, shown on pause and the save slot.
    played: scope.writable("played", 0),
    // The saved slot's meta, for Continue and the slot row.
    slot: scope.writable("slot", { played: 0 }),
    // The title's and main menu's key art and logo (0: none).
    art: scope.writable("art", { keyArt: 0, logo: 0 }),
  };
}

// Declared AFTER the reset snapshot and registered by GameDesc.ui: restoring
// that snapshot supplies this added store's registered initial value.
export function shellReset(world: World, initial = "") {
  return og.ui.scope(world, "shell-reset").writable("initial", initial);
}

function must<T>(result: Result<T>): T {
  if (!result.ok) throw new Error(result.reason);
  return result.value;
}

/**
 * Give the menus their art, in GameDesc.start before startShell: `keyArt` and
 * `logo` are og.load.texture handles (0: none). Without key art the title and
 * main menu stand on whatever the game draws, its scene included; paused
 * menus stand on the frozen game frame.
 */
export function setShellArt(world: World, art: { keyArt?: number; logo?: number }) {
  const s = shellStores(world);
  must(s.art.set({ ...s.scope.get(s.art), ...art }));
}

export function sendShell(world: World, action: string, value = "") {
  must(og.input.intent(world, ShellAction, { action, value }));
}

function navigate(world: World, view: string) {
  must(world.setState("session", view));
  must(og.time.setScale(world, view === "playing" ? 1 : 0));
  og.input.setActionMaps(world, view === "playing" ? ["shell", "gameplay"] : ["shell", "menu"]);
  const flow = og.flow.current(world);
  if (flow !== null) must(view === "playing" ? og.flow.play(world, flow) : og.flow.pause(world, flow));
  must(shellStores(world).view.set(view));
}

function applySettings(world: World, value: ReturnType<typeof defaults>) {
  must(world.resource(og.Audio)).master = value.volume;
  must(og.ui.configure(world, { textScale: value.textScale, reducedMotion: value.reducedMotion }));
  og.i18n.setLocale(world, value.language);
  must(shellStores(world).settings.set(value));
}

export function startShell(world: World) {
  // The look/UI pack is applied by the project before start; do not replace it here.
  // The scheme follows the look's UI pack (light packs keep their light art).
  must(og.ui.configure(world, { unitScale: shellConfig.platform.startsWith("phone-") || shellConfig.input === "touch" ? 1 : 0 }));
  og.i18n.setLocale(world, shellConfig.lang[0]!);
  must(og.input.defineAction(world, { name: "shell.pause", kind: "button", actionMap: "shell", locked: true,
    bindings: shellConfig.input === "gamepad" ? [{ control: "pad.start" }, { control: "Escape" }] : [{ control: "Escape" }, { control: "pad.start" }] }));
  navigate(world, "title");
  shellReset(world, must(world.snapshot()));
  const s = shellStores(world);
  must(s.preferences.set(og.platform.loadSettings(world)));
  const peek = og.platform.loadSlot(world, { slot: "progress", metaOnly: true });
  if (peek.ok) must(s.peek.set(peek.value));
}

export const Shell = defineSystem("Shell", {
  reads: [ShellAction, og.WorldRestored], writes: [og.Audio],
}, (_q, time, world) => {
  const s = shellStores(world), get = s.scope.get.bind(s.scope);
  const view = () => get(s.view);
  const request = (kind: string, answer: Result<number>) => {
    if (answer.ok) {
      must(s.pending.set({ request: answer.value, kind }));
      must(s.notice.set("shell.busy"));
    } else must(s.notice.set("shell.failed"));
  };
  if (world.events(og.WorldRestored).length) {
    // Requests belong to the host session, not to a world snapshot.
    must(s.pending.set({ request: 0, kind: "" }));
    must(s.preferences.set(og.platform.loadSettings(world)));
    const peek = og.platform.loadSlot(world, { slot: "progress", metaOnly: true });
    if (peek.ok) must(s.peek.set(peek.value));
    applySettings(world, get(s.settings));
    navigate(world, "playing");
  }
  const preferences = get(s.preferences);
  const loadedSettings = preferences ? og.platform.result(world, preferences) : null;
  if (loadedSettings) {
    must(s.preferences.set(0));
    const value = (loadedSettings.data as { value?: Partial<ReturnType<typeof defaults>> } | null)?.value;
    if (loadedSettings.ok && value) applySettings(world, { ...defaults(), ...value });
  }
  const peek = get(s.peek), peeked = peek ? og.platform.result(world, peek) : null;
  if (peeked) {
    must(s.peek.set(0));
    const meta = (peeked.data as { meta?: { played?: number } | null } | null)?.meta;
    must(s.saved.set(peeked.ok && meta != null));
    if (meta) must(s.slot.set({ played: meta.played ?? 0 }));
  }
  const pending = get(s.pending);
  const answer = pending.request ? og.platform.result(world, pending.request) : null;
  if (answer) {
    must(s.pending.set({ request: 0, kind: "" }));
    must(s.notice.set(answer.ok ? "" : "shell.failed"));
    if (answer.ok && pending.kind === "save") {
      must(s.saved.set(true));
      must(s.slot.set({ played: get(s.played) }));
      must(s.notice.set("shell.saved"));
    }
    if (answer.ok && pending.kind === "load") {
      const text = (answer.data as { text?: string } | null)?.text;
      if (text) {
        const restored = world.restore(text);
        if (!restored.ok) must(s.notice.set("shell.failed"));
      } else must(s.notice.set("shell.failed"));
    }
  }
  // Resume is the screen's confirm/back handler. A held Start must not
  // re-enter play when switching action maps gives it another press edge.
  if (view() === "playing") must(s.played.set(get(s.played) + time.scaledDt));
  if (view() === "playing" && og.input.isActionJustPressed(world, "shell.pause")) navigate(world, "paused");
  for (const event of world.events(ShellAction)) {
    const { action, value } = event;
    if (get(s.pending).request) continue;
    must(s.notice.set(""));
    if (action === "menu") navigate(world, "menu");
    else if (action === "new") {
      // Taken after the game's setup: New game resets the world, not the player's settings.
      must(world.restore(get(shellReset(world))));
    } else if (action === "pause") navigate(world, "paused");
    else if (action === "resume") navigate(world, "playing");
    else if (action === "back") {
      if (view() === "settings") request("settings", og.platform.saveSettings(world, get(s.settings)));
      navigate(world, get(s.back));
    }
    else if (action === "settings" || action === "save-menu" || action === "load-menu") {
      must(s.back.set(view()));
      navigate(world, action === "settings" ? "settings" : action === "save-menu" ? "save" : "load");
    } else if (action === "save") {
      const snapshot = world.snapshot();
      if (snapshot.ok) request("save", og.platform.saveSlot(world, { slot: "progress", text: snapshot.value, meta: { name: shellConfig.name, played: get(s.played) } }));
      else must(s.notice.set("shell.failed"));
    } else if (action === "load") request("load", og.platform.loadSlot(world, { slot: "progress" }));
    else if (action === "setting" || action === "defaults") {
      let next = defaults();
      if (action === "setting") {
        const change = JSON.parse(value) as { id: string; value: boolean | number | string };
        next = { ...get(s.settings), [change.id]: change.value };
      }
      applySettings(world, next);
    }
  }
});
