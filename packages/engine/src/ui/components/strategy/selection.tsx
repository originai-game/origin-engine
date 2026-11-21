import type { Texture } from "@origingame/engine";
import { For, Icon, Image, Scroll, Show, Text, View, formatNumber, t, type UiEvent } from "@origingame/engine/ui";
import "./selection.en.ogt";

export interface StrategySelectionProps {
  /** The selected unit's or building's name, or the focused subgroup's (Men-at-Arms); empty shows Nothing selected. */
  name: string;
  /** How many of it are selected (×12, shown above 1). */
  count: number;
  /** unit or building (data-kind on the frame). */
  kind: string;
  /** Its class line as the game words it (Heavy infantry · Veteran II), under the name; empty none. */
  line: string;
  /** What it is doing as the game words it (Holding, Under construction), on a tag before the stats; empty none. */
  status: string;
  /** Its portrait (og.load.texture) in a tile with a lit edge; 0 draws icon instead. */
  portrait: Texture | 0;
  /** An icon of the atlas drawn when portrait is 0. */
  icon: string;
  /** Its health now (the subgroup's together), under the portrait as a bar and in figures. */
  health: number;
  /** Its whole health; 0 hides the bar and the figures. Default 1. */
  healthMax?: number;
  /** Its stats on tags, keyed rows of { id, label, value }: label short as the game words it (ATK), value its figure (12). */
  stats?: ReadonlyArray<{ id: string; label: string; value: string }>;
  /** Each unit or subgroup selected, as cards in the game's order (shown when there are two or more; two rows, then they scroll), keyed rows of { id, name, kind, portrait, icon, count, health, healthMax }: portrait a texture, 0 draws icon from the atlas; count shown above 1 on a tag; health over healthMax a bar on the card, green, gold when hurt and red when low (data-tone). */
  units?: ReadonlyArray<{ id: string; name: string; kind: string; portrait: Texture | 0; icon: string; count: number; health: number; healthMax: number }>;
  /** The card whose unit or subgroup the panel shows (its edge lit, aria-selected); empty none. */
  focused: string;
  /** Choose a card (its key: show that unit or subgroup). */
  onChoose?: (e: UiEvent, id: string) => void;
}

/**
 * The selection panel at the foot: the portrait in a tile with a lit edge,
 * its health as a bar and in figures under it; the name large with how
 * many are selected, its class line in small capitals, what it is doing
 * and its stats on tags; and every unit selected as a card with its health,
 * the focused one lit. Reports choose (a card's key: show that unit or
 * subgroup). The game owns every figure and what the panel shows.
 */
