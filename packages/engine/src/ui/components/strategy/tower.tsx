import type { Texture } from "@origingame/engine";
import { For, Icon, Image, Scroll, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./tower.en.ogt";

export interface StrategyTowerProps {
  /** The tower's name; empty takes the kit's (Tower). */
  name: string;
  /** An icon of the atlas for the tower, drawn when portrait is 0. */
  icon: string;
  /** A texture of the tower (og.load.texture); 0 draws icon. */
  portrait: Texture | 0;
  /** The tower's level as the game counts it; 0 hides it. */
  level: number;
  /** Its current stats, keyed rows of { id, label, value, next }: value as the game words it (42); next the value after the upgrade in focus (51), shown after an arrow, empty none. */
  stats?: ReadonlyArray<{ id: string; label: string; value: string; next: string }>;
  /** Upgrade paths, keyed rows of { id, name, icon, detail, tier, tiers, cost, state, reason }: tier of tiers bought; cost the next tier's price as the game words it (250 gold); state affordable, unaffordable, locked or maxed (data-state and a word; only affordable can be bought); reason why it is locked (Requires tier 3 on another path), empty none. */
  paths?: ReadonlyArray<{ id: string; name: string; icon: string; detail: string; tier: number; tiers: number; cost: string; state: string; reason: string }>;
  /** What selling returns, as the game words it (+180 gold), on the Sell button. */
  sellText: string;
  /** The game allows selling now; false disables Sell. */
  canSell?: boolean;
  /** Upgrade. */
  onUpgrade?: (e: UiEvent, id: string) => void;
  /** Sell. */
  onSell?: (e: UiEvent) => void;
}

/**
 * A tower's inspector: its picture, name and level, its current stats with
 * the value after the next upgrade where the game gives one, the upgrade
 * paths (tier of tiers, the next tier's cost, a state word: affordable,
 * too costly, locked with the reason, maxed) and Sell with what it returns.
 * Reports upgrade (the path's key; only an affordable path is enabled) and
 * sell. The game owns prices, refunds, stats and what is unlocked.
 */
export function StrategyTower(props: StrategyTowerProps) {
  return (
    <View class={`w-full max-h-full min-h-0 flex-col text-fg py-3 px-4 gap-y-4 bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none`} label={props.name !== "" ? props.name : t("strategy.tower.name")}>
      <View class="flex-row items-center gap-x-[0.714rem]">
        <View class="shrink flex-row items-center justify-center p-[0.143rem] size-[3.429rem] overflow-hidden bg-linear-to-b from-[#ffffff] to-[#38e8ff] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none">
          <Show when={props.portrait} fallback={
              <Icon class={`size-[70%] text-fg`} name={props.icon} />
          }>
            <Image class="size-full" src={props.portrait} />
          </Show>
        </View>
        <Text class={`flex-1 min-w-0 text-[1.429rem] leading-[1.15] font-bold text-fg whitespace-nowrap overflow-hidden text-shadow-xs text-shadow-bg/55`} role="heading" level={2}>
          {props.name !== "" ? props.name : t("strategy.tower.name")}
        </Text>
        <Show when={props.level > 0}>
          <Text class={`shrink whitespace-nowrap py-0 px-1 leading-[1.6] text-[0.857rem] font-bold tabular-nums text-gold bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`}>
            {t("strategy.tower.level", { level: props.level })}
          </Text>
        </Show>
      </View>
      <Show when={(props.stats ?? []).length}>
        <View class={`flex-col gap-y-1 p-2 bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38]`} role="list" label={t("strategy.tower.stats")}>
          <For each={props.stats ?? []} key={(s) => s.id}>
            {(s) => (
            <View class="flex-row items-baseline w-full" role="listitem" data-next={!!s().next}>
              <Text class={`flex-1 min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-[#ffffffc7] whitespace-nowrap overflow-hidden`}>
                {s().label}
              </Text>
              <Text class={`shrink text-[1.143rem] font-bold tabular-nums whitespace-nowrap text-fg`}>
                {s().value}
              </Text>
              <Show when={s().next}>
                <Text class={`shrink text-[1.143rem] font-bold tabular-nums whitespace-nowrap text-[#55e39a]`} label={t("strategy.tower.nextName", { next: s().next })}>
                  {t("strategy.tower.next", { next: s().next })}
                </Text>
              </Show>
            </View>
            )}
          </For>
        </View>
      </Show>
      <Show when={(props.paths ?? []).length} fallback={
          <View class={`p-3 text-center text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-[#ffffffc7]`} role="status">
            <Text>{t("strategy.tower.empty")}</Text>
          </View>
      }>
        <Scroll class="min-h-0 flex-1" role="list" label={t("strategy.tower.paths")}>
          <For each={props.paths ?? []} key={(p) => p.id}>
            {(p) => (
            <View class={`group/path flex-row items-center gap-x-[0.714rem] p-2 bg-linear-to-r from-[#070b14db] to-[#070b1459]`} role="listitem" label={p().name} data-affordable={p().state === "affordable"} data-locked={p().state === "locked"} data-maxed={p().state === "maxed"} data-unaffordable={p().state === "unaffordable"}>
              <Icon class={`shrink size-[2.286rem] text-fg group-data-affordable/path:text-[#38e8ff] group-data-unaffordable/path:text-[#ff4d5e] group-data-maxed/path:text-gold`} name={p().icon} />
              <View class="flex-1 min-w-0 overflow-hidden flex-col gap-y-0.5">
                <Text class={`min-w-0 text-[1.071rem] font-bold whitespace-nowrap overflow-hidden`}>
                  {p().name}
                </Text>
                <Show when={p().detail}>
                  <Text class={`text-[0.929rem] leading-[1.3] text-fg-muted whitespace-normal`}>
                    {p().detail}
                  </Text>
                </Show>
                <View class="flex-row items-center gap-x-1 w-full" role="meter" label={t("strategy.tower.tierName", { tier: p().tier, tiers: p().tiers })} valueMin={0} valueMax={p().tiers} valueNow={p().tier}>
                  <Text class={`shrink text-[0.857rem] font-bold tabular-nums whitespace-nowrap text-fg-muted`}>
                    {t("strategy.tower.tier", { tier: p().tier, tiers: p().tiers })}
                  </Text>
                  <View class="relative flex-1 min-w-0 h-1 overflow-hidden bg-[#04060cb8] shadow-[0_0_0_1px_#ffffff38,inset_0_1px_0_#ffffff24,0_2px_4px_#00000073]">
                    <View class={`absolute inset-y-0 left-0 bg-[#38e8ff] bg-linear-to-b from-[#ffffff8c] to-transparent to-42% group-data-maxed/path:from-[#ffffff80] group-data-maxed/path:to-gold`} style={{ width: `${(p().tiers > 0 ? p().tier / p().tiers : 0) * 100}%` }} />
                  </View>
                </View>
                <Show when={p().state === "locked" && p().reason}>
                  <Text class={`text-[0.929rem] leading-[1.3] font-semibold text-[#ff4d5e] whitespace-normal`}>
                    {p().reason}
                  </Text>
                </Show>
              </View>
              <View class={`shrink min-h-[3.143rem] min-w-[8.829rem] max-w-[40%] py-[0.143rem] px-3 bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none data-affordable:shadow-[inset_0_0_0_1px_#38e8ff73,inset_0_2px_0_#38e8ff] compact:col-start-2 compact:min-w-0 compact:max-w-none compact:py-1 compact:px-2`} role="button" label={t("strategy.tower.upgrade", { name: p().name })} description={p().state === "locked" ? p().reason : p().cost} disabled={p().state !== "affordable"} data-affordable={p().state === "affordable"} data-locked={p().state === "locked"} data-maxed={p().state === "maxed"} data-unaffordable={p().state === "unaffordable"} onPress={(e) => props.onUpgrade?.(e, p().id)}>
                <View class="flex-col items-center gap-y-0.5">
                  <Show when={p().state !== "affordable"}>
                    <Text class={`max-w-full whitespace-nowrap py-0 px-[0.143rem] leading-[1.6] text-[0.857rem] font-bold tabular-nums bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none text-[#ff4d5e] data-maxed:text-gold`} data-maxed={p().state === "maxed"}>
                      {p().state === "maxed" ? t("strategy.tower.maxed") : p().state === "locked" ? t("strategy.tower.locked") : t("strategy.tower.short")}
                    </Text>
                  </Show>
                  <Show when={p().state !== "maxed" && p().cost}>
                    <Text class={`max-w-full whitespace-nowrap text-[1.143rem] font-bold tabular-nums text-fg data-unaffordable:text-[#ff4d5e]`} data-unaffordable={p().state === "unaffordable"}>
                      {p().cost}
                    </Text>
                  </Show>
                </View>
              </View>
            </View>
            )}
          </For>
        </Scroll>
      </Show>
      <View class="flex-row justify-end compact:justify-stretch">
        <View class={`flex-row items-center gap-x-1 min-h-[3.143rem] py-[0.143rem] px-3 bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none text-[0.857rem] font-bold tracking-[0.14em] uppercase compact:w-full`} role="button" label={t("strategy.tower.sell")} disabled={!(props.canSell ?? false)} onPress={(e) => props.onSell?.(e)}>
          <Text>{t("strategy.tower.sell")}</Text>
          <Show when={props.sellText !== ""}>
            <Text class={`min-w-0 whitespace-nowrap overflow-hidden text-gold tracking-normal normal-case`}>
              {props.sellText}
            </Text>
          </Show>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<StrategyTowerProps>> = {
  default: {"name":"Arcane Spire","icon":"bolt","portrait":0,"level":3,"canSell":true,"sellText":"+180 gold","stats":[{"id":"dmg","label":"Damage","value":"42","next":"51"},{"id":"range","label":"Range","value":"6.5","next":""},{"id":"rate","label":"Attacks/s","value":"1.2","next":"1.4"}],"paths":[{"id":"focus","name":"Focusing Lens","icon":"gem","detail":"More damage to one target","tier":2,"tiers":4,"cost":"250 gold","state":"affordable","reason":""},{"id":"chain","name":"Chain Lightning","icon":"bolt","detail":"Arcs to nearby enemies","tier":1,"tiers":4,"cost":"420 gold","state":"unaffordable","reason":""},{"id":"frost","name":"Frost Ward","icon":"drop","detail":"Slows what it hits","tier":0,"tiers":4,"cost":"300 gold","state":"locked","reason":"Choose two paths at most"}]},
  maxed: {"name":"Cannon","icon":"bomb","portrait":0,"level":8,"canSell":true,"sellText":"+1,240 gold","stats":[{"id":"dmg","label":"Damage","value":"310","next":""},{"id":"splash","label":"Splash","value":"2.5","next":""}],"paths":[{"id":"big","name":"Big Bertha","icon":"bomb","detail":"","tier":4,"tiers":4,"cost":"","state":"maxed","reason":""},{"id":"cluster","name":"Cluster shot","icon":"fire","detail":"","tier":4,"tiers":4,"cost":"","state":"maxed","reason":""}]},
  disabled: {"name":"Watchtower","icon":"flag","portrait":0,"level":1,"canSell":false,"sellText":"+40 gold","stats":[{"id":"dmg","label":"Damage","value":"8","next":""}],"paths":[{"id":"a","name":"Longbows","icon":"bolt","detail":"","tier":0,"tiers":3,"cost":"90 gold","state":"unaffordable","reason":""},{"id":"b","name":"Hot oil","icon":"fire","detail":"","tier":0,"tiers":3,"cost":"120 gold","state":"locked","reason":"Build a Forge first"}]},
  empty: {"name":"Wall","icon":"shield","portrait":0,"level":0,"canSell":true,"sellText":"+5 gold","stats":[],"paths":[]},
  overflow: {"name":"Grand Celestial Observatory of the Seventh Astral Concord","icon":"star","portrait":0,"level":128,"canSell":true,"sellText":"+1,284,000,000 gold","stats":[{"id":"dmg","label":"Damage per celestial alignment","value":"1,240,500","next":"1,860,750"},{"id":"range","label":"Range","value":"14.0","next":""},{"id":"rate","label":"Attacks/s","value":"0.25","next":"0.3"},{"id":"crit","label":"Critical chance","value":"35%","next":"40%"},{"id":"pierce","label":"Pierce","value":"12","next":""}],"paths":[{"id":"p1","name":"Starfall Convergence of the Eternal Night","icon":"star","detail":"Every fifth shot calls down a meteor that splits into lesser stars","tier":7,"tiers":10,"cost":"12,800,000 gold","state":"affordable","reason":""},{"id":"p2","name":"Nebula","icon":"drop","detail":"Clouds that slow","tier":3,"tiers":10,"cost":"900,000,000 gold","state":"unaffordable","reason":""},{"id":"p3","name":"Gravity Well","icon":"gem","detail":"","tier":0,"tiers":10,"cost":"5,000,000 gold","state":"locked","reason":"Requires Starfall tier 8 and the Astral Library on the eastern isle"},{"id":"p4","name":"Lens","icon":"gem","detail":"","tier":10,"tiers":10,"cost":"","state":"maxed","reason":""}]},
};
