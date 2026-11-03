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
export function builder(t0: number) {
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

/** The saga map, the match-3 board's goals and stars, and the idle generators with the prestige, in their states. */
function progression(b: ReturnType<typeof builder>, phone: boolean, prefix = ""): void {
  b.cmd("show:saga");
  // Opens centred on the current level (21), the path lit up to it, 22 on locked, the boss at 25.
  b.shot(`${prefix}saga-idle`, 1.0);
  b.dump(`${prefix}saga`);
  // A locked level refused (it shakes), then an open one picked: Play names it.
  b.seq((at) => tap(at, "Map/View/Path/l22"), 0.15);
  b.shot(`${prefix}saga-refused`, 0.1);
  b.seq((at) => tap(at, "Map/View/Path/l20"), 0.4);
  b.shot(`${prefix}saga-picked`, 0.4);
  // The player moves on: the map follows to the new current level.
  b.cmd("pat:saga:l3");
  b.shot(`${prefix}saga-start`, 1.0);
  b.cmd("pat:reset");
  b.cmd("show:board");
  b.shot(`${prefix}board-idle`, 0.8);
  // The third star crossed (it pops), and a goal met (its chip turns to success with a tick).
  b.cmd("pat:score:27000");
  b.shot(`${prefix}board-star`, 0.12);
  b.cmd("pat:goal:20,6");
  b.shot(`${prefix}board-goals-met`, 0.8);
  b.cmd("pat:reset");
  b.cmd("show:generators");
  b.shot(`${prefix}generators-idle`, 1.0);
  b.dump(`${prefix}generators`);
  if (!phone) {
    // Ascend held part-way: the hold fills, nothing resets yet.
    b.seq((at) => `${at.toFixed(2)}:touch_move_to:Prestige/Action/Button;${(at + 0.05).toFixed(2)}:touch_start`, 0.5);
    b.shot(`${prefix}generators-ascend-holding`, 0.05);
    b.cmd("touch_end", 0.6);
  }
  // A purse that pays none: every Buy glass, Ascend flat with why.
  b.cmd("pat:purse:1000");
  b.shot(`${prefix}generators-poor`, 1.6);
  b.cmd("pat:reset");
}

/** The store and the battle pass in their states. */
function commerce(b: ReturnType<typeof builder>, phone: boolean, prefix = ""): void {
  b.cmd("show:store");
  // The purse, the featured bundle (its saving and its timer), the owned crown, then the daily rotation.
  b.shot(`${prefix}store-idle`, 1.0);
  b.dump(`${prefix}store`);
  if (!phone) {
    b.cmd("touch_move_to:Offers/ashbringer", 0.05);
    b.shot(`${prefix}store-hover`, 0.5);
  }
  // An offer the purse cannot pay shakes and says why; one it can opens the game's confirm, then is owned.
  b.cmd("pat:gems:1000");
  b.seq((at) => tap(at, "Offers/ashbringer"), 0.15);
  b.shot(`${prefix}store-refused`, 0.05);
  b.dump(`${prefix}store-refused`);
  b.cmd("pat:gems:2400");
  b.seq((at) => tap(at, "Offers/ashbringer"), 0.4);
  b.shot(`${prefix}store-confirm`, 0.6);
  b.seq((at) => key(at, "Enter"), 0.4);
  b.shot(`${prefix}store-bought`, 0.8);
  b.dump(`${prefix}store-bought`);
  b.cmd("pat:reset");
  b.cmd("show:pass");
  // Opens on the player's tier (12): the free track claimed to 10 but 8, and 11 ready; premium locked.
  b.shot(`${prefix}pass-idle`, 1.2);
  b.dump(`${prefix}pass`);
  b.seq((at) => tap(at, "Tiers/L11/Free"), 0.15);
  b.shot(`${prefix}pass-claim`, 0.08);
  b.seq((at) => tap(at, "Tiers/L13/Free"), 0.15);
  b.shot(`${prefix}pass-refused`, 0.05);
  // The premium track bought: its rewards up to the level ready, then all claimed at once.
  b.cmd("pat:premium:on");
  b.shot(`${prefix}pass-premium`, 0.8);
  b.seq((at) => tap(at, "Actions/ClaimAll"), 0.4);
  b.shot(`${prefix}pass-claimed`, 0.8);
  b.dump(`${prefix}pass-claimed`);
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
  progression(b, phone);
  commerce(b, phone);
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
  b.cmd("show:saga");
  b.shot("rtl-saga", 1.0);
  b.cmd("show:generators");
  b.shot("rtl-generators", 0.8);
  b.cmd("show:store");
  b.shot("rtl-store", 1.0);
  b.cmd("show:pass");
  b.shot("rtl-pass", 1.2);
  b.cmd("pat:pull:10");
  b.seq((at) => key(at, "Escape"), 0.2);
  b.shot("rtl-gacha-summary", 1.2);
  b.seq((at) => key(at, "Escape"), 0.4);
  b.cmd("pat:reset");
  // Every pattern screen in Japanese, Korean and Chinese: each word the screen shows is the locale's (banner and
  // character names stay proper nouns), in its own face, with no English fallback.
  for (const loc of ["ja", "ko", "zh"]) {
    b.cmd(`locale:${loc === "zh" ? "zh-CN" : loc}`);
    b.cmd("show:gacha");
    b.shot(`${loc}-gacha`, 0.8);
    b.cmd("show:cards");
    b.shot(`${loc}-cards`, 0.8);
    b.cmd("show:rewards");
    b.shot(`${loc}-rewards`, 1.0);
    b.cmd("show:gear");
    b.shot(`${loc}-gear`, 0.8);
    b.cmd("show:store");
    b.shot(`${loc}-store`, 1.0);
    b.cmd("show:pass");
    b.shot(`${loc}-pass`, 1.2);
  }
  // The pull's own words: the reveal's NEW badge and hint, then the summary's.
  b.cmd("locale:ja");
  b.cmd("show:gacha");
  b.cmd("pat:pull:1");
  b.shot("ja-gacha-reveal", 2.2);
  b.seq((at) => key(at, "Escape"), 0.2);
  b.seq((at) => key(at, "Escape"), 0.4);
  b.cmd("pat:pull:10");
  b.seq((at) => key(at, "Escape"), 0.2);
  b.shot("ja-gacha-summary", 1.2);
  b.seq((at) => key(at, "Escape"), 0.4);
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
  b.cmd("show:rewards");
  b.shot("text200-rewards", 1.0);
  b.cmd("show:gacha");
  b.shot("text200-gacha", 0.8);
  b.cmd("pat:pull:10");
  b.seq((at) => key(at, "Escape"), 0.2);
  b.shot("text200-gacha-summary", 1.2);
  b.seq((at) => key(at, "Escape"), 0.4);
  b.cmd("pat:reset");
  b.cmd("show:store");
  b.shot("text200-store", 1.0);
  b.cmd("show:pass");
  b.shot("text200-pass", 1.2);
  b.cmd("demo:text:100");
  return b.end();
}

/** The story patterns (src/story.ts) in their states: the conversation, the visual novel and its backlog, the quest log and tracker. */
function story(b: ReturnType<typeof builder>, phone: boolean, prefix = ""): void {
  b.cmd("show:dialogue");
  b.cmd("pat:dlg:reset");
  // Mira's line typing out (part of it shown, the rest held in place), then whole with the choices risen in.
  b.shot(`${prefix}dialogue-typing`, 0.45);
  b.shot(`${prefix}dialogue-choices`, 2.4);
  b.dump(`${prefix}dialogue-choices`);
  if (!phone) {
    // The pointer over the check: its tag's odds, the lit edge.
    b.cmd("touch_move_to:Talk/Choices/List/dlg.gate.persuade", 0.05);
    b.shot(`${prefix}dialogue-hover`, 0.4);
    b.cmd("touch_move_to:Talk/Tools/Auto", 0.05);
  }
  // The timer near its end: the bar in danger, pulsing; the default choice marked.
  b.shot(`${prefix}dialogue-timer`, phone ? 5.4 : 4.8);
  // Asked who else knows (its number on the keys, a tap on a phone), the answer, then back at the gate: that choice visited.
  if (phone) b.seq((at) => tap(at, "Talk/Choices/List/dlg.gate.who"), 0.3);
  else b.seq((at) => key(at, "Digit4"), 0.3);
  b.shot(`${prefix}dialogue-answer`, 2.4);
  b.seq((at) => tap(at, "Talk/Hit"), 0.3);
  b.shot(`${prefix}dialogue-visited`, 2.4);
  b.dump(`${prefix}dialogue-visited`);
  // The check taken: the d20 rolls, its word in success or failure.
  b.seq((at) => tap(at, "Talk/Choices/List/dlg.gate.persuade"), 0.1);
  b.shot(`${prefix}dialogue-roll`, 0.5);
  b.dump(`${prefix}dialogue-roll`);
  // The backlog over the conversation: every line, the reply chosen under each.
  if (phone) b.seq((at) => tap(at, "Talk/Tools/Log"), 0.3);
  else b.seq((at) => key(at, "KeyX"), 0.3);
  b.shot(`${prefix}dialogue-log`, 0.8);
  b.dump(`${prefix}dialogue-log`);
  b.seq((at) => key(at, "Escape"), 0.3);
  // A voiced line: the bars by the name.
  b.cmd("pat:dlg:reset");
  b.cmd("pat:dlg:voice:on");
  b.shot(`${prefix}dialogue-voice`, 2.4);
  b.cmd("pat:dlg:reset");
  b.cmd("show:vn");
  // The narrator in the box (no plate, muted), then Aria with her plate, Auto on, the choices, and the backlog.
  b.shot(`${prefix}vn-narrator`, 3.2);
  b.seq((at) => tap(at, "Talk/Hit"), 0.3);
  b.shot(`${prefix}vn-line`, 2.0);
  b.dump(`${prefix}vn-line`);
  b.cmd("pat:dlg:auto:on");
  b.shot(`${prefix}vn-auto`, 0.3);
  b.shot(`${prefix}vn-choices`, 5.0);
  b.cmd("pat:dlg:log:on");
  b.shot(`${prefix}vn-log`, 0.8);
  b.cmd("pat:dlg:reset");
  b.cmd("show:quests");
  // The main tab: the tracked quest shown, its objectives (done, counted, optional, hidden), rewards, Track and Map.
  b.shot(`${prefix}quests-idle`, 1.0);
  b.dump(`${prefix}quests`);
  if (phone) {
    b.seq((at) => tap(at, "Log/Side/List/Active/kit_dawn"), 0.3);
    b.shot(`${prefix}quests-detail`, 0.8);
    b.dump(`${prefix}quests-detail`);
    b.seq((at) => tap(at, "Log/Detail/Back"), 0.3);
  } else {
    b.cmd("touch_move_to:Log/Side/List/Active/kit_dawn", 0.05);
    b.shot(`${prefix}quests-hover`, 0.4);
  }
  // The side tab: the active one, then the completed and failed under their heading; a completed one's stamp.
  b.cmd("pat:quest:tab:side");
  b.shot(`${prefix}quests-side`, 0.8);
  if (phone) {
    b.seq((at) => tap(at, "Log/Side/List/Done/kit_letters"), 0.3);
    b.shot(`${prefix}quests-completed`, 0.8);
    b.seq((at) => tap(at, "Log/Detail/Back"), 0.3);
    b.seq((at) => tap(at, "Log/Side/List/Done/kit_tide"), 0.3);
    b.shot(`${prefix}quests-failed`, 0.8);
    b.seq((at) => tap(at, "Log/Detail/Back"), 0.3);
  } else {
    b.seq((at) => tap(at, "Log/Side/List/Done/kit_letters"), 0.3);
    b.shot(`${prefix}quests-completed`, 0.8);
    b.cmd("pat:quest:select:kit_tide");
    b.shot(`${prefix}quests-failed`, 0.8);
  }
  // The daily (new, breathing), tracked.
  b.cmd("pat:quest:tab:daily");
  if (phone) b.seq((at) => tap(at, "Log/Side/List/Active/kit_supply"), 0.3);
  b.shot(`${prefix}quests-daily`, 0.8);
  b.seq((at) => tap(at, "Log/Detail/Actions/Track"), 0.3);
  b.shot(`${prefix}quests-tracked`, 0.6);
  b.dump(`${prefix}quests-tracked`);
  if (phone) b.seq((at) => tap(at, "Log/Detail/Back"), 0.3);
  b.cmd("pat:quest:tab:main");
  // The HUD's tracker: the tracked quests, then a chart found (the count pops).
  b.cmd("pat:quest:hud:on");
  b.shot(`${prefix}quests-tracker`, 0.8);
  b.cmd("pat:quest:progress:kit_dawn.charts");
  b.shot(`${prefix}quests-tracker-progress`, 0.15);
  b.cmd("pat:quest:hud:off");
}

/** The story patterns in their states, then right to left, in Japanese, Korean and Chinese, and at 200 % text. */
export function storyScript(phone: boolean): Script {
  const b = builder(0.3);
  if (phone) b.cmd("viewport:phone");
  story(b, phone);
  for (const loc of ["ar", "ja", "ko", "zh"]) {
    const name = loc === "ar" ? "rtl" : loc;
    b.cmd(`locale:${loc === "zh" ? "zh-CN" : loc}`);
    b.cmd("show:dialogue");
    b.cmd("pat:dlg:reset");
    b.shot(`${name}-dialogue`, 2.6);
    b.cmd("show:vn");
    b.cmd("pat:dlg:reset");
    b.seq((at) => tap(at, "Talk/Hit"), 0.3);
    b.shot(`${name}-vn`, 2.6);
    b.cmd("show:quests");
    b.shot(`${name}-quests`, 1.0);
  }
  b.cmd("locale:en");
  b.cmd("demo:text:200");
  b.cmd("show:dialogue");
  b.cmd("pat:dlg:reset");
  b.shot("text200-dialogue", 2.6);
  b.cmd("show:vn");
  b.cmd("pat:dlg:reset");
  b.seq((at) => tap(at, "Talk/Hit"), 0.3);
  b.shot("text200-vn", 2.6);
  b.cmd("show:quests");
  b.shot("text200-quests", 1.0);
  b.cmd("demo:text:100");
  return b.end();
}

/** The roster and wider-game previews (src/genres.ts) in their states. `phone` keeps to what a finger does. */
function genres(b: ReturnType<typeof builder>, phone: boolean): void {
  // The lobby: the lineup not ready, Ready up tapped, in queue, the ready check, accepted.
  b.cmd("show:lobby");
  b.cmd("pat:reset");
  b.shot("lobby-idle", 1.0);
  b.dump("lobby");
  b.seq((at) => tap(at, "Party/Bar/Action"), 0.3);
  b.shot("lobby-ready", 0.6);
  b.cmd("pat:lobby:search");
  b.shot("lobby-search", 0.6);
  b.cmd("pat:lobby:found");
  b.shot("lobby-found", 0.8);
  b.dump("lobby-found");
  b.seq((at) => tap(at, "Check/Panel/Actions/Accept"), 0.3);
  b.shot("lobby-accepted", 0.5);
  b.cmd("pat:lobby:reset");
  // The character select: Mira under the player's cursor and the rival's Kael locked, another hovered, then locked in.
  b.cmd("show:select");
  b.shot("select-idle", 1.0);
  b.cmd("pat:chars:hover:vesper");
  b.shot("select-hover", 0.5);
  b.seq((at) => tap(at, "Chars/Foot/Lock"), 0.3);
  b.shot("select-locked", 0.6);
  b.dump("select-locked");
  b.cmd("pat:chars:reset");
  // The versus screen as it lands, then nearly loaded.
  b.cmd("show:versus");
  b.cmd("pat:vs:reset");
  b.shot("versus", 1.2);
  b.cmd("pat:vs:progress:0.92");
  b.shot("versus-loading", 0.4);
  // The draft: the player's turn with Vesper hovered, locked in (Red's turn), then the ban phase.
  b.cmd("show:draft");
  b.shot("draft-pick", 1.0);
  b.seq((at) => tap(at, "Middle/Foot/Action"), 0.3);
  b.shot("draft-locked", 0.6);
  b.dump("draft-locked");
  b.cmd("pat:draft:reset");
  b.cmd("pat:draft:phase:ban");
  b.cmd("pat:draft:hover:wren");
  b.shot("draft-ban", 0.6);
  b.cmd("pat:draft:reset");
  // The world map: the realm under its fog, the old king's seat's detail (a fast-travel point), then two kinds turned off.
  b.cmd("show:worldMap");
  b.shot("map-idle", 1.0);
  b.seq((at) => tap(at, "Pins/amberhold"), 0.3);
  b.shot("map-detail", 0.6);
  b.dump("map-detail");
  b.cmd("pat:map:select:");
  b.cmd("pat:map:hide:shop,poi");
  b.shot("map-filtered", 0.5);
  b.cmd("pat:map:reset");
  // The song select: Crystal Echo on the wheel, a tap on the row above, the locked song, the preview paused.
  b.cmd("show:songs");
  b.shot("songs-idle", 1.0);
  b.dump("songs");
  b.seq((at) => tap(at, "Wheel/Wheel/desert-drum"), 0.3);
  b.shot("songs-select", 0.6);
  b.cmd("pat:song:select:phoenix");
  b.shot("songs-locked", 0.6);
  b.cmd("pat:song:playing:off");
  b.shot("songs-paused", 0.4);
  b.cmd("pat:song:reset");
}

const GENRE_SCREENS = ["lobby", "select", "versus", "draft", "worldMap", "songs"] as const;

/** The roster and wider-game previews in their states, then each screen right to left, in Japanese, and at 200 % text. */
export function genresScript(phone: boolean): Script {
  const b = builder(0.3);
  if (phone) b.cmd("viewport:phone");
  genres(b, phone);
  for (const loc of ["ar", "ja"]) {
    b.cmd(`locale:${loc}`);
    for (const s of GENRE_SCREENS) {
      b.cmd(`show:${s}`);
      b.cmd("pat:reset");
      b.shot(`${loc === "ar" ? "rtl" : loc}-${s}`, 1.0);
    }
  }
  b.cmd("locale:en");
  b.cmd("demo:text:200");
  for (const s of GENRE_SCREENS) {
    b.cmd(`show:${s}`);
    b.cmd("pat:reset");
    b.shot(`text200-${s}`, 1.0);
  }
  b.cmd("demo:text:100");
  return b.end();
}

/** The systems games' previews (src/genres2.ts) in their states. `phone` keeps to what a finger does. */
function genres2(b: ReturnType<typeof builder>, phone: boolean): void {
  // The garage: the engine's sport tier previewed, the race tier refused (short), the sport tier bought, then the
  // livery's layers with the stripe moved up.
  const list = phone ? "Body/Parts" : "List";
  const act = phone ? "Body/Garage/Action" : "Side/Action";
  b.cmd("show:garage");
  b.cmd("pat:reset");
  b.shot("garage-idle", 1.0);
  b.dump("garage");
  b.seq((at) => tap(at, `${list}/engine.race`), 0.3);
  b.seq((at) => tap(at, act), 0.15);
  b.shot("garage-short", 0.4);
  b.seq((at) => tap(at, `${list}/engine.sport`), 0.3);
  b.seq((at) => tap(at, act), 0.3);
  b.shot("garage-bought", 0.6);
  b.dump("garage-bought");
  b.cmd("pat:garage:category:livery");
  b.shot("garage-livery", 0.8);
  b.seq((at) => tap(at, `${phone ? "Body" : "List"}/stripe/Up`), 0.3);
  b.shot("garage-layer", 0.5);
  b.dump("garage-layer");
  b.cmd("pat:garage:reset");
  // The ledger: the month's budget, fire underfunded (its warning), sorted by amount, the year's tab.
  b.cmd("show:ledger");
  b.shot("ledger-idle", 1.0);
  b.dump("ledger");
  b.cmd("pat:ledger:fund:fire=60");
  b.shot("ledger-underfunded", 0.5);
  b.dump("ledger-underfunded");
  b.seq((at) => tap(at, "Header/amount"), 0.3);
  b.shot("ledger-sorted", 0.5);
  b.dump("ledger-sorted");
  b.seq((at) => tap(at, "Tabs/year"), 0.3);
  b.shot("ledger-year", 0.8);
  b.cmd("pat:ledger:reset");
  // The city's overlays: traffic shown, power pressed (its legend), power pressed again (none).
  b.cmd("show:cityMap");
  b.shot("citymap-traffic", 1.0);
  b.seq((at) => tap(at, "Row/power"), 0.3);
  b.shot("citymap-power", 0.5);
  b.dump("citymap-power");
  b.seq((at) => tap(at, "Row/power"), 0.3);
  b.shot("citymap-off", 0.5);
  b.cmd("pat:ledger:reset");
  // The chat: every channel with links and a mention of the player, an @word's suggestions, the name written and sent,
  // the party's tab, then the HUD's chat over the game and opened to type.
  b.cmd("show:chat");
  b.shot("chat-idle", 1.0);
  b.dump("chat");
  b.cmd("pat:chat:draft:gg_@Ri");
  b.shot("chat-mention", 0.5);
  b.dump("chat-mention");
  b.seq((at) => tap(at, "Suggest/riven"), 0.3);
  b.seq((at) => tap(at, "Input/Send"), 0.4);
  b.shot("chat-sent", 0.6);
  b.dump("chat-sent");
  b.seq((at) => tap(at, "Tabs/Row/party"), 0.3);
  b.shot("chat-party", 0.6);
  b.cmd("pat:chat:reset");
  b.cmd("pat:chat:hud:on");
  b.shot("chat-hud", 0.8);
  b.cmd("pat:chat:typing:on");
  b.shot("chat-hud-typing", 0.6);
  b.cmd("pat:chat:reset");
  // The RTS console: the workers' orders, Build's page, Barracks by its key (A) awaiting its target, Back cancelling it,
  // Back to the orders; then group 2 (the knights) recalled by its tab, and on a desktop set to the selection with Ctrl+3.
  b.cmd("show:rts");
  b.shot("rts-idle", 1.0);
  b.dump("rts");
  b.seq((at) => tap(at, "Commands/Grid/Row#1/build"), 0.3);
  b.shot("rts-build", 0.5);
  if (phone) b.seq((at) => tap(at, "Commands/Grid/Row#0/barracks"), 0.3);
  else b.seq((at) => key(at, "KeyA"), 0.3);
  b.shot("rts-target", 0.5);
  b.dump("rts-target");
  if (!phone) {
    b.seq((at) => key(at, "Escape"), 0.3);
    b.seq((at) => tap(at, "Commands/Grid/Row#2/PageBack"), 0.3);
  } else b.cmd("pat:rts:target:");
  b.seq((at) => tap(at, "Tabs/g2"), 0.3);
  b.shot("rts-group", 0.5);
  b.dump("rts-group");
  if (!phone) {
    b.seq((at) => [`${at.toFixed(2)}:key_down:ControlLeft`, key(at + 0.05, "Digit3"), `${(at + 0.15).toFixed(2)}:key_up:ControlLeft`].join(";"), 0.4);
    b.dump("rts-assigned");
  }
  b.cmd("pat:rts:reset");
}

const GENRE2_SCREENS = ["garage", "ledger", "cityMap", "chat", "rts"] as const;

/** The systems games' previews in their states, then each screen right to left, in Japanese, and at 200 % text. */
export function genres2Script(phone: boolean): Script {
  const b = builder(0.3);
  if (phone) b.cmd("viewport:phone");
  genres2(b, phone);
  for (const loc of ["ar", "ja"]) {
    b.cmd(`locale:${loc}`);
    for (const s of GENRE2_SCREENS) {
      b.cmd(`show:${s}`);
      b.cmd("pat:reset");
      b.shot(`${loc === "ar" ? "rtl" : loc}-${s}`, 1.0);
    }
  }
  b.cmd("locale:en");
  b.cmd("demo:text:200");
  for (const s of GENRE2_SCREENS) {
    b.cmd(`show:${s}`);
    b.cmd("pat:reset");
    b.shot(`text200-${s}`, 1.0);
  }
  b.cmd("demo:text:100");
  return b.end();
}
