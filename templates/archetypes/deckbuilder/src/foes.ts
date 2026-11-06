// Ashen Crown's foes and the run's map: plain data. A foe cycles through
// its moves, one a turn, and shows the next as its intent before the
// heroes act (Slay the Spire's rule: the player always knows what comes).
// Health grows with the party: each seat past the first adds 60 %.

/** What a move does: its intent's icon. */
export const Intent = { Attack: 0, Block: 1, Buff: 2, Debuff: 3, AttackAll: 4, AttackBlock: 5 } as const;
export type IntentKind = (typeof Intent)[keyof typeof Intent];

export interface Move {
  readonly kind: IntentKind;
  /** Damage a hit (before strength and weak), and how many hits. */
  readonly dmg?: number;
  readonly hits?: number;
  readonly block?: number;
  /** Strength it gains (Buff). */
  readonly strength?: number;
  /** Weak or vulnerable it lays on its target (Debuff, or with an attack). */
  readonly weak?: number;
  readonly vulnerable?: number;
}

export interface FoeDef {
  readonly id: string;
  readonly hp: number;
  readonly moves: readonly Move[];
  readonly elite?: boolean;
  readonly boss?: boolean;
}

export const FOES: readonly FoeDef[] = [
  { id: "imp", hp: 13, moves: [{ kind: Intent.Attack, dmg: 6 }, { kind: Intent.Buff, strength: 2 }, { kind: Intent.Attack, dmg: 4 }] },
  { id: "hound", hp: 17, moves: [{ kind: Intent.Attack, dmg: 3, hits: 2 }, { kind: Intent.Attack, dmg: 8 }, { kind: Intent.Block, block: 6 }] },
  { id: "slime", hp: 20, moves: [{ kind: Intent.Attack, dmg: 7 }, { kind: Intent.Debuff, weak: 2 }, { kind: Intent.Attack, dmg: 5, vulnerable: 1 }] },
  { id: "knight", hp: 46, elite: true, moves: [{ kind: Intent.AttackBlock, dmg: 8, block: 10 }, { kind: Intent.Attack, dmg: 14 }, { kind: Intent.Buff, strength: 3 }] },
  { id: "king", hp: 84, boss: true, moves: [{ kind: Intent.Attack, dmg: 10 }, { kind: Intent.AttackAll, dmg: 5, weak: 1 }, { kind: Intent.Block, block: 14, strength: 2 }, { kind: Intent.Attack, dmg: 9, hits: 2 }] },
];

export const FOE = Object.fromEntries(FOES.map((f, i) => [f.id, i])) as Record<string, number>;

/** What a node on the map is. */
export const Node = { Fight: 0, Elite: 1, Rest: 2, Boss: 3 } as const;
export type NodeKind = (typeof Node)[keyof typeof Node];

export interface Step {
  /** The node, or two to choose between (a fork). */
  readonly options: readonly { readonly kind: NodeKind; readonly foes: readonly number[] }[];
}

/** The run: six steps, the third a fork (a fire, or another fight for one more card). */
export const MAP: readonly Step[] = [
  { options: [{ kind: Node.Fight, foes: [FOE.imp!, FOE.hound!] }] },
  { options: [{ kind: Node.Fight, foes: [FOE.slime!, FOE.slime!] }] },
  { options: [{ kind: Node.Rest, foes: [] }, { kind: Node.Fight, foes: [FOE.hound!, FOE.imp!, FOE.imp!] }] },
  { options: [{ kind: Node.Elite, foes: [FOE.knight!] }] },
  { options: [{ kind: Node.Rest, foes: [] }] },
  { options: [{ kind: Node.Boss, foes: [FOE.king!] }] },
];

/** The most foes on the field at once. */
export const FOE_SLOTS = 3;

/** A foe's health for a party of `seats`. */
export function foeHp(def: FoeDef, seats: number): number {
  return Math.round(def.hp * (1 + 0.6 * (seats - 1)));
}
