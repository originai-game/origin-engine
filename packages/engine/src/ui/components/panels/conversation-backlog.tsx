import { For, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import { Spinner } from "../base/spinner";
import "./dialogue.en.ogt";

export interface RpgConversationBacklogProps {
  /** The history title; empty uses Conversation history. */
  title?: string;
  /** Chronological keyed { id, speaker, text, canReplay } entries. Empty speaker is narration. The game supplies history and audio availability; lines keep their original wording. Default []. */
  lines?: ReadonlyArray<{ id: string; speaker: string; text: string; canReplay: boolean }>;
  /** The entry currently replaying; empty means no replay. The game owns playback and reports completion by clearing this id. */
  playing?: string;
  /** Earlier history is available; older asks the game to prepend it, preserving row ids. Default false. */
  hasOlder?: boolean;
  /** History is loading; earlier-line and replay actions are disabled, close remains available. Default false. */
  loading?: boolean;
  /** The close event, with the row's key where it comes from a row. */
  onClose?: (e: UiEvent, key?: string) => void;
  /** The older event, with the row's key where it comes from a row. */
  onOlder?: (e: UiEvent, key?: string) => void;
  /** The replay event, with the row's key where it comes from a row. */
  onReplay?: (e: UiEvent, key?: string) => void;
  /** The stop event, with the row's key where it comes from a row. */
  onStop?: (e: UiEvent, key?: string) => void;
}

/**
 * The conversation's history: the title and Close, then the lines as cut
 * cards — the speaker in gold small capitals, narration quiet and in
 * italics, the line replaying washed gold with a gold bar — with Load
 * earlier lines over them and Replay or Stop replay under a replayable
 * one. Reports close, older, replay and stop (an entry's key). Answers
 * the back action.
 *
 * Dropped from the .svelte: the rows' lit gradient (a cut shape takes one
 * bg colour).
 */
export function RpgConversationBacklog(props: RpgConversationBacklogProps) {
  return (
    <View class={`border-rim bg-linear-to-b from-glass-hi to-glass-lo shadow-panel text-fg w-full p-4 gap-3 rounded-panel border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title || t("rpg.talk.history")} busy={(props.loading ?? false)} onAction={{ back: (e) => props.onClose?.(e) }}>
      <View class={`flex-row flex-wrap items-center gap-2`}>
        <Text class={`grow shrink font-display text-base font-bold leading-tight tracking-tight text-fg m-0`}>
          {props.title || t("rpg.talk.history")}
        </Text>
        <View class={`flex-row gap-1 min-h-11 shrink px-4 items-center justify-center ${packClass("Button", "plate")}`} role="button" onPress={(e) => props.onClose?.(e)}>
          <Text>{t("rpg.talk.close")}</Text>
        </View>
      </View>
      <Show when={(props.loading ?? false)}>
        <View class={`flex-row items-center text-caption text-fg-muted gap-2`} role="status">
          <Spinner name={t("rpg.talk.historyLoading")} />
          <Text class="shrink">{t("rpg.talk.historyLoading")}</Text>
        </View>
      </Show>
      <View class={`grow shrink gap-3 overflow-auto`} role="log" label={t("rpg.talk.history")}>
        <Show when={(props.hasOlder ?? false)}>
          <View class={`flex-row gap-1 min-h-11 self-center px-4 items-center justify-center ${packClass("Button", "plate")}`} role="button" disabled={(props.loading ?? false)} onPress={(e) => props.onOlder?.(e)}>
            <Text>{t("rpg.talk.older")}</Text>
          </View>
        </Show>
        <For each={(props.lines ?? [])} key={(entry) => entry.id}>
          {(entry) => (
          <View class={`group/row relative nine-plate-cut text-fg transition duration-control py-2 px-3 gap-2 ${packClass("Row", "frame")} hover:bg-surface-hover data-playing:bg-surface-hover disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-dim`} data-playing={entry().id === props.playing}>
            <View class="absolute left-0 inset-y-0 w-0.75 bg-gold hidden group-data-playing/row:block" />
            <Text class={`font-bold text-caption text-gold tracking-label uppercase m-0 data-narration:text-fg-muted data-narration:normal-case`} data-narration={!entry().speaker}>
              {entry().speaker || t("rpg.talk.narration")}
            </Text>
            <Text class={`text-fg m-0 data-narration:text-fg-muted data-narration:italic`} data-narration={!entry().speaker}>
              {entry().text}
            </Text>
            <Show when={entry().id === props.playing} fallback={
                <Show when={entry().canReplay}>
                  <View class={`flex-row gap-1 min-h-11 self-start px-4 items-center justify-center ${packClass("Button", "plate")}`} role="button" disabled={(props.loading ?? false)} onPress={(e) => props.onReplay?.(e, entry().id)}>
                    <Text>{t("rpg.talk.replay", { speaker: entry().speaker || t("rpg.talk.narration") })}</Text>
                  </View>
                </Show>
            }>
              <View class={`flex-row gap-1 min-h-11 self-start px-4 items-center justify-center ${packClass("Button", "plate")}`} role="button" onPress={(e) => props.onStop?.(e, entry().id)}>
                <Text>{t("rpg.talk.stop")}</Text>
              </View>
            </Show>
          </View>
          )}
        </For>
        <Show when={!(props.lines ?? []).length && !(props.loading ?? false)}>
          <Text class={`text-caption text-fg-muted text-center p-4 m-0`} role="status">
            {t("rpg.talk.historyEmpty")}
          </Text>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgConversationBacklogProps>> = {
  default: {"title":"","lines":[{"id":"a","speaker":"Lady Marissa","text":"The road ahead is dangerous.","canReplay":true},{"id":"b","speaker":"Brenna","text":"We have no choice.","canReplay":false},{"id":"c","speaker":"","text":"The wind dies.","canReplay":false}],"playing":"","hasOlder":true,"loading":false},
  empty: {},
  loading: {"loading":true},
  playing: {"playing":"a","lines":[{"id":"a","speaker":"Lady Marissa","text":"The road ahead is dangerous.","canReplay":true}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"back":"Close history"} as const;
