import type { Texture } from "@origingame/engine";
import { For, formatDuration, Icon, Image, packClass, Show, t, Text, View, type UiEvent } from "@origingame/engine/ui";
import "./strategy.en.ogt";

export interface StrategyWaveProps {
  /** The wave on now or coming, from 1. */
  wave: number;
  /** Waves in the level; 0 is an endless run (no total shown). */
  waves: number;
  /** The wave's name as the game words it (Siege), after its number; empty none. */
  name: string;
  /** The wave's picture (og.load.texture) in a cut tile before it; 0 hides it. */
  icon: Texture | 0;
  /** countdown (the wave is coming), active (a wave is on) or done (every wave cleared), as the game says: data-phase; active and done stand as words in place of the countdown. */
  phase: string;
  /** Seconds to the wave while it is coming, shown as a countdown (0:42); negative hides it. Default -1. */
  next?: number;
  /** The whole wait: the bar beside the countdown shows the share still to run; 0 draws no bar. */
  nextTotal: number;
  /** Seconds left when the countdown is near (data-near on the frame); 5 when left out. Default 5. */
  warnAt?: number;
  /** The coming or current wave is the last: the Final wave badge (data-final). */
  finalWave: boolean;
  /** The wave's enemies by kind, under the plate, keyed rows of { id, name, icon, count, boss, scouted }: icon an icon of the atlas; count shown as ×n; boss marks it with a Boss badge (data-boss); scouted false hides name and count behind Not yet scouted. Empty shows none. */
  enemies?: ReadonlyArray<{ id: string; name: string; icon: string; count: number; boss: boolean; scouted: boolean }>;
  /** The game allows calling the wave now; false hides Call early. */
  canCallEarly: boolean;
  /** What calling early pays, as the game words it (+40 gold), on the button in gold; empty none. */
  reward: string;
  /** Call the wave early. */
  onCallEarly?: (e: UiEvent) => void;
}

/**
 * The wave readout at the top right: a plate whose edge is lit in danger
 * while a wave comes or fights, its picture, Wave n (of N) and its name in
 * red capitals, the countdown with a bar of the wait still to run, or
 * Under attack or All waves cleared; the wave's enemies by kind under it,
 * and Call early with what it pays. Reports callEarly; the game owns the
 * timing, the wave's makeup and the reward.
 */
