import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import "./general.en.ogt";

export interface DailyRewardsProps {
  /** The calendar's heading; empty takes the group's (Daily rewards). */
  title: string;
  /** When the next day's reward opens, as the game words it; empty hides it. */
  refresh: string;
  /** The calendar's days in order: keyed rows of { id, label, icon, amount, state, milestone }: label the day in words (Day 1); amount the reward in words (x100); state claimed, today, upcoming or missed, each shown by its own word: the game sends claimed for today once its reward is taken; milestone a big reward, lit violet with a ribbon. */
  days: ReadonlyArray<{ id: string; label: string; icon: string; amount: string; state: string; milestone: boolean }>;
  /** Today's reward is waiting: the day whose state is today is a Claim button; otherwise no day is a button. */
  canClaim?: boolean;
  /** Its place is narrow (under about 300 px): the days (their state words go) tighten. The place decides, not the screen. */
  compact?: boolean;
  /** The claim event, with the row's key where it comes from a row. */
  onClaim?: (e: UiEvent, key?: string) => void;
}

/**
 * A login reward calendar: a grid of day tiles with two corners cut, each a
 * header strip with its day, the reward's icon large in the middle, its
 * amount, and a foot with its state in words and a mark. Today is lit gold
 * (a gold header and foot, a glow round the tile); a day taken carries a
 * check stamp over its dimmed art; a missed day is hatched with its word in
 * danger; a milestone is violet with a ribbon. The game owns the schedule,
 * the rewards and the clock; the calendar reports claim (today's key) while
 * canClaim holds: only then is today's tile a button; every other day is a
 * labelled tile that takes no focus and reports nothing. A day shows the
 * word of the state the game gave it. Seven across on a desktop, four on a
 * narrow panel, three on a phone, where a last day alone on its row takes
 * the whole row.
 */
