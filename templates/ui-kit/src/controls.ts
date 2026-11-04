// The component library's foundations and controls previews (docs/ui.md
// §11, U6a): text and media, buttons, toggles and choices, ranges, fields
// and pickers, key bindings and the on-screen keyboard, each in its states.
// `demo:<key>:<value>` picks a state the playtests (tests/controls*.og.ts)
// draw: `demo:keyboard:<letters|symbols|numeric|kana|none>` opens the
// on-screen keyboard, `demo:device:<keyboardMouse|xbox|playstation|switch>`
// the key-binding rows' device (and the prompts'), `demo:controls:reset` the
// previews' state.
//
// The previews' state is the preview's, kept here between ticks: it is a
// gallery, never saved.

import { og } from "@origingame/engine";
import { page, panel, phone } from "./components.ts";
import type { InputDevice, KeyboardLayout, PickedColor, Size, Texture, World } from "@origingame/engine";

const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: n });

const initial = () => ({
  music: true,
  vibration: false,
  subtitles: true,
  terms: false,
  news: true,
  difficulty: 1,
  mode: 0,
  view: 0,
  quality: 2,
  volume: 0.7,
  sensitivity: 4,
  balance: 0,
  priceLow: 200,
  priceHigh: 800,
  players: 4,
  name: "Aria",
  email: "aria@",
  bio: "",
  query: "",
  language: 0,
  region: -1,
  color: { r: 0.9, g: 0.35, b: 0.2, a: 1 } as PickedColor,
  typed: "",
  pin: "",
  comp: "",
  keyboard: "none" as KeyboardLayout | "none",
  device: "keyboardMouse" as InputDevice,
  yaw: 30,
  saved: 0,
});
let s = initial();
const defined = new WeakSet<object>();

/**
 * `demo:keyboard|controls:<value>`: the controls previews' states; false when the key is not theirs.
 * `demo:device:<device>` sets the key-binding rows' device and answers false, so the components'
 * demo also draws the prompts for it (og.ui.setDevice).
 */
export function controlsDemo(_world: World, args: readonly string[]): boolean {
  const [key, value = ""] = args;
  if (key === "keyboard") s.keyboard = value === "none" ? "none" : (value as KeyboardLayout);
  else if (key === "controls" && value === "reset") s = initial();
  else {
    if (key === "device") s.device = value as InputDevice;
    return false;
  }
  return true;
}

const w = (desktop: number): Size => (phone() ? grow : px(desktop));

// ── foundations ─────────────────────────────────────────────────────────