export function StrategySelection(props: StrategySelectionProps) {
  return (
    <View class={`flex-row items-start gap-x-[1rem] w-full min-w-0 text-fg compact:gap-x-[0.571rem]`} label={props.name !== "" ? props.name : t("strategy.selection.title")} data-kind={props.kind}>
      <Show when={props.name !== ""} fallback={
          <Text class={`flex-1 py-[1.143rem] text-center text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-[#ffffffc7]`} role="status">
            {t("strategy.selection.empty")}
          </Text>
      }>
        <View class={`flex-col gap-y-[0.357rem] shrink min-w-[6.171rem] compact:gap-y-[0.214rem] compact:min-w-[3.429rem]`}>
          <View class={`self-center size-[6.171rem] compact:size-[3.429rem] p-[0.143rem] bg-linear-to-b from-[#ffffff] to-[#38e8ff] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none`}>
            <View class={`size-full flex-row items-center justify-center overflow-hidden bg-linear-to-b from-[#1e2637e5] to-plate-lo rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none`}>
              <Show when={props.portrait} fallback={
                  <Icon class={`size-[55%] text-fg`} name={props.icon} />
              }>
                <Image class="size-full" src={props.portrait} />
              </Show>
            </View>
          </View>
          <Show when={(props.healthMax ?? 1) > 0}>
            <View
              class={`group/health relative h-[0.5rem] compact:h-[0.357rem] overflow-hidden bg-[#04060cb8] shadow-[0_0_0_1px_#ffffff38,inset_0_1px_0_#ffffff24,0_2px_4px_#00000073]`}
              role="meter"
              label={t("strategy.selection.health")}
              valueMin={0}
              valueMax={props.healthMax ?? 1}
              valueNow={props.health <= 0 ? 0 : props.health}
              data-hurt={props.health / (props.healthMax ?? 1) <= 0.6}
              data-low={props.health / (props.healthMax ?? 1) <= 0.3}
            >
              <View class={`absolute inset-y-0 left-0 bg-[#55e39a] bg-linear-to-b from-[#ffffff73] to-transparent to-50% group-data-hurt/health:bg-gold group-data-low/health:bg-[#ff4d5e]`} style={{ width: `${(props.health <= 0 ? 0 : props.health >= (props.healthMax ?? 1) ? 1 : props.health / (props.healthMax ?? 1)) * 100}%` }} />
            </View>
            <View class={`flex-row justify-center items-baseline gap-x-[0.214rem]`}>
              <Text class={`shrink text-[0.929rem] font-bold tabular-nums whitespace-nowrap`}>
                {formatNumber(props.health)}
              </Text>
              <Text class={`shrink text-[0.929rem] font-bold tabular-nums whitespace-nowrap text-fg-muted compact:hidden`}>
                {t("strategy.selection.healthMax", { max: props.healthMax ?? 1 })}
              </Text>
            </View>
          </Show>
        </View>
        <View class={`flex-col gap-y-[0.571rem] flex-1 min-w-0 compact:gap-y-[0.357rem]`}>
          <View class={`flex-row flex-wrap items-start justify-between gap-x-[0.857rem] gap-y-[0.286rem] min-w-0`}>
            <View class={`flex-col gap-y-[0.071rem] flex-1 min-w-0`}>
              <View class={`flex-row items-baseline gap-x-[0.429rem] min-w-0`}>
                <Text class={`shrink min-w-0 text-[1.429rem] leading-[1.15] font-bold whitespace-nowrap overflow-hidden text-shadow-xs text-shadow-bg/55 compact:text-[1.143rem]`}>
                  {props.name}
                </Text>
                <Show when={props.count > 1}>
                  <Text class={`shrink text-[1.143rem] font-bold tabular-nums whitespace-nowrap text-fg-muted compact:text-[0.929rem]`}>
                    {t("strategy.selection.count", { count: props.count })}
                  </Text>
                </Show>
              </View>
              <Show when={props.line !== ""}>
                <Text class={`min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-[#ffffffc7] compact:text-[0.857rem] compact:tracking-normal`}>
                  {props.line}
                </Text>
              </Show>
            </View>
            <Show when={(props.stats ?? []).length || props.status !== ""}>
              <View class={`flex-row flex-wrap items-center justify-end gap-x-[0.357rem] gap-y-[0.286rem] min-w-0`} role="list" label={t("strategy.selection.stats")}>
                <Show when={props.status !== ""}>
                  <Text class={`shrink min-w-0 py-0 px-[0.429rem] leading-[1.6] text-[0.857rem] font-bold whitespace-nowrap overflow-hidden text-gold bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} role="status">
                    {props.status}
                  </Text>
                </Show>
                <For each={props.stats ?? []} key={(s) => s.id}>
                  {(s) => (
                  <Text class={`shrink min-w-0 py-0 px-[0.429rem] leading-[1.6] text-[0.857rem] font-bold whitespace-nowrap overflow-hidden bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} role="listitem" label={t("strategy.selection.stat", { label: s().label, value: s().value })}>
                    {t("strategy.selection.stat", { label: s().label, value: s().value })}
                  </Text>
                  )}
                </For>
              </View>
            </Show>
          </View>
          <Show when={(props.units ?? []).length > 1}>
            <Scroll class={`max-h-[4.571rem] min-h-0 compact:max-h-none`} role="listbox" label={t("strategy.selection.units")}>
              <View class={`flex flex-wrap content-start gap-1 min-w-0 compact:flex-nowrap`}>
              <For each={props.units ?? []} key={(u) => u.id}>
                {(u, i) => (
                <View
                  class={`shrink p-[0.143rem] size-[2.143rem] compact:size-[2.286rem] bg-linear-to-b from-[#ffffffbf] via-[#ffffff38] via-45% to-[#ffffff24] data-selected:bg-linear-to-b data-selected:from-[#ffffff] data-selected:to-[#38e8ff] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none compact:data-past:hidden`}
                  role="option"
                  label={u().name}
                  description={t("strategy.selection.unitValue", { count: u().count, now: u().health, max: u().healthMax })}
                  selected={u().id === props.focused}
                  data-kind={u().kind}
                  data-past={i() >= 4}
                  onPress={(e) => props.onChoose?.(e, u().id)}
                >
                  <View class="relative size-full overflow-hidden rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none">
                    <Show when={u().portrait} fallback={
                        <View class="absolute inset-0 flex-row items-center justify-center bg-linear-to-b from-[#1e2637e5] to-plate-lo">
                          <Icon class={`size-[60%] text-fg`} name={u().icon} />
                        </View>
                    }>
                      <Image class="absolute inset-0 size-full" src={u().portrait} />
                    </Show>
                    <Show when={u().count > 1}>
                      <Text class={`absolute right-0 top-0 py-0 px-[0.143rem] leading-[1.2] text-[0.857rem] font-bold whitespace-nowrap bg-[#060910e0]`}>
                        {t("strategy.selection.count", { count: u().count })}
                      </Text>
                    </Show>
                    <View
                      class={`group/meter absolute left-[0.214rem] right-[0.214rem] bottom-[0.214rem] h-[0.214rem] overflow-hidden bg-[#04060cb2]`}
                      data-hurt={u().healthMax > 0 && u().health / u().healthMax <= 0.6 && u().health / u().healthMax > 0.3}
                      data-low={u().healthMax > 0 && u().health / u().healthMax <= 0.3}
                    >
                      <View class={`absolute inset-y-0 left-0 bg-[#55e39a] group-data-hurt/meter:bg-gold group-data-low/meter:bg-[#ff4d5e]`} style={{ width: `${(u().healthMax <= 0 || u().health <= 0 ? 0 : u().health >= u().healthMax ? 1 : u().health / u().healthMax) * 100}%` }} />
                    </View>
                  </View>
                </View>
                )}
              </For>
              <Show when={(props.units ?? []).length > 4}>
                <View class={`hidden compact:flex-row compact:items-center compact:justify-center shrink compact:size-[2.286rem] text-[0.929rem] font-bold tabular-nums whitespace-nowrap text-fg bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} role="status" label={t("strategy.selection.more", { n: (props.units ?? []).length - 4 })}>
                  <Text>{t("strategy.selection.moreCount", { n: (props.units ?? []).length - 4 })}</Text>
                </View>
              </Show>
              </View>
            </Scroll>
          </Show>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<StrategySelectionProps>> = {
  default: {"name":"Men-at-Arms","count":12,"kind":"unit","line":"Heavy infantry · Veteran II","status":"","portrait":0,"icon":"sword","health":1476,"healthMax":1800,"stats":[{"id":"atk","label":"ATK","value":"12"},{"id":"arm","label":"ARM","value":"4"},{"id":"spd","label":"SPD","value":"3.2"}],"focused":"m1","units":[{"id":"m1","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":90,"healthMax":100},{"id":"m2","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":55,"healthMax":100},{"id":"m3","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"m4","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":25,"healthMax":100},{"id":"m5","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":80,"healthMax":100},{"id":"m6","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"m7","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"a1","name":"Archer","kind":"unit","portrait":0,"icon":"bolt","count":1,"health":70,"healthMax":100},{"id":"a2","name":"Archer","kind":"unit","portrait":0,"icon":"bolt","count":1,"health":100,"healthMax":100},{"id":"a3","name":"Archer","kind":"unit","portrait":0,"icon":"bolt","count":1,"health":45,"healthMax":100},{"id":"a4","name":"Archer","kind":"unit","portrait":0,"icon":"bolt","count":1,"health":100,"healthMax":100},{"id":"a5","name":"Archer","kind":"unit","portrait":0,"icon":"bolt","count":1,"health":95,"healthMax":100}]},
  single: {"name":"Stone Keep","count":1,"kind":"building","line":"Town centre","status":"Training","portrait":0,"icon":"home","health":1850,"healthMax":2400,"stats":[{"id":"arm","label":"ARM","value":"18"},{"id":"gar","label":"GARRISON","value":"10"}],"focused":"","units":[]},
  empty: {"name":"","count":0,"kind":"unit","line":"","status":"","portrait":0,"icon":"","health":0,"healthMax":0,"stats":[],"focused":"","units":[]},
  groups: {"name":"Spearmen","count":6,"kind":"unit","line":"Light infantry","status":"Holding","portrait":0,"icon":"sword","health":340,"healthMax":420,"stats":[{"id":"atk","label":"ATK","value":"6"},{"id":"arm","label":"ARM","value":"1"}],"focused":"spear","units":[{"id":"spear","name":"Spearmen","kind":"unit","portrait":0,"icon":"sword","count":6,"health":340,"healthMax":420},{"id":"archer","name":"Archers","kind":"unit","portrait":0,"icon":"bolt","count":2,"health":150,"healthMax":160},{"id":"captain","name":"Captain Ilse","kind":"unit","portrait":0,"icon":"crown","count":1,"health":96,"healthMax":300}]},
  dense: {"name":"Men-at-Arms","count":40,"kind":"unit","line":"Heavy infantry","status":"","portrait":0,"icon":"sword","health":3610,"healthMax":4000,"stats":[{"id":"atk","label":"ATK","value":"12"},{"id":"arm","label":"ARM","value":"4"}],"focused":"u1","units":[{"id":"u1","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u2","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":90,"healthMax":100},{"id":"u3","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":80,"healthMax":100},{"id":"u4","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":70,"healthMax":100},{"id":"u5","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":60,"healthMax":100},{"id":"u6","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":50,"healthMax":100},{"id":"u7","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":40,"healthMax":100},{"id":"u8","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":30,"healthMax":100},{"id":"u9","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":20,"healthMax":100},{"id":"u10","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":10,"healthMax":100},{"id":"u11","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u12","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u13","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u14","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u15","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u16","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u17","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u18","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u19","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u20","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u21","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u22","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u23","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u24","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u25","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u26","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u27","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u28","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u29","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u30","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u31","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u32","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u33","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u34","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u35","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u36","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u37","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u38","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u39","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100},{"id":"u40","name":"Man-at-Arms","kind":"unit","portrait":0,"icon":"sword","count":1,"health":100,"healthMax":100}]},
  overflow: {"name":"Grand Marshal Aurelia von Hohenstaufen-Wittelsbach the Unyielding","count":1284,"kind":"unit","line":"Hero · Commander of the Northern Expeditionary Host","status":"Channelling the Banner of the Old Kingdom","portrait":0,"icon":"crown","health":2,"healthMax":1800000,"stats":[{"id":"atk","label":"ATTACK","value":"1,240,500"},{"id":"arm","label":"ARMOUR","value":"35"},{"id":"spd","label":"SPEED","value":"3.2"},{"id":"rng","label":"RANGE","value":"14"}],"focused":"u7","units":[{"id":"u1","name":"Spearmen","kind":"unit","portrait":0,"icon":"sword","count":12,"health":900,"healthMax":960},{"id":"u2","name":"Archers","kind":"unit","portrait":0,"icon":"bolt","count":8,"health":500,"healthMax":640},{"id":"u3","name":"Knights","kind":"unit","portrait":0,"icon":"shield","count":6,"health":1100,"healthMax":1500},{"id":"u4","name":"Healers","kind":"unit","portrait":0,"icon":"potion","count":4,"health":220,"healthMax":320},{"id":"u5","name":"Scouts","kind":"unit","portrait":0,"icon":"map","count":3,"health":40,"healthMax":300},{"id":"u6","name":"Rams","kind":"unit","portrait":0,"icon":"bomb","count":2,"health":600,"healthMax":1400},{"id":"u7","name":"Grand Marshal Aurelia","kind":"unit","portrait":0,"icon":"crown","count":1,"health":2,"healthMax":1800},{"id":"u8","name":"Crossbowmen","kind":"unit","portrait":0,"icon":"bolt","count":1284,"health":480,"healthMax":480},{"id":"b1","name":"Watchtower","kind":"building","portrait":0,"icon":"flag","count":2,"health":700,"healthMax":900},{"id":"b2","name":"Barracks","kind":"building","portrait":0,"icon":"home","count":1,"health":1500,"healthMax":1500}]},
};
