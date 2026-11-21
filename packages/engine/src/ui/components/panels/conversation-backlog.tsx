import { For, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
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

export function RpgConversationBacklog(props: RpgConversationBacklogProps) {
  return (
    <View class={`border-rim bg-linear-to-b from-glass-hi via-glass via-[96px] to-glass-lo shadow-panel text-fg w-full p-4 gap-3 rounded-panel border disabled:text-fg-subtle`} label={props.title || t("rpg.talk.history")} busy={(props.loading ?? false)} onAction={{ back: (e) => props.onClose?.(e) }}>
      <View class={`flex-row flex-wrap items-center gap-2`}>
        <Text class={`grow shrink font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg m-0`}>
          {props.title || t("rpg.talk.history")}
        </Text>
        <View class={`min-h-11 shrink`} role="button" onPress={(e) => props.onClose?.(e)}>
          <Text>{t("rpg.talk.close")}</Text>
        </View>
      </View>
      <Show when={(props.loading ?? false)}>
        <View class={`flex-row items-center text-caption text-fg-muted gap-2`} role="status">
          <Spinner name={t("rpg.talk.historyLoading")} />
          <Text class="shrink">{t("rpg.talk.historyLoading")}</Text>
        </View>
      </Show>
      <View class={`grow shrink basis-0 gap-3 overflow-auto`} role="log" label={t("rpg.talk.history")}>
        <Show when={(props.hasOlder ?? false)}>
          <View class={`min-h-11 self-center`} role="button" disabled={(props.loading ?? false)} onPress={(e) => props.onOlder?.(e)}>
            <Text>{t("rpg.talk.older")}</Text>
          </View>
        </Show>
        <For each={(props.lines ?? [])}>
          {(entry) => (
          <View class={`rounded-tl-[11.424px] rounded-tr-none rounded-br-[11.424px] rounded-bl-none text-fg transition duration-317 shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_1px_0px_#ffffff09] py-2 px-3 gap-2 data-playing:bg-transparent data-playing:shadow-[inset_0px_0px_0px_1px_#ffcf5a73,inset_3.424px_0px_0px_#ffcf5a] data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} data-selected={entry().id === props.playing} data-playing={entry().id === props.playing}>
            <Text class={`font-bold text-caption text-gold tracking-[0.14em] m-0 data-narration:text-fg-muted`} data-narration={!entry().speaker}>
              {entry().speaker || t("rpg.talk.narration")}
            </Text>
            <Text class={`text-fg m-0 data-narration:text-fg-muted data-narration:italic`} data-narration={!entry().speaker}>
              {entry().text}
            </Text>
            <Show when={entry().id === props.playing} fallback={
                <Show when={entry().canReplay}>
                  <View class={`min-h-11 self-start`} role="button" disabled={(props.loading ?? false)} onPress={(e) => props.onReplay?.(e, entry().id)}>
                    <Text>{t("rpg.talk.replay", { speaker: entry().speaker || t("rpg.talk.narration") })}</Text>
                  </View>
                </Show>
            }>
              <View class={`min-h-11 self-start`} role="button" onPress={(e) => props.onStop?.(e, entry().id)}>
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