export function DailyRewards(props: DailyRewardsProps) {
  return (
    <View class={`group/dailyrewards grow shrink basis-0 border-rim bg-linear-to-b from-glass-hi to-glass-lo shadow-panel text-fg p-4 gap-2 rounded-panel border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title !== '' ? props.title : t("general.rewards.name")} data-narrow={props.compact === true}>
      <View class={`flex-row flex-wrap items-center min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-3`}>
        <Text class={`grow shrink font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg m-0`}>
          {props.title !== '' ? props.title : t("general.rewards.name")}
        </Text>
        <Show when={props.refresh}>
          <View class={`flex-row items-center tracking-xs text-fg-muted max-w-full bg-[#060910b8] border border-white/8 nine-plate-cut py-0.5 px-2 gap-1 shrink`} role="timer" label={props.refresh}>
            <Icon class="size-4" name="clock" />
            <Text class="text-end text-fg shrink">{props.refresh}</Text>
          </View>
        </Show>
      </View>
      <Scroll class="grow shrink basis-0 gap-2">
        <View class="grid grid-cols-7 w-full pt-0.5 pb-1 px-0 m-0 compact:grid-cols-3">
          <For each={props.days}>
            {(d, i) => (
            <View class="flex-row w-full gap-0 compact:data-alone:col-span-full" data-alone={i() % 3 === 0 && i() === props.days.length - 1}>
              <Show when={d().state === 'today' && props.canClaim} fallback={
                  <View class={`group/cell flex-row nine-plate-cut bg-[#1d2434f2] border border-rim text-fg transition duration-317 w-full min-h-[172.8px] p-0 gap-0 data-selected:border-accent data-done:border-success data-active:border-accent data-milestone:bg-[#b48cff38] data-milestone:to-60% data-milestone:bg-[#221e34f5] data-state-missed:bg-[#151b26f2] data-state-today:bg-[#28241ef5] disabled:text-fg-subtle disabled:scale-100 active:scale-96 data-locked:scale-100 data-down:opacity-45 compact:min-h-[153.6px] shrink`} role="group" current={d().state === 'today'} label={t("general.rewards.day", { day: d().label, reward: d().amount, state: d().state === 'claimed' ? t("general.rewards.claimed") : d().state === 'missed' ? t("general.rewards.missed") : d().state === 'today' ? t("general.rewards.today") : t("general.rewards.upcoming") })} data-state-claimed={(d().state) === 'claimed'} data-state-missed={(d().state) === 'missed'} data-state-today={(d().state) === 'today'} data-milestone={d().milestone}>
                    <View class="w-full gap-0 shrink">
                      <Text class={`tracking-[0.14em] uppercase text-fg-muted text-center font-bold text-caption bg-[#06091066] p-0.5 m-0 group-data-milestone/cell:text-[#d9c6fe] group-data-milestone/cell:bg-none group-data-milestone/cell:bg-[#b48cff47] group-data-state-today/cell:text-[#241a04] group-data-state-today/cell:bg-none group-data-state-today/cell:bg-gold`}>
                        {d().label}
                      </Text>
                      <View class="grow shrink basis-0 min-h-16 w-full gap-2">
                        <View class="absolute justify-center items-center w-full inset-0 gap-2 group-data-state-claimed/cell:opacity-35 group-data-state-missed/cell:opacity-35">
                          <Icon class="text-fg size-12 compact:size-8 group-data-milestone/cell:text-[#d9c6fe] group-data-state-today/cell:text-gold" name={d().icon} />
                        </View>
                        <Show when={d().state === 'claimed'}>
                          <View class="absolute justify-center items-center bg-success text-[#0c1a12] shadow-[0px_0px_0px_3.424px_#55e39a40,0px_4.576px_16px_#00000080] -rotate-8 inset-0 gap-2 size-16 rounded-capsule compact:size-12 animate-stamp">
                            <Icon class="size-6 compact:size-4" name="check" />
                          </View>
                        </Show>
                      </View>
                      <Text class={`text-xl font-bold tabular-nums text-fg pt-0 pb-1 text-center px-0.5 m-0 group-data-state-claimed/cell:opacity-35 group-data-state-missed/cell:opacity-35`}>
                        {d().amount}
                      </Text>
                      <View class={`flex-row items-center tracking-[0.14em] uppercase text-fg-subtle w-full justify-center font-bold text-caption p-0.5 gap-0.5 compact:flex-col compact:tracking-normal group-data-state-claimed/cell:text-success group-data-state-claimed/cell:bg-none group-data-state-claimed/cell:bg-success/8 group-data-state-missed/cell:text-danger group-data-state-missed/cell:bg-none group-data-state-missed/cell:bg-danger/8 group-data-state-today/cell:text-[#241a04] group-data-state-today/cell:bg-linear-to-b group-data-state-today/cell:from-white/42 group-data-state-today/cell:to-transparent group-data-state-today/cell:to-48% group-data-state-today/cell:bg-gold`} data-state-claimed={(d().state) === 'claimed'} data-state-missed={(d().state) === 'missed'} data-state-today={(d().state) === 'today'}>
                        <Show when={d().state !== 'upcoming'}>
                          <Icon class="size-4" name={d().state === 'claimed' ? 'check' : d().state === 'missed' ? 'close' : 'star'} />
                        </Show>
                        <Text class="text-center compact:whitespace-nowrap group-data-narrow/dailyrewards:hidden shrink">
                          {d().state === 'claimed' ? t("general.rewards.claimed") : d().state === 'missed' ? t("general.rewards.missed") : d().state === 'today' ? t("general.rewards.today") : t("general.rewards.upcoming")}
                        </Text>
                      </View>
                    </View>
                  </View>
              }>
                <View class={`group/cell nine-plate-cut bg-[#1d2434f2] border border-rim text-fg transition duration-317 w-full min-h-[172.8px] p-0 data-selected:border-accent data-done:border-success data-active:border-accent data-milestone:bg-[#b48cff38] data-milestone:to-60% data-milestone:bg-[#221e34f5] data-state-missed:bg-[#151b26f2] data-state-today:bg-[#28241ef5] disabled:text-fg-subtle disabled:scale-100 active:scale-96 data-locked:scale-100 data-down:opacity-45 compact:min-h-[153.6px] data-state-today:active:bg-[#28241ef5] shrink`} role="button" current={true} label={t("general.rewards.day", { day: d().label, reward: d().amount, state: t("general.rewards.claim") })} onPress={(e) => props.onClaim?.(e, d().id)} data-state-claimed={(d().state) === 'claimed'} data-state-missed={(d().state) === 'missed'} data-state-today={(d().state) === 'today'} data-milestone={d().milestone}>
                  <View class="w-full gap-0">
                    <Text class={`tracking-[0.14em] uppercase text-fg-muted text-center font-bold text-caption bg-[#06091066] p-0.5 m-0 group-data-milestone/cell:text-[#d9c6fe] group-data-milestone/cell:bg-none group-data-milestone/cell:bg-[#b48cff47] group-data-state-today/cell:text-[#241a04] group-data-state-today/cell:bg-none group-data-state-today/cell:bg-gold`}>
                      {d().label}
                    </Text>
                    <View class="grow shrink basis-0 min-h-16 justify-center items-center w-full gap-2">
                      <Icon class="text-fg size-12 compact:size-8 group-data-milestone/cell:text-[#d9c6fe] group-data-state-today/cell:text-gold" name={d().icon} />
                    </View>
                    <Text class={`text-xl font-bold tabular-nums text-fg pt-0 pb-1 text-center px-0.5 m-0 group-data-state-claimed/cell:opacity-35 group-data-state-missed/cell:opacity-35`}>
                      {d().amount}
                    </Text>
                    <View class={`flex-row items-center tracking-[0.14em] uppercase text-fg-subtle w-full justify-center font-bold text-caption p-0.5 gap-0.5 compact:flex-col compact:tracking-normal group-data-state-claimed/cell:text-success group-data-state-claimed/cell:bg-none group-data-state-claimed/cell:bg-success/8 group-data-state-missed/cell:text-danger group-data-state-missed/cell:bg-none group-data-state-missed/cell:bg-danger/8 group-data-state-today/cell:text-[#241a04] group-data-state-today/cell:bg-linear-to-b group-data-state-today/cell:from-white/42 group-data-state-today/cell:to-transparent group-data-state-today/cell:to-48% group-data-state-today/cell:bg-gold animate-pop`} data-state-claimed={(d().state) === 'claimed'} data-state-missed={(d().state) === 'missed'} data-state-today={(d().state) === 'today'}>
                      <Icon class="size-4" name="star" />
                      <Text class="text-center compact:whitespace-nowrap group-data-narrow/dailyrewards:hidden shrink">
                        {t("general.rewards.claim")}
                      </Text>
                    </View>
                  </View>
                </View>
              </Show>
              <Show when={d().milestone}>
                <Text class="absolute right-0 top-8.5 pr-1 pl-2 font-bold text-caption tracking-[0.14em] uppercase whitespace-nowrap text-[#1b1030] bg-[#cdb4ff] pointer-events-none py-0">
                  {t("general.rewards.milestone")}
                </Text>
              </Show>
            </View>
            )}
          </For>
        </View>
      </Scroll>
    </View>
  );
}

export const fixtures: Record<string, Partial<DailyRewardsProps>> = {
  default: {"title":"","refresh":"Next reward in 9h 40m","days":[{"id":"d1","label":"Day 1","icon":"coin","amount":"x100","state":"claimed","milestone":false},{"id":"d2","label":"Day 2","icon":"potion","amount":"x3","state":"claimed","milestone":false},{"id":"d3","label":"Day 3","icon":"gem","amount":"x20","state":"missed","milestone":false},{"id":"d4","label":"Day 4","icon":"coin","amount":"x300","state":"today","milestone":false},{"id":"d5","label":"Day 5","icon":"key","amount":"x1","state":"upcoming","milestone":false},{"id":"d6","label":"Day 6","icon":"gem","amount":"x50","state":"upcoming","milestone":false},{"id":"d7","label":"Day 7","icon":"crown","amount":"Legend chest","state":"upcoming","milestone":true}],"canClaim":true},
  disabled: {"title":"","refresh":"Next reward in 23h 10m","days":[{"id":"d1","label":"Day 1","icon":"coin","amount":"x100","state":"claimed","milestone":false},{"id":"d2","label":"Day 2","icon":"potion","amount":"x3","state":"claimed","milestone":false},{"id":"d3","label":"Day 3","icon":"gem","amount":"x20","state":"claimed","milestone":true},{"id":"d4","label":"Day 4","icon":"coin","amount":"x300","state":"upcoming","milestone":false}],"canClaim":false},
  overflow: {"title":"Fourteen Days of Treasure from the Royal Vaults","refresh":"","days":[{"id":"d1","label":"Day 1","icon":"coin","amount":"x1,000,000","state":"claimed","milestone":false},{"id":"d2","label":"Day 2","icon":"potion","amount":"A very long reward name","state":"claimed","milestone":false},{"id":"d3","label":"Day 3","icon":"gem","amount":"x20","state":"claimed","milestone":false},{"id":"d4","label":"Day 4","icon":"coin","amount":"x300","state":"claimed","milestone":false},{"id":"d5","label":"Day 5","icon":"key","amount":"x1","state":"claimed","milestone":true},{"id":"d6","label":"Day 6","icon":"gem","amount":"x50","state":"missed","milestone":false},{"id":"d7","label":"Day 7","icon":"crown","amount":"x1","state":"today","milestone":false},{"id":"d8","label":"Day 8","icon":"coin","amount":"x200","state":"upcoming","milestone":false},{"id":"d9","label":"Day 9","icon":"apple","amount":"x5","state":"upcoming","milestone":false},{"id":"d10","label":"Day 10","icon":"trophy","amount":"x1","state":"upcoming","milestone":true},{"id":"d11","label":"Day 11","icon":"coin","amount":"x250","state":"upcoming","milestone":false},{"id":"d12","label":"Day 12","icon":"gem","amount":"x60","state":"upcoming","milestone":false},{"id":"d13","label":"Day 13","icon":"leaf","amount":"x9","state":"upcoming","milestone":false},{"id":"d14","label":"Day 14","icon":"crown","amount":"x1","state":"upcoming","milestone":true}],"canClaim":true},
};
