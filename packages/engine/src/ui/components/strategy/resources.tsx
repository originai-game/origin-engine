import type { Texture } from "@origingame/engine";
import { For, Icon, Image, Show, Text, View, t } from "@origingame/engine/ui";
import "./strategy.en.ogt";

export interface StrategyResourcesProps {
  /** The stockpiles in the game's order, keyed rows of { id, name, icon, glyph, amount, cap, rate, trend, note, warn }: name for a reader (Gold); icon its picture (og.load.texture), 0 draws glyph (an icon of the atlas) instead; amount the store as the game words it (1,250); cap a ceiling shown after it (/ 50, the population's), empty none; rate the income as the game words it (+12/s), under the amount, inked by trend (gain, loss or none: data-trend); note a word in place of the rate (Near cap), empty none; warn marks a store at its limit in gold (data-warn). */
  resources?: ReadonlyArray<{ id: string; name: string; icon: Texture | 0; glyph: string; amount: string; cap: string; rate: string; trend: string; note: string; warn: boolean }>;
  /** The age or tier as the game words it (Castle Age), on a lit tag after the stockpiles; empty hides it. */
  tier: string;
}

/**
 * The resource bar at the top left: each stockpile's picture in a cut tile,
 * its amount large (a ceiling after it, dim, as the population's), and its
 * income under it, green while it grows and red while it drains, or the
 * game's word (Near cap) in gold at a limit; the age or tier on a lit tag.
 * The game owns every figure, rate and limit; read only.
 */