/** `picture`: a 16:9 texture for the art slot and the viewport (null shows their placeholders). */
export function foundationsScreen(picture: Texture | null): void {
  const t = og.ui.frameTheme();
  page("Foundations", () => {
    panel("Type", "Text: roles and tones", w(360), () => {
      og.ui.text("Display", "Ember Saga", { role: "display" });
      og.ui.text("Title", "Chapter II", { role: "title" });
      og.ui.text("Section", "The Ash Road", { role: "heading" });
      og.ui.text("Body", "The caravan leaves at dawn. Pack light and keep to the ridge.", { width: grow });
      og.ui.row("Tones", { gap: t.space.md, wrap: true }, () => {
        og.ui.text("Muted", "Muted", { tone: "muted" });
        og.ui.text("Success", "+120 gold", { tone: "success", role: "numeric" });
        og.ui.text("Danger", "−18 HP", { tone: "danger", role: "numeric" });
        og.ui.text("Caption", "Updated 2 min ago", { role: "caption", tone: "faint" });
      });
      og.ui.separator("Rule", { label: "Prose" });
      og.ui.prose("Lore", "The [b]Ember Blade[/b] was forged in the [color=warn]deep[/color] mines of Kharak.\n\nIt burns whoever holds it without a [link=codex]pact[/link].", {});
    });
    panel("Media", "Icons, art slots and shapes", w(400), () => {
      // The badges stand out over their marks' top edge (more at large text): the row keeps that much room under the
      // heading, as laid last tick.
      const over = Math.max(0, ...["Mail", "Shield"].map((k) => {
        const mark = og.ui.bounds(`Foundations/Body/View/Content/Panels/Media/Icons/${k}`);
        const badge = og.ui.bounds(`Foundations/Body/View/Content/Panels/Media/Icons/${k}/Badge`);
        return mark && badge ? Math.ceil(mark.y - badge.y) : 0;
      }));
      og.ui.row("Icons", { gap: t.space.md, cross: "center", wrap: true, padding: { left: 0, top: over, right: 0, bottom: 0 } }, () => {
        og.ui.iconMark("Mail", "chat-bubble", { badge: "3", label: "Messages" });
        og.ui.iconMark("Bell", "bell", { dot: true, label: "Alerts" });
        og.ui.iconMark("Gem", "gem", { plate: "circle", tone: "primary", label: "Gems" });
        og.ui.iconMark("Shield", "shield", { plate: "square", tone: "success", badge: "120", label: "Defence" });
        og.ui.iconMark("Warn", "warning-triangle", { tone: "warn", size: 32, label: "Warning" });
      });
      og.ui.row("Art", { gap: t.space.sm, wrap: true }, () => {
        og.ui.artSlot("Portrait", { kind: "portrait", aspect: 0.75, width: px(96), label: "Hero portrait", frame: "primary" });
        og.ui.artSlot("Item", { kind: "itemIcon", width: px(72), mask: "round", label: "Sword" });
        og.ui.artSlot("Avatar", { kind: "avatar", width: px(72), mask: "circle", loading: true, label: "Loading avatar" });
        og.ui.artSlot("Key", { kind: "keyArt", aspect: 16 / 9, width: px(128), texture: picture, loading: !picture, label: "Key art" });
        og.ui.artSlot("Crop", { kind: "portrait", aspect: 0.75, width: px(72), texture: picture, focal: { x: 0.7, y: 0.5 }, label: "Cropped to a portrait" });
      });
      og.ui.row("Shapes", { gap: t.space.md, cross: "center", wrap: true }, () => {
        og.ui.shapeBox("Pill", { kind: "capsule", width: px(56), height: px(24), fill: "primary" });
        og.ui.shapeBox("Gem", { kind: "diamond", width: px(32), fill: "info", fillTo: "primary", glow: "info" });
        og.ui.shapeBox("Star", { kind: "star", width: px(32), sides: 5, fill: "warn" });
        og.ui.shapeBox("Ring", { kind: "ring", width: px(32), fill: "success", thickness: 4 });
        og.ui.shapeBox("Hex", { kind: "polygon", width: px(32), sides: 6, fill: "panelRaised", stroke: "lineStrong" });
      });
    });
    panel("Layers", "Viewport, emitter and effect layer", w(360), () => {
      s.yaw = og.ui.viewFrame("Preview", { yaw: s.yaw, aspect: 4 / 3, texture: picture, label: "Character preview", hint: "Drag to turn" }).yaw;
      og.ui.row("Fx", { gap: t.space.md, width: grow, wrap: true }, () => {
        og.ui.effectLayer("Shine", { material: "shine", sweep: true, color: "warn" }, () => {
          og.ui.card("Legend", { title: "Legendary", eyebrow: "Drop", width: px(150) }, () => {});
        });
        og.ui.column("Burst", { width: { kind: "grow", min: 120 }, height: px(96), align: "center", cross: "center" }, () => {
          og.ui.emitter("Sparks", { preset: "sparkle" });
          og.ui.text("Label", "Level up!", { role: "heading", tone: "primary", width: grow, align: "center" });
        });
      });
      og.ui.space("Gap", { step: "sm" });
      og.ui.separator("Strong", { strong: true });
    });
  });
}

// ── buttons and choices ─────────────────────────────────────────────────

