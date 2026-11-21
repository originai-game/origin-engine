import { For, Icon, Image, Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import { HudBar } from "../hud/bar";
import "./leader.en.ogt";

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
        <HudBar value={props.health} max={props.healthMax} ghost={props.ghost} low={low()} />
        <Show when={props.manaMax > 0}>
          <View class="w-[86%]">
            <HudBar value={props.mana} max={props.manaMax} kind="mana" />
          </View>
        </Show>
        <Show when={props.staminaMax > 0}>
          <View class="w-[72%]">
            <HudBar value={props.stamina} max={props.staminaMax} kind="stamina" segment={props.segment ?? 0} />
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
