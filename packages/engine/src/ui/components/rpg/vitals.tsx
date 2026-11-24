import type { Texture } from "@origingame/engine";
import { formatNumber, Icon, Image, packClass, Show, t, Text, View } from "@origingame/engine/ui";
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
    <View class={`group/plate relative w-fit flex-row items-center gap-x-2.5 text-fg data-down:opacity-dim`} role="group" label={props.name} data-down={props.down} data-plate-large={props.plate === "large"} data-plate-compact={props.plate === "compact"}>
      <View class={`shrink-0 size-16 overflow-hidden rounded-capsule bg-tile shadow-sunk inset-ring-hairline inset-ring-border items-center justify-center group-data-plate-large/plate:size-24 compact:group-data-plate-large/plate:size-18`}>
        <Show when={props.portrait !== 0} fallback={
          <Icon class="size-1/2 text-fg-muted" name="person" />
        }>
          <Image class={`size-full object-cover object-top`} src={props.portrait} />
        </Show>
      </View>
      <Show when={props.level !== 0}>
        <View class={`absolute left-0 bottom-0 size-6 group-data-plate-large/plate:size-7 flex-row items-center justify-center ${packClass("Chip", "frame")} rounded-capsule`}>
          <Text class={`text-caption font-bold tabular-nums`}>{formatNumber(props.level)}</Text>
        </View>
      </Show>
      <View class={`flex-col gap-1 w-33.5 group-data-plate-large/plate:w-60 group-data-plate-large/plate:gap-y-1.5 compact:group-data-plate-compact/plate:w-24 compact:group-data-plate-large/plate:w-40.75`}>
        <Show when={props.plate === "large" && props.name !== ""}>
          <Text class="text-sm font-bold text-fg text-shadow-tag truncate">{props.name}</Text>
        </Show>
        <View class={`relative flex-row items-center justify-center rounded-capsule overflow-hidden h-2.5 group-data-plate-large/plate:h-5 compact:group-data-plate-large/plate:h-4 bg-track shadow-sunk inset-ring-hairline inset-ring-border`} role="meter" label={t("rpg.party.health")} valueText={t("rpg.party.figure", { now: props.health, max: props.healthMax })}>
          <View class="absolute left-0 inset-y-0 rounded-capsule bg-health" style={{ width: `${bar(props.healthMax > 0 ? props.health / props.healthMax : 0)}%` }} />
          <Show when={props.plate === "large"}>
            <Text class={`relative whitespace-nowrap px-1.5 text-xs font-bold tabular-nums text-fg text-stroke-2 text-stroke-bg text-shadow-tag`}>
              {t("rpg.party.figure", { now: props.health, max: props.healthMax })}
            </Text>
          </Show>
        </View>
        <Show when={props.manaMax !== 0}>
          <View class={`relative flex-row items-center justify-center rounded-capsule overflow-hidden h-2 group-data-plate-large/plate:h-4.5 compact:group-data-plate-large/plate:h-3.5 bg-track shadow-sunk inset-ring-hairline inset-ring-border`} role="meter" label={t("rpg.party.mana")} valueText={t("rpg.party.figure", { now: props.mana, max: props.manaMax })}>
            <View class="absolute left-0 inset-y-0 rounded-capsule bg-mana" style={{ width: `${bar(props.manaMax > 0 ? props.mana / props.manaMax : 0)}%` }} />
            <Show when={props.plate === "large"}>
              <Text class={`relative whitespace-nowrap px-1.5 text-xs font-bold tabular-nums text-fg text-stroke-2 text-stroke-bg text-shadow-tag`}>
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
