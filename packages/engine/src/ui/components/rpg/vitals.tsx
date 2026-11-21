import type { Texture } from "@origingame/engine";
import { formatNumber, Image, packClass, Show, t, Text, View } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgVitalsProps {
  /** The character's name; read out, and shown on the large plate. */
  name: string;
  /** A head-and-shoulders texture (og.load.texture); 0 is none. */
  portrait: Texture | 0;
  /** The level on the badge; 0 hides the badge. */
  level: number;
  /** Health now. */
  health: number;
  /** Most health; the bar shows health over it. */
  healthMax: number;
  /** Mana now. */
  mana: number;
  /** Most mana; 0 hides the mana bar. */
  manaMax: number;
  /** large (the player's plate, figures on the bars) or compact (a party row). */
  plate: string;
  /** Out of the fight: the plate greys. */
  down: boolean;
}

/**
 * A character's vitals: a round portrait with a level badge and health and
 * mana bars beside it. large is the player's plate, with the figures on the
 * bars; compact is a party row. Read only.
 */
export function RpgVitals(props: RpgVitalsProps) {
  const bar = (fill: number) => Math.min(Math.max(fill, 0), 1) * 100;
  return (
    <View class={`group/plate relative w-fit flex-row text-fg`} role="group" label={props.name} data-down={props.down} data-plate-large={props.plate === "large"} data-plate-compact={props.plate === "compact"}>
      <View class={`shrink size-16 overflow-hidden rounded-capsule group-data-plate-large/plate:size-[10.971rem] compact:group-data-plate-large/plate:size-[7.2rem]`}>
        <Show when={props.portrait !== 0}>
          <Image class={`size-full`} src={props.portrait} />
        </Show>
      </View>
      <Show when={props.level !== 0}>
        <View class={`absolute left-0 top-[3.429rem] size-6 group-data-plate-large/plate:top-[8.686rem] group-data-plate-large/plate:size-8 compact:group-data-plate-large/plate:top-[4.914rem] bg-surface-raised shadow-[inset_0px_0px_0px_1px_#4e586c] text-fg rounded-capsule flex-row items-center justify-center ${packClass("Chip", "frame")}`}>
          <Text class={`text-caption font-bold tabular-nums`}>{formatNumber(props.level)}</Text>
        </View>
      </Show>
      <View class={`flex-col gap-0.5 w-[9.6rem] group-data-plate-large/plate:w-[17.829rem] group-data-plate-large/plate:gap-y-1 compact:group-data-plate-compact/plate:w-24 compact:group-data-plate-large/plate:w-[11.657rem]`}>
        <View class={`relative flex-row items-center justify-center overflow-hidden h-2 group-data-plate-large/plate:h-[1.371rem] group-data-plate-large/plate:min-h-6 compact:group-data-plate-large/plate:h-[1.029rem] bg-[#04060cb8]`} role="meter" label={t("rpg.party.health")} valueText={t("rpg.party.figure", { now: props.health, max: props.healthMax })}>
          <View class="absolute left-0 inset-y-0 bg-health" style={{ width: `${bar(props.healthMax > 0 ? props.health / props.healthMax : 0)}%` }} />
          <Show when={props.plate === "large"}>
            <Text class={`relative whitespace-nowrap px-1 text-[0.857rem] font-bold tabular-nums text-fg text-stroke-3 text-stroke-bg`}>
              {t("rpg.party.figure", { now: props.health, max: props.healthMax })}
            </Text>
          </Show>
        </View>
        <Show when={props.manaMax !== 0}>
          <View class={`relative flex-row items-center justify-center overflow-hidden h-2 group-data-plate-large/plate:h-[1.371rem] group-data-plate-large/plate:min-h-6 compact:group-data-plate-large/plate:h-[1.029rem] bg-[#04060cb8]`} role="meter" label={t("rpg.party.mana")} valueText={t("rpg.party.figure", { now: props.mana, max: props.manaMax })}>
            <View class="absolute left-0 inset-y-0 bg-mana" style={{ width: `${bar(props.mana / props.manaMax)}%` }} />
            <Show when={props.plate === "large"}>
              <Text class={`relative whitespace-nowrap px-1 text-[0.857rem] font-bold tabular-nums text-fg text-stroke-3 text-stroke-bg`}>
                {t("rpg.party.figure", { now: props.mana, max: props.manaMax })}
              </Text>
            </Show>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgVitalsProps>> = {
  default: {"name":"Brenna","portrait":0,"level":12,"health":842,"healthMax":842,"mana":237,"manaMax":237,"plate":"large","down":false},
  compact: {"name":"Idris","portrait":0,"level":11,"health":40,"healthMax":82,"mana":30,"manaMax":54,"plate":"compact","down":false},
};