export function StrategyResources(props: StrategyResourcesProps) {
  return (
    <View class={`flex-row flex-wrap items-center w-fit max-w-full min-w-0 gap-x-[1.571rem] gap-y-[0.429rem] py-[0.429rem] px-[1.143rem] text-fg border-rim bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none border compact:w-full compact:justify-between compact:gap-x-2 compact:gap-y-1 compact:py-1 compact:px-[0.714rem]`} role="group" label={t("strategy.resources.name")}>
      <For each={props.resources ?? []} key={(r) => r.id}>
        {(r) => (
        <View class={`group/res flex-row items-center gap-x-[0.5rem] min-w-0 compact:gap-x-1`} role="group" label={t("strategy.resources.value", { name: r().name, amount: r().amount })} data-warn={r().warn}>
          <Show when={r().icon} fallback={
              <Icon class={`shrink size-[1.714rem] text-fg compact:size-[1.143rem]`} name={r().glyph} />
          }>
            <Image class={`shrink size-[1.714rem] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none compact:size-[1.143rem]`} src={r().icon} />
          </Show>
          <View class="flex-col min-w-0">
            <View class="flex-row items-baseline gap-x-[0.214rem] min-w-0">
              <Text class={`min-w-0 text-[1.286rem] leading-[1.1] font-bold tabular-nums whitespace-nowrap overflow-hidden text-stroke-3 text-stroke-bg group-data-warn/res:text-gold compact:text-[1.071rem]`}>
                {r().amount}
              </Text>
              <Show when={r().cap}>
                <Text class={`shrink text-[0.929rem] leading-[1.1] font-bold tabular-nums whitespace-nowrap text-fg-muted compact:text-[0.857rem]`}>
                  {t("strategy.resources.cap", { cap: r().cap })}
                </Text>
              </Show>
            </View>
            <Show when={r().note} fallback={
                <Show when={r().rate}>
                  <Text class={`min-w-0 leading-[1.2] text-[0.857rem] font-semibold tabular-nums whitespace-nowrap overflow-hidden text-fg-muted data-gain:text-[#55e39a] data-loss:text-[#ff4d5e] data-note:text-gold compact:text-[0.857rem]`} data-gain={r().trend === "gain"} data-loss={r().trend === "loss"} data-note={r().trend === "note"}>
                    {r().rate}
                  </Text>
                </Show>
            }>
              <Text class={`min-w-0 leading-[1.2] text-[0.857rem] font-semibold tabular-nums whitespace-nowrap overflow-hidden text-gold compact:text-[0.857rem]`}>
                {r().note}
              </Text>
            </Show>
          </View>
        </View>
        )}
      </For>
      <Show when={props.tier !== ""}>
        <Text class={`shrink py-[0.143rem] px-[0.571rem] text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-[#141a02] bg-[#38e8ff] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`}>
          {props.tier}
        </Text>
      </Show>
      <Show when={!(props.resources ?? []).length && props.tier === ""}>
        <View class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ffffffc7]`} role="status">
          <Text>{t("strategy.resources.empty")}</Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<StrategyResourcesProps>> = {
  default: {"tier":"","resources":[{"id":"gold","name":"Gold","icon":0,"glyph":"coin","amount":"1,250","cap":"","rate":"+12/s","trend":"gain","note":"","warn":false},{"id":"wood","name":"Wood","icon":0,"glyph":"leaf","amount":"840","cap":"","rate":"+6/s","trend":"gain","note":"","warn":false},{"id":"food","name":"Food","icon":0,"glyph":"apple","amount":"615","cap":"","rate":"−2/s","trend":"loss","note":"","warn":false},{"id":"crystal","name":"Crystal","icon":0,"glyph":"gem","amount":"90","cap":"","rate":"+0/s","trend":"none","note":"","warn":false},{"id":"pop","name":"Population","icon":0,"glyph":"person","amount":"47","cap":"50","rate":"","trend":"none","note":"Near cap","warn":true}]},
  tier: {"tier":"Castle Age","resources":[{"id":"food","name":"Food","icon":0,"glyph":"apple","amount":"2,410","cap":"","rate":"+31/s","trend":"gain","note":"","warn":false},{"id":"wood","name":"Wood","icon":0,"glyph":"leaf","amount":"1,180","cap":"","rate":"+18/s","trend":"gain","note":"","warn":false},{"id":"gold","name":"Gold","icon":0,"glyph":"coin","amount":"960","cap":"","rate":"+9/s","trend":"gain","note":"","warn":false},{"id":"stone","name":"Stone","icon":0,"glyph":"shield","amount":"350","cap":"","rate":"+2/s","trend":"gain","note":"","warn":false},{"id":"pop","name":"Population","icon":0,"glyph":"person","amount":"112","cap":"200","rate":"","trend":"none","note":"","warn":false}]},
  capped: {"tier":"","resources":[{"id":"gold","name":"Gold","icon":0,"glyph":"coin","amount":"0","cap":"","rate":"−8/s","trend":"loss","note":"","warn":true},{"id":"pop","name":"Population","icon":0,"glyph":"person","amount":"50","cap":"50","rate":"","trend":"none","note":"Housed out","warn":true}]},
  empty: {"tier":"","resources":[]},
  overflow: {"tier":"Imperial Age of the Unconquered Sun","resources":[{"id":"gold","name":"Gold","icon":0,"glyph":"coin","amount":"1,284,000,000","cap":"","rate":"+125,000/s","trend":"gain","note":"","warn":false},{"id":"wood","name":"Wood","icon":0,"glyph":"leaf","amount":"98,765,432","cap":"","rate":"−4,500/s","trend":"loss","note":"","warn":false},{"id":"food","name":"Food","icon":0,"glyph":"apple","amount":"12,000,000","cap":"","rate":"+0/s","trend":"none","note":"","warn":false},{"id":"crystal","name":"Crystal","icon":0,"glyph":"gem","amount":"4,096","cap":"","rate":"+64/s","trend":"gain","note":"","warn":false},{"id":"stone","name":"Stone","icon":0,"glyph":"shield","amount":"55,555","cap":"","rate":"+1/s","trend":"gain","note":"","warn":false},{"id":"favour","name":"Favour","icon":0,"glyph":"star","amount":"999","cap":"","rate":"","trend":"none","note":"Storage full","warn":true},{"id":"pop","name":"Population","icon":0,"glyph":"person","amount":"1,000","cap":"1,000","rate":"","trend":"none","note":"Housed out","warn":true}]},
};
