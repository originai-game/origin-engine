// The genre pattern previews' scripts (src/patterns.ts): each screen in its
// states, a frame of each. The desktop and phone playtests share them; each
// state's frame is `<screen>-<state>.png`, compared with
// crates/og-ui/references/<tier>/kit-patterns[-phone]-<file>.

import { key, tap } from "./lib.ts";

export interface Script {
  commands: string;
  files: string[];
}

/** Builds a script: `cmd` runs a command, `shot` takes a frame. */
function builder(t0: number) {
  const steps: string[] = [];
  const files: string[] = [];
  let t = t0;
  return {
    cmd(c: string, after = 0.1) {
      steps.push(`${t.toFixed(2)}:${c}`);
      t += after;
    },
    seq(f: (at: number) => string, after = 0.5) {
      steps.push(f(t));
      t += after;
    },
    shot(name: string, settle = 0.6) {
      t += settle;
      steps.push(`${t.toFixed(2)}:screenshot:${name}.png`);
      files.push(`${name}.png`);
      t += 0.1;
    },
    dump(name: string) {
      steps.push(`${t.toFixed(2)}:dump_ui:${name}.json`);
      t += 0.05;
    },
    end(): Script {
      steps.push(`${t.toFixed(2)}:quit`);
      return { commands: steps.join(";"), files };
    },
  };
}


/** The gear screen (inventory grid and paperdoll) in its states. `phone` keeps to what a finger does. */
function gear(b: ReturnType<typeof builder>, phone: boolean, prefix = ""): void {
  const BAG = "Bag/Cells";
  b.cmd("show:gear");
  b.cmd("pat:reset");
  b.shot(`${prefix}gear-idle`, 0.8);
  if (!phone) {
    // Hover: the item lifts and its tooltip names it.
    b.cmd(`touch_move_to:${BAG}/helm`, 0.1);
    b.shot(`${prefix}gear-hover`, 0.5);
  }
  if (phone) {
    // A phone in portrait stacks the doll over the bags, so one drag cannot reach both: the helm is put in
    // the hand (as the item menu's Equip would), the doll's slots light by what they take, a tap equips it.
    b.cmd("pat:pick:helm");
    b.shot(`${prefix}gear-drag-accepts`, 0.4);
    b.seq((at) => tap(at, "Slots/head"), 0.3);
    b.shot(`${prefix}gear-equipped`, 0.6);
    b.dump(`${prefix}gear-equipped`);
    // Down to the bags: drags inside the backpack.
    b.cmd("touch_move_to:Gear/Body", 0.05);
    b.cmd("scroll:0,600", 0.4);
  } else {
    // A drag: pressed on the helm, held over the paperdoll's head (it accepts helmets), over a
    // slot that refuses it (the feet), over the locked ring (a refused footprint), released on the head.
    b.cmd(`touch_move_to:${BAG}/helm`, 0.05);
    b.cmd("touch_start", 0.1);
    b.cmd("touch_move_to:Slots/head", 0.2);
    b.shot(`${prefix}gear-drag-accepts`, 0.3);
    b.cmd("touch_move_to:Slots/feet", 0.2);
    b.shot(`${prefix}gear-drag-rejects`, 0.3);
    b.cmd(`touch_move_to:${BAG}/ring`, 0.2);
    b.shot(`${prefix}gear-drag-invalid`, 0.3);
    b.cmd("touch_move_to:Slots/head", 0.2);
    b.cmd("touch_end", 0.2);
    b.shot(`${prefix}gear-equipped`, 0.6);
    b.dump(`${prefix}gear-equipped`);
  }
  // One item over another of its size: the swap footprint (let go outside: it goes home).
  b.cmd(`touch_move_to:${BAG}/scroll`, 0.05);
  b.cmd("touch_start", 0.1);
  b.cmd(`touch_move_to:${BAG}/gem`, 0.2);
  b.shot(`${prefix}gear-drag-swap`, 0.3);
  b.cmd("touch_move_to:Gear/Header/Title", 0.2);
  b.cmd("touch_end", 0.3);
  // A stack onto a stack of the same: the merge footprint.
  b.cmd(`touch_move_to:${BAG}/mana`, 0.05);
  b.cmd("touch_start", 0.1);
  b.cmd(`touch_move_to:${BAG}/potions`, 0.2);
  b.shot(`${prefix}gear-drag-merge`, 0.3);
  // Back off the grid and let go outside it: the item goes home.
  b.cmd("touch_move_to:Gear/Header/Title", 0.2);
  b.cmd("touch_end", 0.3);
  // Long press: the item menu.
  b.cmd(`touch_move_to:${BAG}/greatsword`, 0.05);
  b.cmd("touch_start", 0.7);
  b.cmd("touch_end", 0.1);
  b.shot(`${prefix}gear-menu`, 0.4);
  b.dump(`${prefix}gear-menu`);
  b.seq((at) => key(at, "Escape"), 0.3);
  // A tag lit (the dim of everything else), a selected item.
  b.cmd("pat:highlight:potion");
  b.shot(`${prefix}gear-highlight`, 0.5);
  b.cmd("pat:highlight:");
  b.cmd("pat:select:bow");
  b.shot(`${prefix}gear-selected`, 0.5);
  b.cmd("pat:reset");
  if (!phone) {
    // Keys: a click on the potions puts the focus on the backpack; the arrows walk the cursor,
    // Enter picks up, R turns, the footprint follows the cursor.
    b.seq((at) => tap(at, `${BAG}/potions`), 0.3);
    b.seq((at) => key(at, "ArrowRight"), 0.2);
    b.seq((at) => key(at, "ArrowRight"), 0.2);
    b.shot(`${prefix}gear-cursor`, 0.4);
    b.seq((at) => key(at, "Enter"), 0.2);
    b.seq((at) => key(at, "KeyR"), 0.2);
    b.seq((at) => key(at, "ArrowDown"), 0.2);
    b.shot(`${prefix}gear-keys-carry`, 0.4);
    b.dump(`${prefix}gear-keys`);
    b.seq((at) => key(at, "Escape"), 0.3);
  } else {
    // A finger's hand: the item picked, the Rotate button shown.
    b.cmd("pat:pick:boots");
    b.shot(`${prefix}gear-hand`, 0.5);
    b.seq((at) => key(at, "Escape"), 0.3);
    b.cmd("pat:reset");
  }
}