export function controlsScreen(): void {
  const t = og.ui.frameTheme();
  page("Controls", () => {
    panel("Buttons", "Buttons: variants, sizes, icons, badges, loading, disabled", w(480), () => {
      og.ui.row("Variants", { gap: t.space.sm, wrap: true, cross: "center" }, () => {
        og.ui.pushButton("Primary", "Play", { variant: "primary", icon: "play" });
        og.ui.pushButton("Secondary", "Settings", { variant: "secondary" });
        og.ui.pushButton("Ghost", "Credits", { variant: "ghost" });
        og.ui.pushButton("Quiet", "Skip", { variant: "quiet" });
        og.ui.pushButton("Danger", "Delete", { variant: "danger", icon: "trash" });
      });
      og.ui.row("States", { gap: t.space.sm, wrap: true, cross: "center" }, () => {
        og.ui.pushButton("Small", "Small", { size: "small" });
        og.ui.pushButton("Large", "Claim reward", { size: "large", variant: "primary", badge: "3" });
        og.ui.pushButton("Loading", "Saving", { loading: true });
        og.ui.pushButton("Disabled", "Shop", { disabled: true, disabledReason: "Reach level 5 to open the shop" });
        og.ui.pushButton("Pinned", "Pinned", { selected: true, icon: "map-pin" });
      });
      // The count badge stands out over its button's end edge (more at large text): the row's gap keeps that much room
      // more, as laid last tick.
      const mail = og.ui.bounds("Controls/Body/View/Content/Panels/Buttons/Icons/Mail");
      const mailBadge = og.ui.bounds("Controls/Body/View/Content/Panels/Buttons/Icons/Mail/Badge");
      const past = mail && mailBadge ? Math.max(0, Math.ceil(mailBadge.x + mailBadge.width - (mail.x + mail.width))) : 0;
      og.ui.row("Icons", { gap: t.space.sm + past, wrap: true, cross: "center" }, () => {
        og.ui.iconButton("Close", "cross", { label: "Close" });
        og.ui.iconButton("Mail", "chat-bubble", { label: "Messages", badge: "12" });
        og.ui.iconButton("Bell", "bell", { label: "Alerts", dot: true, variant: "secondary" });
        og.ui.iconButton("Fav", "star", { label: "Favourite", selected: true, round: true });
        og.ui.iconButton("Gear", "settings", { label: "Settings", variant: "ghost", size: "large" });
        og.ui.iconButton("Off", "share-android", { label: "Share", disabled: true });
      });
      const r = og.ui.holdButton("Hold", "Delete save", { variant: "danger", holdMs: 900, holdingLabel: "Keep holding…", hint: "Hold to delete. This cannot be undone." });
      if (r.confirmed) s.saved++;
      og.ui.holdButton("Toggle", "Abandon quest", { variant: "secondary", toggle: true, hint: "Tap to start, tap again to stop" });
    });
    panel("Toggles", "Switches and checkboxes", w(300), () => {
      s.music = og.ui.toggleSwitch("Music", s.music, { label: "Music", description: "Background tracks", width: grow }).on;
      s.vibration = og.ui.toggleSwitch("Vibration", s.vibration, { label: "Vibration", width: grow }).on;
      og.ui.toggleSwitch("Cloud", true, { label: "Cloud saves", description: "Signed out", disabled: true, width: grow });
      og.ui.separator("Rule", {});
      s.subtitles = og.ui.checkField("Subtitles", s.subtitles, { label: "Subtitles" }).on;
      og.ui.checkField("All", false, { label: "All notifications", mixed: true });
      s.terms = og.ui.checkField("Terms", s.terms, { label: "I accept the terms of service", invalid: !s.terms }).on;
      og.ui.checkField("Locked", true, { label: "Crash reports (required)", disabled: true });
    });
    panel("Choices", "Radios, segmented tabs and a cycler", w(360), () => {
      s.difficulty = og.ui.radioGroup("Difficulty", {
        label: "Difficulty",
        selected: s.difficulty,
        options: [{ label: "Story", description: "Enjoy the plot" }, { label: "Normal", description: "As it was meant" }, { label: "Hard", description: "Enemies hit twice as hard" }, { label: "Nightmare", disabled: true, description: "Finish the game first" }],
      }).selected;
      s.mode = og.ui.radioGroup("Mode", {
        label: "Mode",
        variant: "cards",
        row: true,
        selected: s.mode,
        options: [{ label: "Solo", icon: "user" }, { label: "Duo", icon: "group" }, { label: "Squad", icon: "flag" }],
      }).selected;
      s.view = og.ui.segmented("View", { label: "View", selected: s.view, items: [{ label: "Grid", icon: "cube" }, { label: "List", icon: "menu" }, { label: "Map", icon: "map" }] }).selected;
      s.quality = og.ui.cycler("Quality", ["Low", "Medium", "High", "Ultra"], s.quality, { label: "Graphics quality" }).selected;
    });
  });
}

// ── ranges ──────────────────────────────────────────────────────────────

