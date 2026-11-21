import { For, Image, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
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
  /** The skip event, with the row's key where it comes from a row. */
  onSkip?: (e: UiEvent, key?: string) => void;
  /** The toggleAuto event, with the row's key where it comes from a row. */
  onToggleAuto?: (e: UiEvent, key?: string) => void;
}

/**
 * A conversation across the foot of the screen: the speaker's portrait,
 * name and line, and the player's numbered replies. Reports reply (a
 * reply's key) and next (the Continue button, or confirm, while there are
 * no replies). The game decides what a press does.
 *
 * Dropped from the .svelte: the typewriter reveal (the rate prop and the
 * revealed event; the TSX primitives have no typewriter yet, so the line
 * shows whole), the name plate's two-tone gold gradient, the row
 * entries' lit gradient (a cut shape takes one bg colour), and the auto
 * key's inset gold keyline when on (its gold bar and text stay).
 */
export function RpgConversation(props: RpgConversationProps) {
  return (
    <View class={`nine-plate-cut bg-[#1e2637e5] text-fg flex-row grow shrink basis-0 flex-wrap overflow-y-auto p-4 gap-4 disabled:text-fg-subtle ${packClass("Plate", "frame")}`} label={props.speaker ? props.speaker : t("rpg.talk.narration")}>
      <View class="absolute left-4 right-[30%] top-0 h-px bg-linear-to-r from-gold/50 to-transparent pointer-events-none" />
      <Show when={props.portrait}>
        <Image class="w-[11.4rem] object-cover object-top compact:size-24 shrink-0" src={props.portrait} />
      </Show>
      <View class="w-[42%] py-3 px-0 gap-2 compact:w-full compact:grow compact:shrink compact:basis-0 compact:p-0">
        <Show when={props.speaker}>
          <Text class={`font-bold text-lg text-[#241a04] self-start max-w-full pr-4 pl-2 tracking-[0.14em] bg-gold nine-chip-cut py-0.5 m-0`}>
            {props.speaker}
          </Text>
        </Show>
        <Text class={`text-[1.571rem] font-bold text-fg m-0 compact:text-[1.286rem]`}>
          {props.line}
        </Text>
        <Show when={(props.showControls ?? false)}>
          <View class={`flex-row flex-wrap items-center gap-1`}>
            <View class={`min-h-11 shrink px-4 items-center justify-center ${packClass("Button", "plate")}`} role="button" disabled={!(props.canBacklog ?? true)} onPress={(e) => props.onBacklog?.(e)}>
              <Text>{t("rpg.talk.backlog")}</Text>
            </View>
            <View class={`min-h-11 shrink px-4 items-center justify-center ${packClass("Button", "plate")}`} role="button" disabled={!(props.canSkip ?? false)} onPress={(e) => props.onSkip?.(e)}>
              <Text>{t("rpg.talk.skip")}</Text>
            </View>
            <View class={`group/auto relative min-h-11 shrink px-4 items-center justify-center data-selected:text-gold ${packClass("Button", "plate")}`} role="button" pressed={(props.auto ?? false)} disabled={!(props.canAuto ?? true)} onPress={(e) => props.onToggleAuto?.(e)} data-selected={(props.auto ?? false)}>
              <View class="absolute left-0 inset-y-0 w-[3.424px] bg-gold hidden group-data-selected/auto:block" />
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
          <For each={props.replies} key={(r) => r.id}>
            {(r) => (
            <View class="w-full">
              <View class={`group/row relative nine-plate-cut bg-entry-hi border border-rim text-fg transition duration-317 w-full min-h-9 pointer-coarse:min-h-11 py-1 px-3 hover:bg-none hover:bg-entry-hi data-selected:bg-transparent data-selected:bg-[#38e8ff33] data-selected:text-fg disabled:text-fg-subtle data-used:text-fg-subtle data-down:opacity-45`} role="option" selected={r().id === props.selected} onPress={(e) => props.onReply?.(e, r().id)} data-used={r().used} data-selected={r().id === props.selected}>
                <View class="absolute left-0 inset-y-0 w-[3.424px] bg-[#38e8ff] hidden group-data-selected/row:block" />
                <View class="flex-row items-center w-full gap-3">
                  <Text class={`tabular-nums text-gold min-w-6 text-center font-bold bg-[#060910b8] py-0 px-0.5 nine-chip-cut group-data-selected/row:text-[#241a04] group-data-selected/row:bg-none group-data-selected/row:bg-gold group-data-used/row:text-fg-subtle ${packClass("Chip", "frame")}`}>
                    {t("rpg.talk.number", { number: r().number })}
                  </Text>
                  <Text class="grow shrink basis-0 text-start">{r().text}</Text>
                  <View class="rotate-45 bg-gold shadow-[0px_0px_0px_3.424px_#ffcf5a38,0px_0px_11.424px_#ffcf5ab2] size-2 shrink-0 invisible group-data-selected/row:visible" />
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
  default: {"speaker":"Lady Marissa","portrait":0,"line":"The road ahead is dangerous.","replies":[{"id":"r1","number":1,"text":"We have no choice.","used":false},{"id":"r2","number":2,"text":"Perhaps another way.","used":true}],"selected":"r1"},
  line: {"speaker":"","portrait":0,"line":"The wind dies.","replies":[],"selected":""},
  controls: {"speaker":"Marissa","line":"There is still time to turn back.","showControls":true,"canSkip":true,"replies":[]},
  auto: {"speaker":"","line":"The bell rings across the marsh.","showControls":true,"auto":true,"replies":[]},
  disabled: {"speaker":"","line":"Choose your answer before continuing.","showControls":true,"canBacklog":false,"canSkip":false,"canAuto":false,"replies":[]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Continue"} as const;