/** The combat table's hand in its states. `phone` keeps to what a finger does. */
function cards(b: ReturnType<typeof builder>, phone: boolean, prefix = ""): void {
  const FAN = "Hand/Fan";
  b.cmd("show:cards");
  b.cmd("pat:reset");
  b.shot(`${prefix}cards-idle`, 0.8);
  if (!phone) {
    // Hover: the card lifts clear of the fan, straight and larger; its neighbours part.
    b.cmd(`touch_move_to:${FAN}/bash`, 0.1);
    b.shot(`${prefix}cards-hover`, 0.5);
  }
  // A card that plays on release, held below the play line, then above it (ready), let go back in the hand.
  b.cmd(`touch_move_to:${FAN}/defend`, 0.05);
  b.cmd("touch_start", 0.1);
  b.cmd("touch_move_to:Foot/Bar/Energy", 0.2);
  b.shot(`${prefix}cards-held`, 0.3);
  b.cmd("touch_move_to:Cards/Top/Title", 0.2);
  b.shot(`${prefix}cards-ready`, 0.3);
  b.cmd(`touch_move_to:${FAN}/wound`, 0.2);
  b.cmd("touch_end", 0.4);
  // A targeted card dragged up: the arc to the enemy behind a taunt (invalid), then to the other (valid), played there.
  b.cmd(`touch_move_to:${FAN}/bash`, 0.05);
  b.cmd("touch_start", 0.1);
  b.cmd("touch_move_to:Foes/louse", 0.25);
  b.shot(`${prefix}cards-aim-invalid`, 0.3);
  b.cmd("touch_move_to:Foes/cultist", 0.25);
  b.shot(`${prefix}cards-aim-valid`, 0.3);
  b.dump(`${prefix}cards-aim`);
  b.cmd("touch_end", 0.2);
  b.shot(`${prefix}cards-played`, 0.5);
  b.dump(`${prefix}cards-played`);
  // Out of energy now for a 2-cost card: Inflame and Anger stay playable, an unplayable Wound shakes when tried.
  b.cmd(`touch_move_to:${FAN}/wound`, 0.05);
  b.cmd("touch_start", 0.05);
  b.cmd("touch_move_to:Cards/Top/Title", 0.1);
  b.shot(`${prefix}cards-refused`, 0.1);
  b.cmd("touch_end", 0.4);
  // A card drawn flies in; a card shown large; the enemy's turn.
  b.cmd("pat:draw:strike");
  b.shot(`${prefix}cards-drawn`, 0.25);
  b.cmd("pat:inspect:inflame");
  b.shot(`${prefix}cards-inspect`, 0.6);
  b.dump(`${prefix}cards-inspect`);
  b.cmd("pat:inspect:");
  b.cmd("pat:turn:off");
  b.shot(`${prefix}cards-enemy-turn`, 0.5);
  b.cmd("pat:reset");
  if (!phone) {
    // Keys: a click on Strike puts the focus on the hand; the cursor walks it, Enter aims Bash, Right steps
    // the target onto the enemy behind a taunt (the arc says invalid), Back stops aiming.
    b.seq((at) => tap(at, "Hand/Fan/strike"), 0.2);
    b.cmd("touch_move_to:Cards/Top/Title", 0.2);
    b.seq((at) => key(at, "ArrowRight"), 0.2);
    b.seq((at) => key(at, "ArrowRight"), 0.2);
    b.shot(`${prefix}cards-keys-cursor`, 0.4);
    b.seq((at) => key(at, "Enter"), 0.2);
    b.shot(`${prefix}cards-keys-aim`, 0.4);
    b.seq((at) => key(at, "ArrowRight"), 0.2);
    b.shot(`${prefix}cards-keys-aim-invalid`, 0.4);
    b.dump(`${prefix}cards-keys`);
    b.seq((at) => key(at, "Escape"), 0.3);
  } else {
    // A finger: a tap picks a card (it stays up), a second tap aims it, a tap on the enemy plays it.
    b.seq((at) => tap(at, `${FAN}/strike`), 0.3);
    b.shot(`${prefix}cards-picked`, 0.4);
    b.seq((at) => tap(at, `${FAN}/strike`), 0.3);
    b.cmd("touch_move_to:Foes/cultist", 0.2);
    b.shot(`${prefix}cards-tap-aim`, 0.4);
    b.cmd("touch_start", 0.05);
    b.cmd("touch_end", 0.3);
    b.dump(`${prefix}cards-tapped`);
  }
  b.cmd("pat:reset");
}

