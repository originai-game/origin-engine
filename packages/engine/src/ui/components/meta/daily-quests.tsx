import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import { ProgressBar } from "../hud/progress-bar";
import { Spinner } from "../base/spinner";
import "./general.en.ogt";

export interface DailyQuestsProps {
  /** The list's heading; empty takes the group's (Daily quests). */
  title: string;
  /** When the quests refresh, as the game words it (Resets in 4h 12m); empty hides it. The game counts it down. */
  refresh: string;
  /** Today's quests in the game's order: keyed rows of { id, name, icon, progress, progressText, reward, rewardIcon, state }: progress 0 to 1 and progressText (3 / 5); reward the prize in words (50) with its icon; state active, done (its reward waiting) or claimed. */
  quests: ReadonlyArray<{ id: string; name: string; icon: string; progress: number; progressText: string; reward: string; rewardIcon: string; state: string }>;
  /** Today's quests are on their way: a spinner stands in. */
  loading?: boolean;
  /** The claim event, with the row's key where it comes from a row. */
  onClaim?: (e: UiEvent, key?: string) => void;
}

/**
 * Today's quests: a band with the title and when they refresh (the game
 * counts it down), then one entry per quest: its icon in an octagon socket,
 * its name over a leaning bar with the count beside it, and its prize in a
 * dark box at the end. A done quest is lit gold, its prize box glows and
 * a gold Claim stands beside it; a claimed one dims behind a Claimed stamp.
 * On a narrow list the socket goes and the prize and Claim take a line of
 * their own. The game owns progress, rewards and the clock; the list
 * reports claim (a quest's key).
 */
