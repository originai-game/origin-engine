import {For, Icon, Show, Text, View, formatCompact, t, packClass} from "@origingame/engine/ui";
import { Counter } from "../../controls/counter";
import "./idle.en.ogt";

export interface IdleResourcesProps {
  /** The currencies in the game's order, keyed rows of { id, icon, name, amount, value, rate, perSecond, full }: value the amount as the game formats it (it wins; empty shows amount, a number, compacted: 12.3K, counting to each new amount); rate the income as the game words it (+1.2K/s; it wins; empty shows perSecond, a number per second, compacted; 0 hides it); perSecond also sets data-trend (up, down, flat) whether or not rate is given, so pass it with a worded rate too; full the store is at its cap, named Full. */
  currencies?: ReadonlyArray<{ id: string; icon: string; name: string; amount: number; value: string; rate: string; perSecond: number; full: boolean }>;
  /** The amounts are on their way: each row keeps its icon and name and shows a placeholder for its figures. */
  loading?: boolean;
}

/**
 * The resource bar of an idle game: one cut chip per currency with its art
 * in a socket, its name, the amount large and the income under it, green
 * when it rises. A store at its cap gets a gold rim and a Full tag. The
 * game owns every number: a formatted value or rate it passes wins, else
 * the bare number is compacted (12.3K) and counts to each new amount.
 * While loading each chip keeps its art and name and shows a placeholder,
 * which a pack may animate; the row's name says it is loading. It reports
 * no intents. The chips wrap onto more lines when they do not fit.
 */
