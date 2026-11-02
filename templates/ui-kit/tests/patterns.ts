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
  b.dump(`${prefix}gear-idle`);
  // Without its art: the items' and the figure's placeholders (the standing figure, the kinds' marks).
  b.cmd("pat:art:off");
  b.shot(`${prefix}gear-placeholder`, 0.4);
  b.cmd("pat:art:on");
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
    // Let go there: refused, the error sound, and the helm shakes back home.
    b.cmd("touch_end", 0.05);
    b.shot(`${prefix}gear-drop-refused`, 0.1);
    b.cmd(`touch_move_to:${BAG}/helm`, 0.4);
    b.cmd("touch_start", 0.1);
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
  // Without its art: the style's placeholders where the portraits and the cards' art go (the path a game takes
  // before its art lands, beside the painted one above).
  b.cmd("pat:art:off");
  b.shot(`${prefix}cards-placeholder`, 0.4);
  b.cmd("pat:art:on");
  if (!phone) {
    // Hover: the card lifts clear of the fan, straight and larger; its neighbours part.
    b.cmd(`touch_move_to:${FAN}/bash`, 0.1);
    b.shot(`${prefix}cards-hover`, 0.5);
  }
  // A card that plays on release, held below the play line (over the middle of the hand), then above it over the table
  // (ready: it stops under the header), let go back in the hand.
  b.cmd(`touch_move_to:${FAN}/defend`, 0.05);
  b.cmd("touch_start", 0.1);
  b.cmd(`touch_move_to:${FAN}`, 0.2);
  b.shot(`${prefix}cards-held`, 0.3);
  b.cmd("touch_move_to:Cards/Table", 0.2);
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
  // Refused by the taunt: Bash let go on the Red Louse shakes back, the louse's taunt mark lit in danger ink.
  b.cmd("pat:reset", 0.3);
  b.cmd(`touch_move_to:${FAN}/bash`, 0.05);
  b.cmd("touch_start", 0.1);
  b.cmd("touch_move_to:Foes/louse", 0.25);
  b.cmd("touch_end", 0.05);
  b.shot(`${prefix}cards-refused`, 0.1);
  b.dump(`${prefix}cards-refused`);
  // An unplayable card (a Wound) shakes when tried: its gem carries a cross.
  b.cmd(`touch_move_to:${FAN}/wound`, 0.4);
  b.cmd("touch_start", 0.05);
  b.cmd("touch_move_to:Cards/Table", 0.1);
  b.shot(`${prefix}cards-unplayable`, 0.1);
  b.cmd("touch_end", 0.4);
  // One energy left: Bash (2) shaded, its cost in danger ink; Inflame's lowered cost marked down.
  b.cmd("pat:energy:1");
  b.shot(`${prefix}cards-unaffordable`, 0.5);
  b.dump(`${prefix}cards-unaffordable`);
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
    b.cmd("touch_move_to:Cards/Top/Heading/Title", 0.2);
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

/** The reward choice in its states. */
function rewards(b: ReturnType<typeof builder>, phone: boolean, prefix = ""): void {
  const OPT = "Choice/Options";
  b.cmd("show:rewards");
  b.cmd("pat:reset");
  // The deal: cards flip in one after another (a frame mid-deal, then dealt).
  b.shot(`${prefix}rewards-deal`, 0.15);
  b.shot(`${prefix}rewards-idle`, 0.8);
  if (!phone) {
    // Hover: the card lifts and its edge brightens.
    b.cmd(`touch_move_to:${OPT}/vigor`, 0.1);
    b.shot(`${prefix}rewards-hover`, 0.5);
  }
  // A locked reward pressed: it shakes and is refused.
  b.seq((at) => tap(at, `${OPT}/crown`), 0.05);
  b.shot(`${prefix}rewards-locked-shake`, 0.1);
  // Banish armed: every card shows its cross; Back disarms.
  b.seq((at) => tap(at, "Choice/Actions/Banish"), 0.3);
  b.shot(`${prefix}rewards-banish-armed`, 0.4);
  b.dump(`${prefix}rewards-banish`);
  b.seq((at) => key(at, "Escape"), 0.3);
  // Two picks: two selected (their ticks), the Confirm count.
  b.cmd("pat:picks:2");
  b.cmd("pat:rselect:thunder,vigor");
  b.shot(`${prefix}rewards-selected`, 0.5);
  b.dump(`${prefix}rewards-selected`);
  b.cmd("pat:picks:1");
  // Reroll: a new set deals in.
  b.seq((at) => tap(at, "Choice/Actions/Reroll"), 0.2);
  b.shot(`${prefix}rewards-rerolled`, 1.0);
  // Taken: the one pressed rises, the others fall away.
  b.seq((at) => tap(at, `${OPT}/gold`), 0.1);
  b.shot(`${prefix}rewards-taken`, 0.2);
  b.dump(`${prefix}rewards-taken`);
  b.cmd("pat:reset");
}