export function DailyQuests(props: DailyQuestsProps) {
  return (
    <View class={`grow shrink basis-0 border-rim bg-linear-to-b from-glass-hi to-glass-lo shadow-panel text-fg p-4 gap-2 rounded-panel border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title !== '' ? props.title : t("general.quests.name")} busy={props.loading}>
      <View class={`flex-row flex-wrap items-center min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-3`}>
        <Text class={`grow shrink font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg m-0`}>
          {props.title !== '' ? props.title : t("general.quests.name")}
        </Text>
        <Show when={props.refresh}>
          <View class={`flex-row items-center tracking-xs text-fg-muted max-w-full bg-well/82 border border-white/8 nine-plate-cut py-0.5 px-2 gap-1 shrink`} role="timer" label={props.refresh}>
            <Icon class="size-4" name="clock" />
            <Text class="text-end text-fg shrink">{props.refresh}</Text>
          </View>
        </Show>
      </View>
      <Show when={props.loading} fallback={
          <Show when={props.quests.length} fallback={
              <View class="grow shrink basis-0 justify-center items-center w-full gap-2" role="status">
                <Icon class="size-12" name="flag" />
                <Text class="m-0">{t("general.quests.empty")}</Text>
              </View>
          }>
            <Scroll class="grow shrink basis-0 gap-2">
              <View class="w-full pt-0 pb-1 px-0 m-0 gap-1" label={props.title !== '' ? props.title : t("general.quests.name")}>
                <For each={props.quests}>
                  {(q) => (
                  <View class={`group/row flex-row items-center nine-plate-cut text-fg transition duration-317 border border-rim w-full pr-3 pl-2 py-2 gap-3 compact:gap-y-1 compact:flex-wrap compact:p-2 hover:bg-transparent data-state-done:bg-transparent data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} label={q().name} data-state-claimed={(q().state) === 'claimed'} data-state-done={(q().state) === 'done'}>
                    <View class={`group/well relative nine-socket gap-2 size-16 compact:hidden group-data-state-claimed/row:opacity-66`} data-state-claimed={(q().state) === 'claimed'} data-state-done={(q().state) === 'done'}>
                      <View class="absolute inset-0 nine-socket-rim bg-white/40 group-data-state-done/well:bg-gold group-data-state-claimed/well:bg-white/15" />
                      <View class="absolute justify-center items-center text-fg inset-0 gap-2 group-data-state-done/well:text-gold">
                        <Icon class="size-6" name={q().state === 'claimed' ? 'check' : q().icon} />
                      </View>
                    </View>
                    <View class="grow shrink basis-0 gap-1 group-data-state-claimed/row:opacity-66">
                      <Text class={`text-fg text-start font-semibold text-lg m-0`}>
                        {q().name}
                      </Text>
                      <View class="group/progress flex-row items-center w-full gap-1" data-state-claimed={(q().state) === 'claimed'} data-state-done={(q().state) === 'done'}>
                        <View class="grow shrink basis-0 min-w-0">
                          <ProgressBar value={q().progress} valueText={q().progressText} tone={q().state === 'claimed' ? 'success' : ''} />
                        </View>
                        <Text class="shrink-0 whitespace-nowrap font-bold tabular-nums text-fg group-data-state-done/progress:text-gold">
                          {q().progressText}
                        </Text>
                      </View>
                    </View>
                    <View class="flex-row items-center max-w-[45%] gap-1 compact:max-w-full compact:w-full compact:justify-between">
                      <View class={`text-fg min-w-[89.6px] max-w-full min-h-16 items-center justify-center bg-well/82 border border-white/8 nine-plate-cut py-0.5 px-2 gap-0 compact:flex-row compact:gap-x-1 compact:min-h-0 compact:py-0.5 compact:px-3 data-state-done:text-gold data-state-done:bg-transparent group-data-state-claimed/row:opacity-66`} label={t("general.quests.reward", { reward: q().reward })} data-state-claimed={(q().state) === 'claimed'} data-state-done={(q().state) === 'done'}>
                        <Icon class="size-6" name={q().rewardIcon} />
                        <Text class="text-center text-xl font-bold tabular-nums">{q().reward}</Text>
                      </View>
                      <Show when={q().state === 'done'}>
                        <Button class={`text-accent min-h-11 animate-pop ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onClaim?.(e, q().id)}>
                          <Text>{t("general.quests.claim")}</Text>
                        </Button>
                      </Show>
                      <Show when={q().state === 'claimed'}>
                        <View class={`flex-row items-center font-bold tracking-[0.14em] uppercase text-success gap-0.5 animate-stamp`}>
                          <Icon class="size-4" name="check" />
                          <Text class="whitespace-nowrap shrink">{t("general.quests.claimed")}</Text>
                        </View>
                      </Show>
                    </View>
                  </View>
                  )}
                </For>
              </View>
            </Scroll>
          </Show>
      }>
        <View class="grow shrink basis-0 justify-center items-center w-full gap-2" role="status">
          <Spinner name={t("general.quests.loading")} />
          <Text class="m-0">{t("general.quests.loading")}</Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<DailyQuestsProps>> = {
  default: {"title":"","refresh":"Resets in 4h 12m","quests":[{"id":"win","name":"Win 3 matches","icon":"trophy","progress":0.67,"progressText":"2 / 3","reward":"50","rewardIcon":"gem","state":"active"},{"id":"gather","name":"Gather 20 herbs","icon":"leaf","progress":1,"progressText":"20 / 20","reward":"500","rewardIcon":"coin","state":"done"},{"id":"login","name":"Log in","icon":"check","progress":1,"progressText":"1 / 1","reward":"100","rewardIcon":"coin","state":"claimed"}],"loading":false},
  empty: {"title":"","refresh":"Resets in 23h 59m","quests":[],"loading":false},
  loading: {"title":"","refresh":"","quests":[],"loading":true},
  overflow: {"title":"Daily Challenges of the Adventurers Guild","refresh":"Resets in 0h 01m, hurry before the next rotation","quests":[{"id":"q1","name":"Defeat twenty-five elite guardians in the Sunken Cathedral without healing","icon":"sword","progress":0.2,"progressText":"5 / 25","reward":"1,000,000 gold and a chest","rewardIcon":"coin","state":"active"},{"id":"q2","name":"Craft","icon":"gear","progress":0,"progressText":"0 / 1","reward":"5","rewardIcon":"gem","state":"active"},{"id":"q3","name":"Fish","icon":"drop","progress":1,"progressText":"10 / 10","reward":"20","rewardIcon":"gem","state":"done"},{"id":"q4","name":"Explore","icon":"map","progress":0.5,"progressText":"1 / 2","reward":"200","rewardIcon":"coin","state":"active"},{"id":"q5","name":"Trade","icon":"bag","progress":1,"progressText":"3 / 3","reward":"30","rewardIcon":"gem","state":"claimed"}],"loading":false},
};
