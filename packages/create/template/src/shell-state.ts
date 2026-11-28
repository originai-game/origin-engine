// Navigation only. Bind gameplay systems to state: "session", value: "playing".
// UI callbacks record intents; only this system changes the session/world.
import { og, defineEvent, defineSystem, str, type World, type Result } from "@origingame/engine";
import { shellConfig } from "./shell-config.ts";

export const ShellAction = defineEvent("ShellAction", { action: str(32), value: str(256) });
export const shellStates = [{ name: "session", values: ["title", "menu", "loading", "playing", "paused", "settings", "controls", "save", "load", "credits"], initial: "title" }];
const defaults = () => ({ volume: 1, textScale: 1, reducedMotion: false, language: shellConfig.lang[0]!, bindings: "" });
// Seconds the loading screen stays at least, so it reads instead of flashing.
const LOADING_HOLD = 0.8;
// The credits' roll, CSS px a second; a Faster press jumps a second ahead.
const CREDITS_SPEED = 40;

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
    // The menus' key art and logo from the art slots (SHELL_ART; 0: none, the pack's fallback).
    art: scope.writable("art", { keyArt: 0, logo: 0 }),
    // The question the Confirm dialog asks ("" closed, "quit-to-menu").
    confirm: scope.writable("confirm", ""),
    // The loading screen: seconds shown and the bundle's progress, 0 to 1.
    loading: scope.writable("loading", { shown: 0, progress: 0 }),
    // The controls screen's cell waiting for an input ("action:key" or "action:pad").
    listening: scope.writable("listening", ""),
    // How far the credits have rolled, CSS px.
    credits: scope.writable("credits", 0),
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
 * The shell's art slots: each a texture declared at its content path, which the
 * shell loads at start. Replace an image by replacing its file under assets/,
 * or point the declaration at another file. A slot with no declaration falls
 * back to the UI pack's designed backdrop (key art) or the game's name set as
 * the pack's wordmark (logo). `brief` is the art kind packages/knowledge's
 * artBrief takes for it.
 *
 *   keyArt  content/ui/key-art.ts  assets/ui/key-art.png  16:9 1920x1080 (9:16 1080x1920 for a phone)
 *   logo    content/ui/logo.ts     assets/ui/logo.png     transparent, about 1200x500
 */
export const SHELL_ART = {
  keyArt: { id: "base/ui/key-art", brief: "keyArt" },
  logo: { id: "base/ui/logo", brief: "logo" },
} as const;

/** A slot's texture, or 0 when the project declares none (literal IDs, so the build keeps exactly these). */
function shellArt(world: World) {
  const keyArt = og.load.texture(world, "base/ui/key-art"), logo = og.load.texture(world, "base/ui/logo");
  return { keyArt: keyArt.ok ? (keyArt.value as unknown as number) : 0, logo: logo.ok ? (logo.value as unknown as number) : 0 };
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
  if (value.bindings !== "") og.input.loadBindings(world, value.bindings);
  else must(og.input.resetBindings(world));
  must(world.resource(og.Audio)).master = value.volume;
  must(og.ui.configure(world, { textScale: value.textScale, reducedMotion: value.reducedMotion }));
  og.i18n.setLocale(world, value.language);
  must(shellStores(world).settings.set(value));
}

function enterLoading(world: World) {
  must(shellStores(world).loading.set({ shown: 0, progress: 0 }));
  navigate(world, "loading");
}

export function startShell(world: World) {
  // The look/UI pack is applied by the project before start; do not replace it here.
  // The scheme follows the look's UI pack (light packs keep their light art).
  must(og.ui.configure(world, { unitScale: shellConfig.platform.startsWith("phone-") || shellConfig.input === "touch" ? 1 : 0 }));
  og.i18n.setLocale(world, shellConfig.lang[0]!);
  must(og.input.defineAction(world, { name: "shell.pause", kind: "button", actionMap: "shell", locked: true,
    bindings: shellConfig.input === "gamepad" ? [{ control: "pad.start" }, { control: "Escape" }] : [{ control: "Escape" }, { control: "pad.start" }] }));
  must(shellStores(world).art.set(shellArt(world)));
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
    enterLoading(world);
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
  if (view() === "loading") {
    // The game's own bundle: a web build fetches it; natively it is ready at once.
    const bundle = og.load.defaultBundle(world);
    const progress = bundle.ok ? og.load.progress(world, bundle.value) : null;
    const shown = get(s.loading).shown + time.dt;
    const fraction = progress?.ok ? progress.value.fraction : 1;
    must(s.loading.set({ shown, progress: fraction }));
    if (fraction >= 1 && shown >= LOADING_HOLD) navigate(world, "playing");
  }
  if (view() === "credits") must(s.credits.set(get(s.credits) + CREDITS_SPEED * time.dt));
  if (get(s.listening) !== "") {
    const capture = og.input.capture(world);
    if (capture.done) {
      const applied = og.input.applyCapture(world, "swap");
      if (!applied.ok) must(s.notice.set("shell.failed"));
      must(s.listening.set(""));
      applySettings(world, { ...get(s.settings), bindings: og.input.saveBindings(world) });
    } else if (capture.cancelled) must(s.listening.set(""));
  }
  if (view() === "playing" && og.input.isActionJustPressed(world, "shell.pause")) navigate(world, "paused");
  for (const event of world.events(ShellAction)) {
    const { action, value } = event;
    if (get(s.pending).request) continue;
    must(s.notice.set(""));
    if (action === "menu") navigate(world, "menu");
    else if (action === "new") {
      // Taken after the game's setup: New game resets the world, not the player's settings.
      must(world.restore(get(shellReset(world))));
      enterLoading(world);
    } else if (action === "pause") navigate(world, "paused");
    else if (action === "ask") must(s.confirm.set(value));
    else if (action === "cancel") must(s.confirm.set(""));
    else if (action === "confirm") {
      const asked = get(s.confirm);
      must(s.confirm.set(""));
      if (asked === "quit-to-menu") navigate(world, "menu");
    }
    else if (action === "credits") {
      must(s.credits.set(0));
      must(s.back.set(view()));
      navigate(world, "credits");
    }
    else if (action === "controls") {
      must(s.back.set(view()));
      navigate(world, "controls");
    }
    else if (action === "rebind") {
      const [name, device] = value.split(":") as [string, string];
      const info = og.input.actions(world).find((a) => a.name === name);
      if (info) {
        // The slot of the device's first binding; past the last adds one.
        const pad = (b: { control?: string }) => (b.control ?? "").startsWith("pad.");
        const at = info.bindings.findIndex((b) => (device === "pad") === pad(b));
        const started = og.input.startCapture(world, { action: name, slot: at < 0 ? info.bindings.length : at, accept: device === "pad" ? "gamepad" : "keyboardMouse" });
        if (started.ok) must(s.listening.set(value));
        else must(s.notice.set("shell.failed"));
      }
    }
    else if (action === "reset-bindings") applySettings(world, { ...get(s.settings), bindings: "" });
    else if (action === "resume") navigate(world, "playing");
    else if (action === "back") {
      if (view() === "controls" && get(s.listening) !== "") {
        og.input.cancelCapture(world);
        must(s.listening.set(""));
        continue;
      }
      if (view() === "settings" || view() === "controls") request("settings", og.platform.saveSettings(world, get(s.settings)));
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
    else if (action === "faster") must(s.credits.set(get(s.credits) + CREDITS_SPEED));
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
