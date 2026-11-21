import { For, Icon, Image, Show, Text, View, createMemo, formatNumber, t } from "@origingame/engine/ui";
import "./leader.en.ogt";
import "../hud/bar.en.ogt";

export interface RpgLeaderProps {
  /** The leader's name; empty hides it. */
  name: string;
  /** The level beside the name; 0 hides it. */
  level: number;
  /** A head-and-shoulders texture (og.load.texture); 0 draws the person glyph. */
  portrait: number;
  /** Health now, shown as a figure over its most and as the fat bar. */
  health: number;
  /** Most health. */
  healthMax: number;
  /** Health before the latest loss: a warm ghost behind the fill that drains after it (at or under health: none). */
  ghost: number;
  /** Mana now. */
  mana: number;
  /** Most mana; 0 hides the mana bar. */
  manaMax: number;
  /** Stamina (or energy) now. */
  stamina: number;
  /** Most stamina; 0 hides the stamina bar. */
  staminaMax: number;
  /** Stamina per cell of the stamina bar (one dodge's cost); 0 draws it whole. Default 0. */
  segment?: number;
  /** The share of health under which it is low: the fill turns to danger and the bar's rim glows (0.25). Default 0.25. */
  low?: number;
}

/**
 * The party leader's vitals at the screen's top corner, built from the RPG
 * HUD's blocks: a portrait in a large octagon, the name and level, health
 * as a figure over its most, and three bars: a fat health bar that leaves
 * a warm ghost when it drops and turns to danger when low, a mana bar and
 * a stamina bar cut into cells. Read only.
 */

/**
 * One HudBar block, inlined (hud/bar.svelte is not ported yet): a dark
 * well with a lit rim, a warm ghost behind the fill, the fill in its
 * kind's colour, the value over its most, cells cut across the well and
 * the figure upright over it when showValue. The skew and the gloss
 * gradients stand in for the leaning bar's paint.
 */
function VitalBar(props: { value: number; max: number; ghost: number; segment: number; low: boolean; showValue: boolean; kind: string; name: string; height: number }) {
  const cuts = createMemo(() => {
    const n = props.segment > 0 && props.max > 0 ? Math.max(1, Math.round(props.max / props.segment)) : 1;
    const lines: number[] = [];
    for (let k = 1; k < n; k++) lines.push(k / n);
    return lines;
  });
  const fill = (v: number) => (props.max <= 0 || v <= 0 ? 0 : v >= props.max ? 1 : v / props.max);
  return (
    <View class={`group/bar relative w-full overflow-hidden bg-[#04060cb8] shadow-[0px_0px_0px_1px_#ffffff38,inset_0px_1px_0px_#ffffff24,0px_2px_4px_#00000073] data-low:shadow-[0px_0px_0px_1px_#ff4d5e,0px_0px_12px_#ff4d5eb3]`} role="meter" label={props.name} valueMin={0} valueMax={props.max > 0 ? props.max : 1} valueNow={props.value <= 0 ? 0 : props.max > 0 && props.value >= props.max ? props.max : props.value} data-low={props.low} style={{ height: `${props.height}px` }}>
      <View class="absolute inset-y-0 left-0 bg-gold/55" style={{ width: `${fill(props.ghost <= props.value ? 0 : props.ghost) * 100}%` }} />
      <View class={`absolute inset-y-0 left-0 shadow-[inset_0px_2px_0px_#ffffff8c] data-life:bg-[#ff4d5e] data-mana:bg-[#5b9dff] data-stamina:bg-gold group-data-low/bar:bg-danger`} data-life={props.kind === "life"} data-mana={props.kind === "mana"} data-stamina={props.kind === "stamina"} style={{ width: `${fill(props.value) * 100}%` }} />
      <For each={cuts()}>
        {(p) => (
        <View class="absolute top-0 bottom-0 w-[0.214rem] bg-[#04060ce0]" style={{ left: `${p() * 100}%` }} />
        )}
      </For>
      <Show when={props.showValue}>
        <View class="absolute inset-0 flex-row items-center justify-center px-1">
          <Text class="text-[0.857rem] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs text-shadow-bg/55">
            {t("hud.bar.figure", { now: props.value, max: props.max })}
          </Text>
        </View>
      </Show>
    </View>
  );
}

