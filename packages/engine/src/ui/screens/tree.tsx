import { type UiEvent, View } from "@origingame/engine/ui";

export interface TreeProps {
  /** The visible rows in order, a keyed collection of { id, label, depth (0 at the top), open (true, false, or null for a leaf), selected }; the game flattens the tree as it opens and closes. */
  rows: ReadonlyArray<{ id: string; label: string; depth: number; open: boolean | null; selected: boolean }>;
  /** The tree's accessible name. */
  label: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The toggle event, with the row's key where it comes from a row. */
  onToggle?: (e: UiEvent, key?: string) => void;
}

/**
 * A HUD plate of rows, each indented by its depth with a faint guide
 * down from its parent; the chosen row is washed in the accent.
 */
export function Tree(props: TreeProps) {
  return (
    <View class="w-full bg-linear-to-b from-[#171f30d6] to-[#080c16e0] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] rounded-tl-[13.712px] rounded-tr-none rounded-br-[13.712px] rounded-bl-none text-accent-hover py-1">
      {/* TODO(world UI): <list id="Tree"> is not ported (bb11's later head) */}
    </View>
  );
}

export const fixtures: Record<string, Partial<TreeProps>> = {
  default: {"label":"Scene","rows":[{"id":"w","label":"World","depth":0,"open":true,"selected":false},{"id":"p","label":"Player","depth":1,"open":null,"selected":true},{"id":"c","label":"Camera","depth":1,"open":false,"selected":false}]},
  overflow: {"label":"Scene","rows":[{"id":"t0","label":"World","depth":0,"open":true,"selected":false},{"id":"t1","label":"Region of the Ninefold Gate","depth":1,"open":true,"selected":false},{"id":"t2","label":"Lighthouse","depth":2,"open":true,"selected":false},{"id":"t3","label":"Lantern_03 (the one on the north stair)","depth":3,"open":null,"selected":true},{"id":"t4","label":"Player","depth":1,"open":null,"selected":false},{"id":"t5","label":"Camera","depth":1,"open":true,"selected":false},{"id":"t6","label":"Light","depth":2,"open":true,"selected":false},{"id":"t7","label":"Particles","depth":3,"open":null,"selected":false},{"id":"t8","label":"Audio","depth":1,"open":false,"selected":false}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"toggle":"Open or close","choose":"Select"} as const;
