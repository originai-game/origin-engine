import { og, defineSystem } from "@origingame/engine";

import { createEngine, type GameToken } from "../../../../src/runtime.ts";

import { card, type CardHandResult, type CardPlay, type CardTarget, type HandCard, type RewardChoiceDesc, type RewardChoiceResult, type RewardOption } from "@origingame/engine/extensions/card/index.ts";

import { systemIds, kit, run, byPath, centre } from "../../../../tests/support/patterns.ts";

// ── card hand ───────────────────────────────────────────────────────────

export const HAND: HandCard[] = [
  { key: "strike", name: "Strike", cost: 1, typeLine: "Attack", text: "Deal [b]6[/b] damage.", needsTarget: true },
  { key: "defend", name: "Defend", cost: 1, typeLine: "Skill", text: "Gain [b]5[/b] Block." },
  { key: "bash", name: "Bash", cost: 2, typeLine: "Attack", text: "Deal 8 damage.", needsTarget: true, rarity: "rare" },
  { key: "anger", name: "Anger", cost: 0, typeLine: "Attack", text: "Deal 6 damage." },
  { key: "wound", name: "Wound", typeLine: "Status", text: "Unplayable.", playable: false },
];

// Two enemies on the table: the first can be targeted, the second (behind a taunt) cannot.
export const TARGETS: CardTarget[] = [
  { key: "cultist", name: "Cultist", rect: { x: 760, y: 80, width: 160, height: 180 } },
  { key: "louse", name: "Louse", rect: { x: 980, y: 80, width: 160, height: 180 }, valid: false },
];

/** A hand along the bottom of a 1280×720 desktop (or right to left), the game removing each card played. */
export function hand(rtl = false, width = 1200, phone = false) {
  const cards = (phone ? [...HAND, { key: "clash", name: "Clash", cost: 0, typeLine: "Attack", text: "Deal 14 damage." }] : HAND).map((c) => ({ ...c }));
  const played: CardPlay[] = [];
  const results: CardHandResult[] = [];
  const sys = defineSystem(`UpHand${systemIds.next++}`, {}, () => {
    og.ui.anchor("Table", { anchor: "bottom", width: { kind: "fixed", px: width }, direction: "column" } as never, () => {
      const r = card.cardHand("Hand", { cards, targets: TARGETS, cardWidth: 150 });
      results.push(r);
      if (r.played) {
        played.push(r.played);
        cards.splice(cards.findIndex((c) => c.key === r.played!.key), 1);
      }
    });
  });
  const made = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, rtl ? { root: kit } : {});
  if (!made.ok) throw new Error(made.reason);
  const engine = made.value;
  if (rtl) {
    og.i18n.load(engine.world, "strings/ar.ogt");
    og.i18n.setLocale(engine.world, "ar-EG");
  }
  og.ui.previewViewport(engine.world, phone ? { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as never : { width: 1280, height: 720, dpr: 1, coarsePointer: false, breakpoint: "desktop", safeArea: { left: 0, top: 0, right: 0, bottom: 0 } } as never);
  engine.advance(6);
  return { engine, played, results, cards };
}

// ── reward choice ───────────────────────────────────────────────────────

export const REWARDS: RewardOption[] = [
  { key: "strike", title: "Thunder Strike", kind: "Attack", text: "Deal [b]12[/b] damage.", rarity: "rare" },
  { key: "vigor", title: "Vigor", kind: "Blessing", text: "+20 % Max Health.", rarity: "epic", tag: "New" },
  { key: "crown", title: "Crown", kind: "Relic", text: "Gain a crown.", rarity: "legendary", locked: true, lockedHint: "Reach floor 3" },
];

/** A reward choice the game answers as it should: it keeps the selection, rerolls and banishes. */
export function rewards(extra: Partial<RewardChoiceDesc> = {}, after?: () => void) {
  let options = REWARDS.map((o) => ({ ...o }));
  let selected: string[] = [];
  let rerolls = extra.rerolls ?? -1;
  let banishes = extra.banishes ?? -1;
  const results: RewardChoiceResult[] = [];
  const engine = run(() => {
    const r = card.rewardChoice("Rew", { ...extra, options, selected, rerolls, banishes, cardWidth: 240 });
    results.push(r);
    selected = [...r.selected];
    if (r.rerolled) {
      rerolls -= 1;
      options = options.map((o) => ({ ...o, key: `${o.key}2` }));
    }
    if (r.banished) {
      banishes -= 1;
      options = options.filter((o) => o.key !== r.banished);
    }
    after?.();
  });
  const press = (k: string) => {
    const e = byPath(engine, `Rew/Cards/Options/${k}`) ?? byPath(engine, `Rew/Actions/${k}`)!;
    engine.probe.commands(`0:touch_move:${centre(e)};0.05:touch_start;0.1:touch_end`);
    engine.advance(14);
  };
  const picked = () => results.flatMap((r) => r.picked);
  return { engine, results, press, picked, options: () => options };
}
