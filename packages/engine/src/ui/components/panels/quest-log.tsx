import { For, Icon, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import "./quest-log.en.ogt";

export interface RpgQuestLogProps {
  /** The log's heading (Quest Journal). */
  title: string;
  /** The groups in order, keyed rows of { id, label, done }: done marks the group of finished quests, shown struck and quiet. */
  groups: ReadonlyArray<{ id: string; label: string; done: boolean }>;
  /** The quests, keyed rows of { id, group, name, tracked, fresh }: group a group's id; tracked the one the HUD follows; fresh an update the player has not read. */
  quests: ReadonlyArray<{ id: string; group: string; name: string; tracked: boolean; fresh: boolean }>;
  /** The quest shown in detail; empty is none. */
  selected: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The track event, with the row's key where it comes from a row. */
  onTrack?: (e: UiEvent, key?: string) => void;
}

/**
 * The quest list, grouped as the game groups it (main, side, finished):
 * each group a gold label in small capitals behind a short gold rule (a
 * finished group's quiet), each quest a row with its mark. The mark is a
 * diamond: hollow, lit gold on the quest the HUD tracks, filled in the
 * success colour once the quest is done. The selected row (the one in
 * detail) is washed gold from its leading edge with a gold bar on it and a
 * chevron toward the detail; a fresh update carries a New tag. Finished
 * quests are struck and quiet. Reports choose (a quest's key) and track (X
 * or pad Y on the focused quest, its key). It fills its height and scrolls
 * inside it.
 * Dropped against the .svelte: the chosen quest's name scrolling in place
 * when it does not fit (it ends in an ellipsis like the others), and the
 * compact touch/narrow row height tweak (one row height everywhere).
 */
export function RpgQuestLog(props: RpgQuestLogProps) {
  return (
    <View class={`relative grow shrink basis-0 min-h-0 nine-panel-cut bg-glass text-fg w-full p-4 gap-2 disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title}>
      <View class={`flex-row items-center min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-2`}>
        <Text class={`grow shrink basis-0 font-display text-base font-bold leading-tight tracking-tight text-fg m-0`}>
          {props.title}
        </Text>
      </View>
      <Show when={props.quests.length} fallback={
          <View class="grow shrink basis-0 justify-center items-center w-full gap-2" role="status">
            <Icon class="size-12" name="flag" />
            <Text class="m-0">{t("rpg.quests.empty")}</Text>
          </View>
      }>
        <View class="grow shrink basis-0 py-1 gap-3 overflow-auto">
          <For each={props.groups}>
            {(g) => (
            <View class="w-full gap-0.5" label={g().label} data-done={g().done}>
              <View class={`group/heading flex-row items-center tracking-xs text-fg-muted pt-1 pb-0.5 px-2 m-0 gap-1`} role="heading" data-done={g().done}>
                <View class="w-2 h-0.75 bg-gold data-done:bg-none data-done:bg-fg-subtle" data-done={g().done} />
                <Text class={`font-semibold text-caption text-gold tracking-label uppercase whitespace-nowrap text-ellipsis overflow-hidden group-data-done/heading:text-fg-subtle shrink`}>
                  {g().label}
                </Text>
              </View>
              <View class="w-full gap-0" role="listbox" label={g().label}>
                <For each={props.quests}>
                  {(q) => (
                  <Show when={q().group === g().id}>
                    <View class={`flex-row items-center justify-start gap-1 group/row text-fg transition duration-317 w-full min-h-11 pr-2 pl-3 py-0 rounded-control hover:bg-surface-hover active:bg-surface-hover ${q().id === props.selected ? packClass("Row", "frame", "selected") : packClass("Row", "frame")} data-selected:text-fg data-selected:inset-shadow-bar data-selected:inset-shadow-hud-accent disabled:bg-transparent disabled:bg-none disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} role="option" data-fresh={q().fresh} selected={q().id === props.selected} label={q().name} onPress={(e) => props.onChoose?.(e, q().id)} onAction={{ details: (e) => props.onTrack?.(e, q().id) }} data-tracked={q().tracked} data-done={g().done} data-selected={q().id === props.selected}>
                      <View class="flex-row items-center w-full gap-3">
                        <View class={`text-fg rotate-45 inset-ring-2 inset-ring-fg-subtle mx-0.5 size-2 data-tracked:bg-gold data-tracked:bg-none data-tracked:shadow-glow data-tracked:shadow-gold data-tracked:ring-3 data-tracked:ring-glow data-done:bg-success data-done:bg-none data-done:shadow-none`} data-tracked={q().tracked} data-done={g().done} />
                        <Text class="grow shrink basis-0 text-start font-semibold whitespace-nowrap text-ellipsis overflow-hidden group-data-done/row:text-fg-subtle group-data-done/row:line-through">
                          {q().name}
                        </Text>
                        <Show when={q().fresh && !g().done}>
                          <Text class={`font-bold text-caption tracking-label uppercase whitespace-nowrap text-gold-fg bg-gold nine-chip-cut py-0 px-1 animate-pop ${packClass("Chip", "frame")}`}>
                            {t("rpg.quests.fresh")}
                          </Text>
                        </Show>
                        <Show when={q().id === props.selected}>
                          <Icon class="text-gold size-4 rtl:-scale-x-100 rtl:scale-y-100" name="chevronRight" />
                        </Show>
                      </View>
                    </View>
                  </Show>
                  )}
                </For>
              </View>
            </View>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgQuestLogProps>> = {
  default: {"title":"Quest Journal","groups":[{"id":"main","label":"Main Quests","done":false},{"id":"done","label":"Completed","done":true}],"quests":[{"id":"crown","group":"main","name":"The Hollow Crown","tracked":true,"fresh":false},{"id":"heir","group":"done","name":"The Lost Heir","tracked":false,"fresh":false}],"selected":"crown"},
  empty: {"title":"Quest Journal","groups":[],"quests":[],"selected":""},
  overflow: {"title":"Quest Journal of the Long Campaign","groups":[{"id":"main","label":"Main Quests of the Second Act","done":false},{"id":"side","label":"Side Quests","done":false},{"id":"done","label":"Completed","done":true}],"quests":[{"id":"crown","group":"main","name":"The Hollow Crown","tracked":true,"fresh":false},{"id":"gate","group":"main","name":"Ashes of the Ninefold Gate beneath the Drowned Abbey","tracked":false,"fresh":true},{"id":"bell","group":"side","name":"The Sunken Bell","tracked":false,"fresh":false},{"id":"lantern","group":"side","name":"A Lantern for the Ferryman’s Daughter","tracked":false,"fresh":true},{"id":"heir","group":"done","name":"The Lost Heir","tracked":false,"fresh":false},{"id":"mill","group":"done","name":"Wolves at the Mill","tracked":false,"fresh":false}],"selected":"crown"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"details":"Track"} as const;
