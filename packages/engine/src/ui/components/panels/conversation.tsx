import { For, Image, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import { Button } from "../../controls/button";
import "./conversation.en.ogt";

export interface RpgConversationProps {
  /** The speaker's name; empty is narration. */
  speaker: string;
  /** The speaker's portrait (og.load.texture); 0 hides it. */
  portrait: Texture | 0;
  /** What is said now. */
  line: string;
  /** Characters of the line shown per second, starting again with each new line; 0 (and when left out) shows the line at once, and setting 0 finishes a line early. Default 0. */
  rate?: number;
  /** The player's replies, keyed rows of { id, number, text, used }: number the figure before it; used greys a reply already chosen. Empty shows a Continue button. */
  replies: ReadonlyArray<{ id: string; number: number; text: string; used: boolean }>;
  /** The reply the focus rests on; empty is none. */
  selected: string;
  /** Shows optional History, Skip and Auto buttons; hidden in existing compact layouts. Default false. */
  showControls?: boolean;
  /** The game permits opening conversation history. Default true. */
  canBacklog?: boolean;
  /** The game permits requesting a skip; the component never advances dialogue itself. Default false. */
  canSkip?: boolean;
  /** The game permits requesting an automatic-advance setting change. Default true. */
  canAuto?: boolean;
  /** The game's automatic-advance setting; toggleAuto requests a change without advancing dialogue. Default false. */
  auto?: boolean;
  /** The backlog event, with the row's key where it comes from a row. */
  onBacklog?: (e: UiEvent, key?: string) => void;
  /** The next event, with the row's key where it comes from a row. */
  onNext?: (e: UiEvent, key?: string) => void;
  /** The reply event, with the row's key where it comes from a row. */
  onReply?: (e: UiEvent, key?: string) => void;
  /** The revealed event, with the row's key where it comes from a row. */
  onRevealed?: (e: UiEvent, key?: string) => void;
  /** The skip event, with the row's key where it comes from a row. */
  onSkip?: (e: UiEvent, key?: string) => void;
  /** The toggleAuto event, with the row's key where it comes from a row. */
  onToggleAuto?: (e: UiEvent, key?: string) => void;
}

/**
 * A conversation across the foot of the screen: the speaker's portrait,
 * name and line, and the player's numbered replies. Reports reply (a
 * reply's key) and next (the Continue button, or confirm, while there are
 * no replies). With a rate the line shows a few characters at a time
 * (whole at once under reduced motion, and always whole to a screen
 * reader) and reports revealed once it is whole; the game decides what a
 * press before that does.
 */
export function RpgConversation(props: RpgConversationProps) {
  return (
    <View class={`flex-row grow shrink basis-0 border-rim rounded-tl-[11.424px] rounded-tr-none rounded-br-[11.424px] rounded-bl-none bg-linear-to-b from-[#131925eb] to-[#0b0f18f7] shadow-[inset_0px_0px_0px_1px_#ffffff1f,inset_0px_1px_0px_#ffcf5a80] text-fg flex-wrap overflow-y-auto p-4 gap-4 border disabled:text-fg-subtle`} label={props.speaker ? props.speaker : t("rpg.talk.narration")}>
      <Show when={props.portrait}>
        <Image class="w-[182.4px] object-cover object-top compact:size-24" src={props.portrait} />
      </Show>
      <View class="w-[42%] py-3 px-0 gap-2 compact:w-full compact:grow compact:shrink compact:basis-0 compact:p-0">
        <Show when={props.speaker}>
          <Text class={`font-bold text-lg text-[#241a04] self-start max-w-full pr-4 pl-2 tracking-[0.14em] bg-linear-to-b from-[#ffe9a6] to-gold py-0.5 m-0`}>
            {props.speaker}
          </Text>
        </Show>
        <View class="text-[1.571rem] font-bold text-fg m-0 compact:text-[1.286rem]" rate={(props.rate ?? 0)} onAction={{ revealed: (e) => props.onRevealed?.(e) }}>
          <Text>{props.line}</Text>
        </View>
        <Show when={(props.showControls ?? false)}>
          <View class={`flex-row flex-wrap items-center gap-1`}>
            <View class={`min-h-11 shrink`} role="button" disabled={!(props.canBacklog ?? true)} onPress={(e) => props.onBacklog?.(e)}>
              <Text>{t("rpg.talk.backlog")}</Text>
            </View>
            <View class={`min-h-11 shrink`} role="button" disabled={!(props.canSkip ?? false)} onPress={(e) => props.onSkip?.(e)}>
              <Text>{t("rpg.talk.skip")}</Text>
            </View>
            <View class={`min-h-11 data-selected:text-gold data-selected:shadow-[inset_0px_0px_0px_1px_#ffcf5a73,inset_3.424px_0px_0px_#ffcf5a] shrink`} role="button" pressed={(props.auto ?? false)} disabled={!(props.canAuto ?? true)} onPress={(e) => props.onToggleAuto?.(e)} data-selected={(props.auto ?? false)}>
              <Text>{t("rpg.talk.auto")}</Text>
            </View>
          </View>
        </Show>
      </View>
      <Show when={props.replies.length} fallback={
          <View class="grow shrink basis-0 items-end justify-end p-3 gap-2 compact:basis-full compact:p-0">
            <Button onPress={(e) => props.onNext?.(e)} onAction={{ confirm: (e) => props.onNext?.(e) }}>
              <Text>{t("rpg.talk.continue")}</Text>
            </Button>
          </View>
      }>
        <View class="grow shrink basis-0 justify-center py-3 px-0 m-0 gap-0.5 compact:basis-full compact:p-0" role="listbox" label={t("rpg.talk.replies")}>
          <For each={props.replies}>
            {(r) => (
            <View class="w-full">
              <View class={`group/row rounded-tl-[11.424px] rounded-tr-none rounded-br-[11.424px] rounded-bl-none bg-linear-to-r from-entry-hi to-entry-lo text-fg transition duration-317 shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_1px_0px_#ffffff09] w-full min-h-11 py-1 px-3 hover:bg-transparent data-selected:bg-transparent data-selected:bg-linear-to-r data-selected:from-[#38e8ff3d] data-selected:via-[#38e8ff14] data-selected:via-55% data-selected:to-[#38e8ff08] data-selected:text-fg data-selected:shadow-[inset_3.424px_0px_0px_#38e8ff,inset_0px_1px_0px_#ffffff2e] disabled:text-fg-subtle data-done:text-fg-subtle data-used:text-fg-subtle data-down:opacity-45`} role="option" data-done={r().used} selected={r().id === props.selected} onPress={(e) => props.onReply?.(e, r().id)} data-used={r().used} data-selected={r().id === props.selected}>
                <View class="flex-row items-center w-full gap-3">
                  <Text class={`tabular-nums text-gold min-w-6 text-center font-bold bg-[#0609108c] py-0 px-0.5 group-data-selected/row:text-[#241a04] group-data-selected/row:bg-none group-data-selected/row:bg-gold group-data-used/row:text-fg-subtle`}>
                    {t("rpg.talk.number", { number: r().number })}
                  </Text>
                  <Text class="grow shrink basis-0 text-start">{r().text}</Text>
                  <View class="rotate-45 bg-gold shadow-[0px_0px_0px_3.424px_#ffcf5a38,0px_0px_11.424px_#ffcf5ab2] size-2 group-data-selected/row:visible" />
                </View>
              </View>
            </View>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgConversationProps>> = {
  default: {"speaker":"Lady Marissa","portrait":0,"line":"The road ahead is dangerous.","rate":0,"replies":[{"id":"r1","number":1,"text":"We have no choice.","used":false},{"id":"r2","number":2,"text":"Perhaps another way.","used":true}],"selected":"r1"},
  line: {"speaker":"","portrait":0,"line":"The wind dies.","replies":[],"selected":""},
  controls: {"speaker":"Marissa","line":"There is still time to turn back.","showControls":true,"canSkip":true,"replies":[]},
  auto: {"speaker":"","line":"The bell rings across the marsh.","showControls":true,"auto":true,"replies":[]},
  disabled: {"speaker":"","line":"Choose your answer before continuing.","showControls":true,"canBacklog":false,"canSkip":false,"canAuto":false,"replies":[]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Continue"} as const;
