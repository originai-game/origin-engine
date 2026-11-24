import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View} from "@origingame/engine/ui";
import { Button } from "../controls/button";

export interface TreeProps {
  /** The visible rows in order, a keyed collection of { id, label, depth (0 at the top), open (true, false, or null for a leaf), selected }; the game flattens the tree as it opens and closes. */
  rows?: ReadonlyArray<{ id: string; label: string; depth: number; open: boolean | null; selected: boolean }>;
  /** The tree's accessible name. */
  label: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The toggle event, with the row's key where it comes from a row. */
  onToggle?: (e: UiEvent, key?: string) => void;
}

/**
 * A HUD plate of rows, each indented by its depth with a faint guide
 * down from its parent; the chosen row is washed in the accent from its
 * leading edge. A leaf has no twisty and keeps its room, so its name
 * lines up with its siblings' that open. A name longer than its row
 * wraps inside it, at any word.
 */
export function Tree(props: TreeProps) {
  return (
    <Scroll class={`w-full py-1 ${packClass("Panel", "frame")}`} role="tree" label={props.label}>
      <For each={props.rows ?? []}>
        {(r) => (
          <View class={`group/row relative flex-row items-start gap-0.5 py-0.5 pr-3 w-full text-fg-muted hover:bg-surface-hover data-selected:text-fg data-selected:bg-surface-hover data-selected:inset-shadow-bar data-selected:inset-shadow-hud-accent`} role="treeitem" level={r().depth + 1} selected={r().selected} expanded={r().open ?? undefined} data-selected={r().selected} data-leaf={r().open == null} style={{ paddingLeft: `calc(${r().depth * 1.143}rem + 0.571rem)` }}>
            <Show when={r().depth > 0}>
          <View class={`absolute top-0 bottom-0 w-px bg-rim-soft`} style={{ left: `calc(${(r().depth - 0.5) * 1.143}rem + 0.571rem)` }} />
            </Show>
            <Button class={`size-6 p-0 justify-center items-center shrink-0 ${r().open == null ? "invisible" : ""}`} label={r().label} onPress={(e) => props.onToggle?.(e, r().id)}>
              <Icon class={`size-3 ${r().open ? "rotate-90" : ""}`} name="play" />
            </Button>
            <Button class={`grow shrink basis-0 min-w-0 text-start justify-start whitespace-normal py-0.5 data-selected:text-fg data-selected:font-semibold`} onPress={(e) => props.onChoose?.(e, r().id)}>
              <Text class="min-w-0 text-start">{r().label}</Text>
            </Button>
          </View>
        )}
      </For>
    </Scroll>
  );
}

export const fixtures: Record<string, Partial<TreeProps>> = {
  default: {"label":"Scene","rows":[{"id":"w","label":"World","depth":0,"open":true,"selected":false},{"id":"p","label":"Player","depth":1,"open":null,"selected":true},{"id":"c","label":"Camera","depth":1,"open":false,"selected":false}]},
  overflow: {"label":"Scene","rows":[{"id":"t0","label":"World","depth":0,"open":true,"selected":false},{"id":"t1","label":"Region of the Ninefold Gate","depth":1,"open":true,"selected":false},{"id":"t2","label":"Lighthouse","depth":2,"open":true,"selected":false},{"id":"t3","label":"Lantern_03 (the one on the north stair)","depth":3,"open":null,"selected":true},{"id":"t4","label":"Player","depth":1,"open":null,"selected":false},{"id":"t5","label":"Camera","depth":1,"open":true,"selected":false},{"id":"t6","label":"Light","depth":2,"open":true,"selected":false},{"id":"t7","label":"Particles","depth":3,"open":null,"selected":false},{"id":"t8","label":"Audio","depth":1,"open":false,"selected":false}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"toggle":"Open or close","choose":"Select"} as const;