/** Every pattern screen in its states, then right to left and at 200 % text. */
export function patternsScript(phone: boolean): Script {
  const b = builder(0.3);
  if (phone) b.cmd("viewport:phone");
  gear(b, phone);
  cards(b, phone);
  b.cmd("locale:ar");
  b.cmd("show:gear");
  b.shot("rtl-gear", 0.8);
  b.cmd("pat:pick:helm");
  b.shot("rtl-gear-hand", 0.5);
  // Back puts what the keys carry back where it was.
  b.seq((at) => key(at, "Escape"), 0.3);
  b.cmd("pat:reset");
  b.cmd("show:cards");
  b.cmd("pat:reset");
  b.shot("rtl-cards", 0.8);
  b.cmd(`touch_move_to:Hand/Fan/bash`, 0.05);
  b.cmd("touch_start", 0.1);
  b.cmd("touch_move_to:Foes/cultist", 0.25);
  b.shot("rtl-cards-aim", 0.3);
  b.cmd("touch_end", 0.3);
  b.cmd("pat:reset");
  b.cmd("locale:en");
  b.cmd("demo:text:200");
  b.cmd("show:gear");
  b.shot("text200-gear", 0.8);
  b.cmd("pat:select:greatsword");
  b.shot("text200-gear-selected", 0.5);
  b.cmd("pat:reset");
  b.cmd("show:cards");
  b.shot("text200-cards", 0.8);
  b.cmd("pat:inspect:bash");
  b.shot("text200-cards-inspect", 0.6);
  b.cmd("pat:reset");
  b.cmd("demo:text:100");
  return b.end();
}