export function StrategyWave(props: StrategyWaveProps) {
  const next = () => props.next ?? -1;
  return (
    <View
      class={`group/wave flex-col gap-y-[0.357rem] w-fit max-w-full min-w-0 text-fg compact:w-full`}
      label={t("strategy.wave.name")}
      data-final={props.finalWave}
      data-done={props.phase === "done"}
      data-near={props.phase === "countdown" && next() >= 0 && next() <= (props.warnAt ?? 5)}
    >
      <View class={`flex-row items-center gap-x-[0.714rem] min-w-0 py-[0.429rem] px-[1rem] nine-plate-cut bg-[#1e2637e5] compact:gap-x-[0.571rem] compact:py-[0.286rem] compact:px-[0.714rem] ${packClass("Plate", "frame")}`}>
        <Show when={props.icon}>
          <Image class={`shrink size-[1.714rem] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} src={props.icon} />
        </Show>
        <View class={`flex-col gap-y-[0.071rem] flex-1 min-w-0`}>
          <View class={`flex-row items-center gap-x-[0.429rem] min-w-0`}>
            <Text class={`shrink min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ff4d5e] group-data-done/wave:text-[#ffffffc7] compact:text-[0.857rem] compact:tracking-normal`}>
              {props.waves > 0
                ? (props.name !== "" ? t("strategy.wave.titleOfNamed", { wave: props.wave, waves: props.waves, name: props.name }) : t("strategy.wave.titleOf", { wave: props.wave, waves: props.waves }))
                : (props.name !== "" ? t("strategy.wave.titleNamed", { wave: props.wave, name: props.name }) : t("strategy.wave.title", { wave: props.wave }))}
            </Text>
            <Show when={props.finalWave}>
              <Text class={`shrink py-0 px-[0.357rem] leading-[1.4] text-[0.857rem] font-bold whitespace-nowrap text-gold nine-chip-cut bg-[#060910e0] ${packClass("Chip", "frame")}`}>
                {t("strategy.wave.final")}
              </Text>
            </Show>
          </View>
          <Show when={props.phase === "countdown" && next() >= 0} fallback={
              <Text class={`min-w-0 text-[1.143rem] leading-[1.2] font-bold whitespace-nowrap overflow-hidden text-stroke-3 text-stroke-bg group-data-done/wave:text-[#55e39a]`} role="status">
                {props.phase === "done" ? t("strategy.wave.done") : props.phase === "active" ? t("strategy.wave.active") : t("strategy.wave.countdown")}
              </Text>
          }>
            <View class={`flex-row items-center gap-x-[0.571rem] min-w-0`} role="timer" label={t("strategy.wave.next")} valueText={formatDuration(next())}>
              <Text class={`shrink text-[1.286rem] leading-[1.1] font-bold tabular-nums whitespace-nowrap text-stroke-3 text-stroke-bg group-data-near/wave:text-[#ff4d5e] compact:text-[1.071rem]`}>
                {formatDuration(next())}
              </Text>
              <Show when={props.nextTotal > 0}>
                <View class={`relative flex-1 min-w-[5.486rem] compact:min-w-[3.429rem] h-[0.429rem] overflow-hidden bg-[#04060cb8] shadow-[0_0_0_1px_#ffffff38,inset_0_1px_0_#ffffff24,0_2px_4px_#00000073]`}>
                  <View class={`absolute inset-y-0 left-0 bg-[#ff4d5e] bg-linear-to-b from-[#ffffff66] to-transparent to-42%`} style={{ width: `${(next() >= props.nextTotal ? 1 : next() / props.nextTotal) * 100}%` }} />
                </View>
              </Show>
            </View>
          </Show>
        </View>
      </View>
      <Show when={(props.enemies ?? []).length && props.phase !== "done"}>
        <View class={`flex-col gap-y-[0.214rem] min-w-0`} role="list" label={t("strategy.wave.incoming")}>
          <For each={props.enemies ?? []} key={(e) => e.id}>
            {(e) => (
            <View class={`group/enemy flex-row items-center gap-x-[0.429rem] min-w-0 pt-[0.143rem] pb-[0.143rem] pr-[0.714rem] bg-linear-to-r from-[#070b14db] to-[#070b1459]`} role="listitem" label={e().scouted ? t("strategy.wave.enemyName", { count: e().count, name: e().name }) : t("strategy.wave.unknown")} data-boss={e().boss} data-unscouted={!e().scouted}>
              <View class={`shrink self-stretch w-[0.214rem] mr-[0.286rem] bg-[#ff4d5e] group-data-unscouted/enemy:bg-[#ffffff99]`} />
              <Icon class={`shrink size-[1.143rem] text-fg`} name={e().icon} />
              <Text class={`flex-1 min-w-0 text-[1rem] font-semibold whitespace-nowrap overflow-hidden group-data-unscouted/enemy:text-fg-muted`}>
                {e().scouted ? e().name : t("strategy.wave.unknown")}
              </Text>
              <Show when={e().boss}>
                <Text class={`shrink py-0 px-[0.357rem] leading-[1.4] text-[0.857rem] font-bold whitespace-nowrap text-[#ff4d5e] nine-chip-cut bg-[#060910e0] ${packClass("Chip", "frame")}`}>
                  {t("strategy.wave.boss")}
                </Text>
              </Show>
              <Show when={e().scouted}>
                <Text class={`shrink text-[1rem] font-bold tabular-nums whitespace-nowrap`}>
                  {t("strategy.wave.enemyCount", { count: e().count })}
                </Text>
              </Show>
            </View>
            )}
          </For>
        </View>
      </Show>
      <Show when={props.canCallEarly && props.phase !== "done"}>
        <View
          class={`min-h-[3.143rem] min-w-0 max-w-full py-[0.286rem] px-[0.857rem] nine-plate-cut bg-[#1e2637e5] ${packClass("Button", "plate", "primary")}`}
          role="button"
          label={t("strategy.wave.callEarly")}
          onPress={(e) => props.onCallEarly?.(e)}
        >
          <View class={`flex-row flex-wrap items-baseline justify-center gap-x-[0.571rem] max-w-full min-w-0`}>
            <Text class={`shrink text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-fg`}>
              {t("strategy.wave.callEarly")}
            </Text>
            <Show when={props.reward !== ""}>
              <Text class={`shrink min-w-0 max-w-full text-[0.929rem] font-bold text-gold whitespace-nowrap overflow-hidden`}>
                {props.reward}
              </Text>
            </Show>
          </View>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<StrategyWaveProps>> = {
  default: {"wave":7,"waves":0,"name":"Siege","icon":0,"phase":"countdown","next":42,"nextTotal":140,"warnAt":5,"finalWave":false,"canCallEarly":false,"reward":"","enemies":[]},
  incoming: {"wave":6,"waves":20,"name":"","icon":0,"phase":"countdown","next":14.2,"nextTotal":30,"warnAt":5,"finalWave":false,"canCallEarly":true,"reward":"+35 gold","enemies":[{"id":"goblin","name":"Goblin","icon":"skull","count":18,"boss":false,"scouted":true},{"id":"wolf","name":"Warg rider","icon":"bolt","count":6,"boss":false,"scouted":true},{"id":"shaman","name":"Shaman","icon":"fire","count":2,"boss":false,"scouted":true}]},
  active: {"wave":7,"waves":20,"name":"","icon":0,"phase":"active","next":-1,"nextTotal":0,"warnAt":5,"finalWave":false,"canCallEarly":false,"reward":"","enemies":[{"id":"goblin","name":"Goblin","icon":"skull","count":24,"boss":false,"scouted":true},{"id":"ogre","name":"Ogre","icon":"shield","count":3,"boss":false,"scouted":true}]},
  final: {"wave":20,"waves":20,"name":"Warlord","icon":0,"phase":"countdown","next":4.5,"nextTotal":45,"warnAt":5,"finalWave":true,"canCallEarly":true,"reward":"+120 gold","enemies":[{"id":"warlord","name":"Iron Warlord","icon":"crown","count":1,"boss":true,"scouted":true},{"id":"guard","name":"Honour guard","icon":"shield","count":8,"boss":false,"scouted":true},{"id":"x","name":"","icon":"skull","count":0,"boss":false,"scouted":false}]},
  done: {"wave":20,"waves":20,"name":"","icon":0,"phase":"done","next":-1,"nextTotal":0,"warnAt":5,"finalWave":false,"canCallEarly":false,"reward":"","enemies":[]},
  empty: {"wave":1,"waves":0,"name":"","icon":0,"phase":"countdown","next":30,"nextTotal":30,"warnAt":5,"finalWave":false,"canCallEarly":true,"reward":"","enemies":[]},
  overflow: {"wave":1284,"waves":0,"name":"The Abyssal Tide of the Thousand-Year Eclipse","icon":0,"phase":"countdown","next":3725,"nextTotal":3725,"warnAt":5,"finalWave":false,"canCallEarly":true,"reward":"+12,450,000 gold and a Legendary Chest","enemies":[{"id":"e1","name":"Abyssal Behemoth of the Thousand-Year Eclipse","icon":"skull","count":1,"boss":true,"scouted":true},{"id":"e2","name":"Goblin","icon":"skull","count":12480,"boss":false,"scouted":true},{"id":"e3","name":"Warg rider","icon":"bolt","count":3200,"boss":false,"scouted":true},{"id":"e4","name":"Shaman","icon":"fire","count":640,"boss":false,"scouted":true},{"id":"e5","name":"Ogre","icon":"shield","count":210,"boss":false,"scouted":true},{"id":"e6","name":"Wyvern","icon":"drop","count":96,"boss":false,"scouted":true},{"id":"e7","name":"","icon":"skull","count":0,"boss":false,"scouted":false}]},
};
