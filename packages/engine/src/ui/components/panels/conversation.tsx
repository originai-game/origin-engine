import { For, Image, Show, Text, View, createEffect, createMemo, packClass, t, untrack, useClock, useWorld, type UiEvent } from "@origingame/engine/ui";
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
  /** Characters of the line shown per second, by the world's tick, starting again with each new line; 0 (and when left out) shows the line at once, and setting 0 finishes a line early. A screen reader always reads the whole line. */
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
  /** The skip event, with the row's key where it comes from a row. */
  onSkip?: (e: UiEvent, key?: string) => void;
  /** The toggleAuto event, with the row's key where it comes from a row. */
  onToggleAuto?: (e: UiEvent, key?: string) => void;
  /** The revealed event: the line is whole, once per line that was revealed at a rate. */
  onRevealed?: (e: UiEvent) => void;
}

/**
 * A conversation across the foot of the screen: the speaker's portrait,
 * name and line, and the player's numbered replies. Reports reply (a
 * reply's key) and next (the Continue button, or confirm, while there are
 * no replies). With a rate the line shows a few characters at a time by
 * the world's tick (always whole to a screen reader) and reports revealed
 * once it is whole; the game decides what a press before that does.
 *
 * Dropped from the .svelte: the whole line at once under reduced motion
 * (the TSX UI cannot read the setting yet), the name plate's two-tone gold gradient, the row
 * entries' lit gradient (a cut shape takes one bg colour), and the auto
 * key's inset gold keyline when on (its gold bar and text stay).
 */
export function RpgConversation(props: RpgConversationProps) {
  const clock = useClock();
  const world = useWorld();
  // The tick the line started on: a new line starts its reveal again.
  let revealedStart = -1;
  let revealedLine = "";
  const start = createMemo(() => {
    const line = props.line;
    const tick = untrack(clock).tick;
    if (line !== revealedLine) {
      revealedLine = "";
      revealedStart = -1;
    }
    return { line, tick };
  });
  const shown = createMemo(() => {
    const rate = props.rate ?? 0;
    const { line, tick } = start();
    if (rate <= 0 || (revealedLine === line && revealedStart === tick)) return line;
    const chars = Array.from(line);
    const c = clock();
    const n = Math.floor((c.tick - tick) * c.dt * rate);
    if (n >= chars.length) {
      revealedLine = line;
      revealedStart = tick;
      return line;
    }
    return chars.slice(0, Math.max(n, 0)).join("");
  });
  // Revealed once a line shown at a rate is whole.
  let told = "";
  createEffect(() => {
    const line = props.line;
    if ((props.rate ?? 0) > 0 && shown() === line && told !== line) {
      told = line;
      untrack(() => props.onRevealed?.({ world, type: "revealed", x: 0, y: 0, key: "", value: "" }));
    }
  });
  return (
    <View class={`flex-row grow shrink basis-0 flex-wrap overflow-y-auto p-4 gap-4 ${packClass("Plate", "frame")} disabled:text-fg-subtle`} label={props.speaker ? props.speaker : t("rpg.talk.narration")}>
      <View class="absolute left-4 right-3/10 top-0 h-px bg-linear-to-r from-gold to-transparent pointer-events-none" />
      <Show when={props.portrait}>
        <Image class="w-40 object-cover object-top compact:size-24 shrink-0" src={props.portrait} />
      </Show>
      <View class="w-21/50 py-3 px-0 gap-2 compact:w-full compact:grow compact:shrink compact:basis-0 compact:p-0">
        <Show when={props.speaker}>
          <Text class={`font-bold text-lg text-gold-fg self-start max-w-full pr-4 pl-2 tracking-label bg-gold rounded-control py-0.5 m-0`}>
            {props.speaker}
          </Text>
        </Show>
        <Text class={`text-2xl font-bold m-0 compact:text-lg`} label={props.line}>
          {shown()}
        </Text>
        <Show when={(props.showControls ?? false)}>
          <View class={`flex-row flex-wrap items-center gap-1`}>
            <View class={`flex-row gap-1 min-h-11 shrink px-4 items-center justify-center ${packClass("Button", "plate")}`} role="button" disabled={!(props.canBacklog ?? true)} onPress={(e) => props.onBacklog?.(e)}>
              <Text>{t("rpg.talk.backlog")}</Text>
            </View>
            <View class={`flex-row gap-1 min-h-11 shrink px-4 items-center justify-center ${packClass("Button", "plate")}`} role="button" disabled={!(props.canSkip ?? false)} onPress={(e) => props.onSkip?.(e)}>
              <Text>{t("rpg.talk.skip")}</Text>
            </View>
            <View class={`flex-row gap-1 group/auto relative min-h-11 shrink px-4 items-center justify-center data-selected:text-gold ${packClass("Button", "plate")}`} role="button" pressed={(props.auto ?? false)} disabled={!(props.canAuto ?? true)} onPress={(e) => props.onToggleAuto?.(e)} data-selected={(props.auto ?? false)}>
              <View class="absolute left-0 inset-y-0 w-0.75 bg-gold hidden group-data-selected/auto:block" />
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
              <View class={`flex-row items-center justify-start gap-1 group/row relative text-fg transition duration-control w-full min-h-9 pointer-coarse:min-h-11 py-1 px-3 ${r().id === props.selected ? packClass("Row", "frame", "selected") : packClass("Row", "frame")} hover:bg-surface-hover disabled:text-fg-subtle data-used:text-fg-subtle data-down:opacity-dim`} role="option" selected={r().id === props.selected} onPress={(e) => props.onReply?.(e, r().id)} data-used={r().used} data-selected={r().id === props.selected}>
                <View class="absolute left-0 inset-y-0 w-0.75 bg-hud-accent hidden group-data-selected/row:block" />
                <View class="flex-row items-center w-full gap-3">
                  <Text class={`tabular-nums min-w-6 text-center font-bold py-0 px-0.5 ${r().id === props.selected ? "rounded-control bg-gold text-gold-fg shadow-none":`${packClass("Chip", "frame")} text-gold group-data-used/row:text-fg-subtle`}`}>
                    {t("rpg.talk.number", { number: r().number })}
                  </Text>
                  <Text class="grow shrink basis-0 text-start">{r().text}</Text>
                  <View class="rotate-45 bg-gold shadow-glow shadow-gold ring-thick ring-glow size-2 shrink-0 invisible group-data-selected/row:visible" />
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
  typing: {"speaker":"Marissa","line":"The lanterns went out one by one.","rate":30,"replies":[]},
  auto: {"speaker":"","line":"The bell rings across the marsh.","showControls":true,"auto":true,"replies":[]},
  disabled: {"speaker":"","line":"Choose your answer before continuing.","showControls":true,"canBacklog":false,"canSkip":false,"canAuto":false,"replies":[]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Continue"} as const;