export function rangesScreen(): void {
  page("Ranges", () => {
    panel("Sliders", "Sliders: percent, stepped with marks, balance, disabled", w(420), () => {
      s.volume = og.ui.valueSlider("Volume", s.volume, { label: "Master volume", format: { percent: true } }).value;
      s.sensitivity = og.ui.valueSlider("Sensitivity", s.sensitivity, {
        label: "Look sensitivity",
        min: 1,
        max: 10,
        step: 1,
        ticks: true,
        marks: [{ value: 1, label: "Slow" }, { value: 5, label: "Default" }, { value: 10, label: "Fast" }],
      }).value;
      s.balance = og.ui.valueSlider("Balance", s.balance, { label: "Balance", min: -1, max: 1, step: 0.1, origin: 0 }).value;
      og.ui.valueSlider("Locked", 0.3, { label: "Voice chat (muted by host)", disabled: true, format: { percent: true } });
    });
    panel("Range", "A range and steppers", w(380), () => {
      const r = og.ui.rangeSlider("Price", s.priceLow, s.priceHigh, { label: "Price", min: 0, max: 1000, step: 50, minGap: 100, format: { unit: " g" } });
      s.priceLow = r.low;
      s.priceHigh = r.high;
      s.players = og.ui.numberStepper("Players", s.players, { label: "Players", min: 1, max: 8 }).value;
      og.ui.numberStepper("Rounds", 3, { label: "Rounds", min: 1, max: 9, wrap: true, format: { unit: " rounds" } });
      og.ui.numberStepper("Off", 2, { label: "Bots", disabled: true });
    });
  });
}

// ── fields and pickers ──────────────────────────────────────────────────

const LANGUAGES = ["English", "Español", "Français", "Deutsch", "Italiano", "Português", "Русский", "Polski", "Türkçe", "العربية", "日本語", "한국어", "简体中文", "繁體中文"];
const ITEMS = ["Iron sword", "Iron shield", "Iron helm", "Oak bow", "Oak staff", "Ember blade", "Frost dagger", "Healing potion", "Mana potion"];

export function fieldsScreen(): void {
  page("Fields", () => {
    panel("Inputs", "Text fields: label, helper, error, counter, suffix, disabled", w(400), () => {
      s.name = og.ui.inputField("Name", s.name, { label: "Hero name", icon: "user", required: true, helper: "Shown to other players", maxChars: 16, counter: true }).value;
      s.email = og.ui.inputField("Email", s.email, { label: "Email", error: s.email.includes(".") ? "" : "Enter a whole address, like name@example.com" }).value;
      og.ui.inputField("Gold", "250", { label: "Offer", suffix: "gold" });
      og.ui.inputField("Off", "Guest-4821", { label: "Account", disabled: true });
      s.bio = og.ui.inputArea("Bio", s.bio, { label: "About you", placeholder: "A line or two for your profile", rows: 3, maxChars: 140, counter: true }).value;
    });
    panel("Pickers", "Search, selects and a colour", w(380), () => {
      s.query = og.ui.searchBox("Search", s.query, { placeholder: "Search items", suggestions: ITEMS, filter: true, recent: ["Ember blade", "Mana potion"] }).query;
      s.language = og.ui.dropdown("Language", { label: "Language", selected: s.language, options: LANGUAGES.map((label) => ({ label })) }).selected;
      s.region = og.ui.dropdown("Region", {
        label: "Region",
        placeholder: "Choose a region",
        error: s.region < 0 ? "Pick the region you play from" : "",
        selected: s.region,
        options: [{ label: "Europe", group: "Recommended" }, { label: "North America", group: "Recommended" }, { label: "Asia", group: "Other" }, { label: "Oceania", group: "Other", description: "Higher latency" }, { label: "Antarctica", group: "Other", disabled: true }],
      }).selected;
      s.color = og.ui.colorPicker("Livery", s.color, { label: "Livery", alpha: true, swatches: ["#e63946", "#f4a261", "#2a9d8f", "#264653", "#8338ec", "#ffffff"] }).color;
    });
  });
}

// ── key bindings and the on-screen keyboard ─────────────────────────────