/** The gacha banner and a pull in its states. */
function gacha(b: ReturnType<typeof builder>, phone: boolean, prefix = ""): void {
  const BTN = "Banner/Body/Main/Foot/Pulls/Buttons";
  b.cmd("show:gacha");
  b.cmd("pat:reset");
  b.shot(`${prefix}gacha-idle`, 0.8);
  b.dump(`${prefix}gacha-idle`);
  if (!phone) {
    b.cmd(`touch_move_to:${BTN}/Multi`, 0.1);
    b.shot(`${prefix}gacha-hover`, 0.4);
  }
  // Another banner: its tab current, its art and title; then the purse too short for ten.
  // In portrait the tabs lead the words (under the splash's face); beside the art they are the rail.
  b.seq((at) => tap(at, phone ? "Banner/Body/Main/Tabs/weapon" : "Banner/Body/Tabs/weapon"), 0.3);
  // The weapon banner wears a real texture as its key art (the others draw the placeholder).
  b.shot(`${prefix}gacha-weapon`, 0.6);
  b.cmd("pat:banner:event");
  b.cmd("pat:balance:500");
  b.shot(`${prefix}gacha-short`, 0.5);
  b.dump(`${prefix}gacha-short`);
  b.cmd("pat:balance:2400");
  // A ten-pull: the charge (gold, the pull holds a 5★), the first reveal, the 5★ reveal, the summary.
  b.seq((at) => tap(at, `${BTN}/Multi`), 0.05);
  b.shot(`${prefix}gacha-charge`, 0.6);
  b.shot(`${prefix}gacha-reveal`, 1.2);
  for (let k = 0; k < 5; k++) b.seq((at) => tap(at, "PullLayer/Panel/Stage"), 0.25);
  // The 5★: its burst as the last star lands, then the reveal at rest.
  b.shot(`${prefix}gacha-reveal-burst`, 1.0);
  b.shot(`${prefix}gacha-reveal-legendary`, 0.9);
  b.dump(`${prefix}gacha-reveal`);
  b.seq((at) => tap(at, "PullLayer/Panel/Stage/Bar/Skip"), 0.2);
  b.shot(`${prefix}gacha-summary`, 1.2);
  b.dump(`${prefix}gacha-summary`);
  b.seq((at) => key(at, "Escape"), 0.4);
  // A single pull of a 5★ straight to its reveal.
  b.cmd("pat:pull:1");
  b.shot(`${prefix}gacha-single-reveal`, 2.2);
  b.seq((at) => key(at, "Escape"), 0.2);
  b.seq((at) => key(at, "Escape"), 0.4);
  b.cmd("pat:reset");
}

/** Every pattern screen in its states, then right to left and at 200 % text. */
export function patternsScript(phone: boolean): Script {
  const b = builder(0.3);
  if (phone) b.cmd("viewport:phone");
  gear(b, phone);
  cards(b, phone);
  rewards(b, phone);
  gacha(b, phone);
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
  b.cmd("show:rewards");
  b.shot("rtl-rewards", 1.0);
  b.cmd("show:gacha");
  b.shot("rtl-gacha", 0.8);
  b.cmd("pat:pull:10");
  b.seq((at) => key(at, "Escape"), 0.2);
  b.shot("rtl-gacha-summary", 1.2);
  b.seq((at) => key(at, "Escape"), 0.4);
  b.cmd("pat:reset");
  // The rate line in Japanese and Korean: each locale's own sentence in its own face, no English fallback.
  b.cmd("locale:ja");
  b.shot("ja-gacha", 0.8);
  b.cmd("locale:ko");
  b.shot("ko-gacha", 0.8);
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
  b.cmd("show:rewards");
  b.shot("text200-rewards", 1.0);
  b.cmd("show:gacha");
  b.shot("text200-gacha", 0.8);
  b.cmd("pat:pull:10");
  b.seq((at) => key(at, "Escape"), 0.2);
  b.shot("text200-gacha-summary", 1.2);
  b.seq((at) => key(at, "Escape"), 0.4);
  b.cmd("pat:reset");
  b.cmd("demo:text:100");
  return b.end();
}
