// The foundations and controls previews' scripts (src/controls.ts): each
// screen in its states, every interactive part idle, hovered, pressed,
// focused and disabled, then driven: a hold, a drag, a capture and its
// conflict, the on-screen keyboard typing letters and composing kana; then
// right to left and at 150 % text. The desktop and phone playtests share
// them; each frame is `<screen>-<state>.png`, compared with
// crates/og-ui/references/<tier>/kit-controls[-phone]-<file>.

import { builder, type Script } from "./components.ts";
import { key, tap, tapText } from "./lib.ts";

/** Every foundations and controls screen in its states. `phone` keeps to what a finger does (no hover, no keys, no pad). */
export function controlsScript(phone: boolean): Script {
  const b = builder(0.3);
  if (phone) b.cmd("viewport:phone");
  // A screen opens with the pointer parked on empty ground, so its idle frame shows no hover left by the last screen.
  const show = (screen: string) => {
    if (!phone) b.cmd("touch_move:1270,712");
    b.cmd(`show:${screen}`);
  };

  // Foundations: idle (the art slots' sheen, the shine sweeping), the viewport turned.
  show("foundations");
  b.shot("foundations-idle", 1.0);
  if (phone) b.scroll("Foundations", 600);
  if (phone) {
    // A finger's drag across it (towards the burst beside it).
    b.cmd("touch_move_to:Layers/Preview", 0.05);
    b.cmd("touch_start", 0.05);
    b.cmd("touch_move_to:Fx/Burst", 0.1);
    b.cmd("touch_end", 0.4);
  } else {
    b.seq((at) => tap(at, "Layers/Preview"), 0.2);
    b.seq((at) => key(at, "ArrowRight"), 0.2);
    b.seq((at) => key(at, "ArrowRight"), 0.2);
  }
  b.shot("foundations-turned", 0.6);
  b.dump("foundations");

  // Buttons and choices: every part at rest, then each state of the primary button, the disabled one's reason,
  // a hold filling, a switch flipped, a radio stepped, a segment and the cycler moved.
  show("controls");
  b.shot("controls-idle", 0.8);
  if (!phone) {
    b.cmd("touch_move_to:Variants/Primary", 0.1);
    b.shot("controls-hover", 0.4);
    b.cmd("touch_start", 0.05);
    b.shot("controls-press", 0.1);
    b.cmd("touch_end", 0.3);
    b.cmd("touch_move_to:States/Disabled", 0.1);
    b.shot("controls-disabled-reason", 1.0);
    b.seq((at) => key(at, "Tab"), 0.2);
    b.shot("controls-focus", 0.4);
    // The pad: the d-pad walks (the focus ring shows), A presses.
    b.cmd("gamepad:down", 0.05);
    b.cmd("gamepad:-", 0.2);
    b.shot("controls-pad-focus", 0.4);
  }
  // A hold: halfway full, then let go (it drains), then held to the end.
  b.cmd("touch_move_to:Hold/Button", 0.05);
  b.cmd("touch_start", 0.45);
  b.shot("controls-holding", 0.0);
  // Let go: the fill drains (a full one in a third of the hold), caught a moment after the release.
  b.cmd("touch_end", 0.02);
  b.shot("controls-hold-draining", 0.05);
  b.seq((at) => tap(at, "Toggles/Music"), 0.2);
  if (phone) b.scroll("Controls", 300);
  b.seq((at) => tap(at, "Terms"), 0.2);
  b.shot("controls-toggled", 0.5);
  if (phone) b.scroll("Controls", 300);
  b.seq((at) => tap(at, "Difficulty/Options/Option#2"), 0.2);
  b.seq((at) => tap(at, "Mode/Options/Option#1"), 0.2);
  if (phone) b.scroll("Controls", 300);
  b.seq((at) => tap(at, "View/Strip/Track/Item#2"), 0.2);
  b.seq((at) => tap(at, "Quality/Next"), 0.2);
  b.shot("controls-chosen", 0.6);
  if (!phone) {
    // Keys: Right steps the radio group's choice (Up and Down only move the focus, as a pad browses a list).
    b.seq((at) => tap(at, "Difficulty/Options/Option#0"), 0.1);
    b.seq((at) => key(at, "ArrowRight"), 0.2);
    b.shot("controls-radio-keys", 0.4);
  }
  b.dump("controls");

  // Ranges: idle, the volume dragged (its bubble), the sensitivity stepped by keys, the range's low knob moved.
  show("ranges");
  b.shot("ranges-idle", 0.8);
  // Held on the volume and dragged to the track's middle (where the balance's knob sits, under it).
  b.cmd("touch_move_to:Volume/Track", 0.05);
  b.cmd("touch_start", 0.1);
  b.cmd("touch_move_to:Balance/Track", 0.1);
  b.shot("ranges-drag", 0.2);
  b.cmd("touch_end", 0.2);
  if (!phone) {
    // A tap on the knob takes the focus without moving the value.
    b.seq((at) => tap(at, "Sensitivity/Track/Knob"), 0.1);
    b.seq((at) => key(at, "ArrowRight"), 0.1);
    b.seq((at) => key(at, "ArrowRight"), 0.1);
    b.shot("ranges-keys", 0.3);
  }
  // The range's low knob dragged to the track's middle (500 g, still a minimum gap below the high one).
  b.cmd("touch_move_to:Price/Track/Low", 0.05);
  b.cmd("touch_start", 0.1);
  b.cmd("touch_move_to:Price/Track", 0.1);
  b.cmd("touch_end", 0.2);
  b.seq((at) => tap(at, "Players/Frame/Inc"), 0.2);
  b.shot("ranges-stepped", 0.5);
  b.dump("ranges");

  // Fields: idle (an error, a counter), typing, the search's suggestions, a select open, a swatch picked.
  show("fields");
  b.shot("fields-idle", 0.8);
  b.seq((at) => tap(at, "Name/Frame/Field"), 0.2);
  b.cmd("type: the Brave", 0.6);
  b.shot("fields-typing", 0.3);
  b.seq((at) => tap(at, "Search/Frame/Field"), 0.2);
  b.cmd("type:iro", 0.6);
  b.shot("fields-suggestions", 0.4);
  if (!phone) {
    b.seq((at) => key(at, "ArrowDown"), 0.2);
    b.shot("fields-suggestion-focus", 0.3);
  }
  b.seq((at) => key(at, "Escape"), 0.2);
  if (phone) b.scroll("Fields", 300);
  b.seq((at) => tap(at, "Language/Trigger"), 0.2);
  b.shot("fields-dropdown", 0.5);
  // A long list filters as the player types (its filter has the focus): 日本 leaves 日本語.
  b.cmd("type:日本", 0.4);
  b.shot("fields-dropdown-filtered", 0.3);
  b.seq((at) => tapText(at, "=日本語"), 0.2);
  b.seq((at) => tap(at, "Region/Trigger"), 0.2);
  b.shot("fields-dropdown-groups", 0.5);
  b.seq((at) => key(at, "Escape"), 0.3);
  if (phone) b.scroll("Fields", 300);
  b.seq((at) => tap(at, "Livery/Swatches/Swatch#2"), 0.2);
  b.shot("fields-colour", 0.5);
  b.dump("fields");

  // Key bindings: idle, a slot waiting, a free key landing, a used one asking, swapped; a pad's rows; then the
  // on-screen keyboard typing letters, and composing kana with its candidates.
  show("bindings");
  b.cmd("demo:controls:reset");
  b.shot("bindings-idle", 0.8);
  b.seq((at) => tap(at, "Row#3/Main/Bound/Slots/Slot#0"), 0.1);
  b.shot("bindings-waiting", 0.4);
  b.seq((at) => key(at, "KeyF"), 0.3);
  b.shot("bindings-landed", 0.4);
  b.seq((at) => tap(at, "Row#3/Main/Bound/Slots/Slot#0"), 0.4);
  b.seq((at) => key(at, "KeyC"), 0.3);
  b.shot("bindings-conflict", 0.4);
  b.dump("bindings-conflict");
  b.seq((at) => tap(at, "Row#3/Conflict/Swap"), 0.3);
  b.shot("bindings-swapped", 0.4);
  // On a phone the keyboard's openers are below the rows: scrolled into view so the dump measures them whole.
  if (phone) b.scroll("Bindings", 600);
  b.dump("bindings");
  if (phone) b.scroll("Bindings", -600);
  for (const d of ["xbox", "playstation", "switch"]) {
    b.cmd(`demo:device:${d}`);
    b.shot(`bindings-${d}`, 0.4);
  }
  b.cmd("demo:device:keyboardMouse");
  b.cmd("demo:keyboard:letters");
  b.shot("bindings-keyboard", 0.8);
  for (const k of ["Shift", "=A", "=r", "=i", "=a"]) {
    if (k === "Shift") b.seq((at) => tap(at, "Keys/Row#2/Key#0"), 0.15);
    else b.seq((at) => tapText(at, k), 0.15);
  }
  b.shot("bindings-keyboard-typed", 0.3);
  if (!phone) {
    b.cmd("gamepad:down", 0.05);
    b.cmd("gamepad:-", 0.2);
    b.shot("bindings-keyboard-pad", 0.3);
  }
  b.cmd("demo:keyboard:kana");
  b.shot("bindings-kana", 0.8);
  for (const k of ["=ゆ", "=う", "=し", "=や"]) b.seq((at) => tapText(at, k), 0.15);
  b.seq((at) => tap(at, "Keys/Row#5/Key#2"), 0.2);
  b.shot("bindings-kana-composing", 0.3);
  b.dump("bindings-kana");
  // Another field's keyboard: the name's closes, the PIN's opens on its own empty text.
  b.cmd("demo:keyboard:none", 0.4);
  b.cmd("demo:keyboard:numeric", 0.8);
  // Two digits in: masked as dots.
  for (const k of ["=4", "=2"]) b.seq((at) => tapText(at, k), 0.15);
  b.shot("bindings-numeric", 0.3);
  b.cmd("demo:keyboard:none", 0.4);

  // Right to left: the rows run from the right, the sliders fill from the right, the switches' knobs start right.
  b.cmd("locale:ar");
  for (const s of ["controls", "ranges", "fields", "bindings"]) {
    show(s);
    b.shot(`rtl-${s}`, 0.8);
  }
  b.cmd("locale:en");

  // At the player's largest text size (150 %): every control grows with its text and cuts none of it.
  b.cmd("demo:text:150");
  for (const s of ["foundations", "controls", "ranges", "fields", "bindings"]) {
    show(s);
    b.shot(`text150-${s}`, 0.8);
    if (s === "controls") b.dump("controls-150");
  }
  b.cmd("demo:keyboard:letters");
  b.shot("text150-keyboard", 0.8);
  b.cmd("demo:keyboard:none", 0.4);
  b.cmd("demo:text:100");
  return b.end();
}