export function IdleResources(props: IdleResourcesProps) {
  return (
    <View class={`flex-row items-center text-fg w-full p-0 gap-2 disabled:text-fg-subtle`} label={t("idle.resources.name")} busy={props.loading}>
      <Show when={(props.currencies ?? []).length} fallback={
          <Text class={`p-3 m-0 shrink`} role="status">
            {t("idle.resources.empty")}
          </Text>
      }>
        <View class="flex-row flex-wrap items-center w-full p-0 m-0 gap-1 shrink">
          <For each={props.currencies}>
            {(c) => (
            <View class={`flex-row items-center nine-plate-cut bg-plate-hi text-fg transition duration-317 border border-rim-soft shrink pr-2 pl-0.5 min-h-11 max-w-full py-0.5 gap-1 data-full:nine-plate-cut data-full:bg-gold data-full:bg-none data-full:text-gold-fg data-full:grow-0 data-full:shrink-0 data-full:basis-auto data-full:text-xs data-full:font-semibold data-full:tracking-label data-full:uppercase data-full:py-0 data-full:px-0.5 data-down:opacity-45 compact:grow compact:shrink compact:basis-0 compact:pr-1`} data-loading={props.loading === true} label={props.loading ? t("idle.resources.loading", { name: c().name }) : t("idle.resources.row", { name: c().name, amount: c().value ? c().value : formatCompact(c().amount) })} data-full={c().full} data-trend-down={((c().perSecond < 0 ? 'down' : c().perSecond > 0 ? 'up' : 'flat') === 'down') || ((c().perSecond < 0 ? 'down' : c().perSecond > 0 ? 'up' : 'flat') === "down")} data-trend-up={((c().perSecond < 0 ? 'down' : c().perSecond > 0 ? 'up' : 'flat') === 'up') || ((c().perSecond < 0 ? 'down' : c().perSecond > 0 ? 'up' : 'flat') === "up")} data-trend-flat={(c().perSecond < 0 ? 'down' : c().perSecond > 0 ? 'up' : 'flat') === "flat"}>
              <View class={`justify-center items-center bg-tile border-[1.5px] border-border-strong nine-plate-cut gap-2 size-12 compact:size-8 ${packClass("Slot", "frame")}`}>
                <Icon class="text-shadow-2xs size-18/25" name={c().icon} />
              </View>
              <View class="gap-0 shrink">
                <Text class={`text-fg-muted whitespace-nowrap text-ellipsis text-start text-xs font-semibold overflow-hidden`}>
                  {c().name}
                </Text>
                <Show when={!props.loading} fallback={
                    <View class={`w-16 h-2 bg-tile my-0.5`} />
                }>
                  <View class="flex-row items-center gap-1">
                    <Show when={c().value} fallback={
                        <Counter class={`text-fg text-lg font-bold shrink`} value={c().amount} format="compact" duration="300" />
                    }>
                      <Text class={`tabular-nums text-fg whitespace-nowrap text-lg font-bold shrink`}>
                        {c().value}
                      </Text>
                    </Show>
                    <Show when={c().full}>
                      <Text class={`text-shadow-2xs bg-gold nine-chip-cut whitespace-nowrap text-xs font-semibold tracking-label uppercase text-gold-fg py-0 px-0.5 ${packClass("Chip", "frame")}`}>
                        {t("idle.resources.full")}
                      </Text>
                    </Show>
                  </View>
                  <Show when={c().rate || c().perSecond !== 0}>
                    <Text class={`text-fg-muted whitespace-nowrap text-ellipsis text-xs font-bold tabular-nums text-shadow-2xs overflow-hidden data-trend-up:text-success data-trend-down:text-warn`} data-trend-down={((c().perSecond < 0 ? 'down' : c().perSecond > 0 ? 'up' : 'flat') === 'down') || ((c().perSecond < 0 ? 'down' : c().perSecond > 0 ? 'up' : 'flat') === "down")} data-trend-up={((c().perSecond < 0 ? 'down' : c().perSecond > 0 ? 'up' : 'flat') === 'up') || ((c().perSecond < 0 ? 'down' : c().perSecond > 0 ? 'up' : 'flat') === "up")} data-trend-flat={(c().perSecond < 0 ? 'down' : c().perSecond > 0 ? 'up' : 'flat') === "flat"}>
                      {c().rate ? c().rate : c().perSecond > 0 ? t("idle.resources.perSecond", { rate: formatCompact(c().perSecond) }) : t("idle.resources.perSecondDown", { rate: formatCompact(c().perSecond) })}
                    </Text>
                  </Show>
                </Show>
              </View>
            </View>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<IdleResourcesProps>> = {
  default: {"currencies":[{"id":"gold","icon":"coin","name":"Gold","amount":1234567,"value":"","rate":"","perSecond":4820,"full":false},{"id":"gems","icon":"gem","name":"Gems","amount":320,"value":"320","rate":"","perSecond":0,"full":false},{"id":"wood","icon":"tree","name":"Wood","amount":5000,"value":"5,000 / 5,000","rate":"+12/s","perSecond":12,"full":true}],"loading":false},
  loading: {"currencies":[{"id":"gold","icon":"coin","name":"Gold","amount":0,"value":"","rate":"","perSecond":0,"full":false},{"id":"gems","icon":"gem","name":"Gems","amount":0,"value":"","rate":"","perSecond":0,"full":false}],"loading":true},
  full: {"currencies":[{"id":"gold","icon":"coin","name":"Gold","amount":1000000,"value":"1M / 1M","rate":"","perSecond":0,"full":true},{"id":"mana","icon":"drop","name":"Mana","amount":250,"value":"250 / 250","rate":"+3.5/s","perSecond":3.5,"full":true}],"loading":false},
  draining: {"currencies":[{"id":"food","icon":"apple","name":"Food","amount":842,"value":"","rate":"","perSecond":-36,"full":false},{"id":"energy","icon":"bolt","name":"Energy","amount":12.5,"value":"","rate":"","perSecond":0.25,"full":false}],"loading":false},
  empty: {"currencies":[],"loading":false},
  overflow: {"currencies":[{"id":"gold","icon":"coin","name":"Gold Sovereigns of the Eastern Treasury","amount":1.234e+24,"value":"","rate":"","perSecond":5.6e+21,"full":false},{"id":"gems","icon":"gem","name":"Gems","amount":99999999,"value":"","rate":"+1.25 quadrillion per second","perSecond":0,"full":false},{"id":"wood","icon":"tree","name":"Wood","amount":5000,"value":"5,000 / 5,000","rate":"","perSecond":0,"full":true},{"id":"stone","icon":"shield","name":"Stone","amount":48210,"value":"","rate":"","perSecond":120,"full":false},{"id":"mana","icon":"drop","name":"Mana","amount":250,"value":"","rate":"","perSecond":3.5,"full":false},{"id":"souls","icon":"skull","name":"Souls","amount":7,"value":"","rate":"","perSecond":0,"full":false},{"id":"stars","icon":"star","name":"Stardust","amount":1500000000,"value":"","rate":"","perSecond":2200000,"full":false},{"id":"crowns","icon":"crown","name":"Crowns","amount":3,"value":"","rate":"","perSecond":0,"full":false}],"loading":false},
};