export function RpgLeader(props: RpgLeaderProps) {
  const low = () => props.healthMax > 0 && props.health <= props.healthMax * (props.low ?? 0.25);
  return (
    <View class={`flex-row items-center gap-x-[0.857rem] w-[23.571rem] max-w-full text-fg text-shadow-xs text-shadow-bg/55 compact:gap-x-2 compact:w-full`} role="group" label={props.name !== "" ? props.name : t("rpg.party.health")}>
      {/* Portrait: a large HudPortrait, lifted off the scene; the octagon
          cut stands in as a square plate with a lit edge. */}
      <View class="shrink-0 p-[2px] bg-[#ffffff59] shadow-[0px_5px_10px_#000000a6] size-[4.857rem] compact:size-[3.143rem]">
        <View class="relative size-full overflow-hidden bg-[#1e2637e5]">
          <Show when={props.portrait} fallback={
              <View class="absolute inset-0 flex-row items-center justify-center">
                <Icon class="size-1/2 text-fg-muted" name="person" />
              </View>
          }>
            <Image class="absolute inset-0 size-full" src={String(props.portrait)} />
          </Show>
        </View>
      </View>
      {/* Body: the name and level take the row first, the figure at its
          end; the bars, each shorter and thinner than the one above. */}
      <View class="flex-col grow min-w-0 gap-y-[0.357rem] compact:gap-y-[0.286rem]">
        <View class="flex-row flex-wrap items-baseline gap-x-1 min-w-0">
          <View class="flex-row items-baseline gap-x-1 min-w-0 shrink">
            <Show when={props.name !== ""}>
              <Text class={`min-w-0 text-[1.214rem] font-semibold whitespace-nowrap overflow-hidden text-fg compact:text-[1.143rem]`}>
                {props.name}
              </Text>
            </Show>
            <Show when={props.level > 0}>
              <Text class={`shrink text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#38e8ff]`}>
                {t("rpg.leader.level", { level: props.level })}
              </Text>
            </Show>
          </View>
          <Show when={props.healthMax > 0}>
            <View class="flex-row items-baseline gap-x-1 ml-auto shrink">
              <Text class={`shrink text-[1.214rem] leading-none font-bold tabular-nums whitespace-nowrap text-fg data-low:text-[#ff4d5e] compact:text-[1.143rem]`} data-low={low()}>
                {formatNumber(props.health < 0 ? 0 : props.health)}
              </Text>
              <Text class={`shrink text-[1.143rem] font-bold tabular-nums whitespace-nowrap text-fg-muted compact:text-[0.929rem]`}>
                {t("rpg.leader.max", { max: formatNumber(props.healthMax) })}
              </Text>
            </View>
          </Show>
        </View>
        <VitalBar value={props.health} max={props.healthMax} ghost={props.ghost} segment={0} low={low()} showValue={false} kind="life" name={t("hud.bar.health")} height={16} />
        <Show when={props.manaMax > 0}>
          <View class="w-[86%]">
            <VitalBar value={props.mana} max={props.manaMax} ghost={0} segment={0} low={false} showValue={false} kind="mana" name={t("hud.bar.mana")} height={8} />
          </View>
        </Show>
        <Show when={props.staminaMax > 0}>
          <View class="w-[72%]">
            <VitalBar value={props.stamina} max={props.staminaMax} ghost={0} segment={props.segment ?? 0} low={false} showValue={false} kind="stamina" name={t("hud.bar.stamina")} height={6} />
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgLeaderProps>> = {
  default: {"name":"Ysolde","level":42,"portrait":0,"health":2184,"healthMax":3420,"ghost":2668,"mana":72,"manaMax":100,"stamina":5,"staminaMax":8,"segment":1,"low":0.25},
  full: {"name":"Ysolde","level":42,"portrait":0,"health":3420,"healthMax":3420,"ghost":0,"mana":100,"manaMax":100,"stamina":8,"staminaMax":8,"segment":1,"low":0.25},
  low: {"name":"Ysolde","level":42,"portrait":0,"health":610,"healthMax":3420,"ghost":1500,"mana":12,"manaMax":100,"stamina":1,"staminaMax":8,"segment":1,"low":0.25},
  critical: {"name":"Ysolde","level":42,"portrait":0,"health":42,"healthMax":3420,"ghost":300,"mana":0,"manaMax":100,"stamina":0,"staminaMax":8,"segment":1,"low":0.25},
  empty: {"name":"","level":0,"portrait":0,"health":0,"healthMax":0,"ghost":0,"mana":0,"manaMax":0,"stamina":0,"staminaMax":0,"segment":0,"low":0.25},
  overflow: {"name":"Ysolde Varrenhallow of the Ninefold Gate and the Ashen Vale","level":1234,"portrait":0,"health":123456,"healthMax":999999,"ghost":200000,"mana":99999,"manaMax":99999,"stamina":37,"staminaMax":40,"segment":4,"low":0.25},
};