const ACTIONS: Array<{ name: string; label: string; key: string[]; pad: string[]; locked?: boolean }> = [
  { name: "jump", label: "Jump", key: ["Space"], pad: ["pad.a"] },
  { name: "crouch", label: "Crouch", key: ["KeyC", "ControlLeft"], pad: ["pad.b"] },
  { name: "interact", label: "Interact", key: ["KeyE"], pad: ["pad.x"] },
  { name: "sprint", label: "Sprint", key: ["ShiftLeft"], pad: ["pad.leftStick"] },
  { name: "map", label: "Open map", key: ["KeyM"], pad: ["pad.back"] },
  { name: "menu", label: "Pause menu", key: ["Escape"], pad: ["pad.start"], locked: true },
];

function defineActions(world: World): void {
  if (defined.has(world)) return;
  defined.add(world);
  for (const a of ACTIONS) {
    const r = og.input.defineAction(world, { name: a.name, label: a.label, locked: a.locked ?? false, bindings: [...a.key.map((control) => ({ control })), ...a.pad.map((control) => ({ control }))] });
    if (!r.ok) throw new Error(r.reason);
  }
  const r = og.input.defineAction(world, { name: "move", label: "Move", kind: "axis2", bindings: [{ up: "KeyW", left: "KeyA", down: "KeyS", right: "KeyD" }, { control: "pad.leftXY" }] });
  if (!r.ok) throw new Error(r.reason);
}

export function bindingsScreen(world: World): void {
  const t = og.ui.frameTheme();
  defineActions(world);
  page("Bindings", () => {
    panel("Controls", "Key bindings: press a slot, then a key or a button", w(620), () => {
      const devices: InputDevice[] = ["keyboardMouse", "xbox", "playstation", "switch"];
      const picked = og.ui.segmented("Device", { label: "Device", selected: Math.max(0, devices.indexOf(s.device)), items: [{ label: "Keyboard" }, { label: "Xbox" }, { label: "PlayStation" }, { label: "Switch" }] }).selected;
      s.device = devices[picked] ?? "keyboardMouse";
      og.ui.column("Rows", { width: grow, gap: t.space.xxs }, () => {
        for (const a of ["move", ...ACTIONS.map((x) => x.name)]) {
          og.ui.keyBindingRow(world, og.ui.idi("Row", ["move", ...ACTIONS.map((x) => x.name)].indexOf(a)), { action: a, device: s.device, slots: s.device === "keyboardMouse" ? 2 : 1 });
        }
      });
      og.ui.row("Footer", { width: grow, align: "end" }, () => {
        if (og.ui.pushButton("ResetAll", "Reset all", { variant: "ghost", size: "small" }).clicked) og.input.resetBindings(world);
      });
    });
    panel("Typing", "On-screen keyboard: pads and touch", w(380), () => {
      og.ui.text("Typed", s.typed ? `Name: ${s.typed}` : "No name yet", { tone: s.typed ? "normal" : "muted" });
      og.ui.row("Open", { gap: t.space.sm, wrap: true }, () => {
        if (og.ui.pushButton("Letters", "Name your hero", { variant: "primary", icon: "edit-pencil" }).clicked) s.keyboard = "letters";
        if (og.ui.pushButton("Kana", "かな", {}).clicked) s.keyboard = "kana";
        if (og.ui.pushButton("Numbers", "PIN", {}).clicked) s.keyboard = "numeric";
      });
    });
  });
  const open = s.keyboard !== "none";
  const k = og.ui.onScreenKeyboard("Keyboard", open, {
    // The PIN is a field of its own: the name's keyboard and the PIN's each edit their own text.
    value: s.keyboard === "numeric" ? s.pin : s.typed,
    layout: open ? (s.keyboard as KeyboardLayout) : "letters",
    kana: true,
    title: s.keyboard === "numeric" ? "Enter your PIN" : "Name your hero",
    placeholder: s.keyboard === "numeric" ? "4 digits" : "Up to 16 letters",
    maxChars: s.keyboard === "numeric" ? 4 : 16,
    password: s.keyboard === "numeric",
    // The game's dictionary: what the composition converts to (the keyboard adds its katakana).
    candidates: s.comp === "ゆうしゃ" ? ["勇者"] : s.comp === "ひかり" ? ["光"] : [],
  });
  s.comp = k.composition;
  if (open && s.keyboard === "numeric") s.pin = k.value;
  else if (open) s.typed = k.value;
  if (k.submitted || k.closed) s.keyboard = "none";
}
