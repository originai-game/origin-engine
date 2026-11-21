import type { Texture } from "@origingame/engine";
import { For, Icon, Image, Show, Text, View, t } from "@origingame/engine/ui";
import { HudBar } from "../hud/bar";
import "./rpg.en.ogt";

export interface RpgEnemyInfoProps {
  /** The enemy's name. */
  name: string;
  /** Its portrait (og.load.texture); 0 is none. */
  portrait: Texture | 0;
  /** Its level. */
  level: number;
  /** Health now. */
  health: number;
  /** Most health. */
  healthMax: number;
  /** Status effects and traits, keyed rows of { id, icon, label, tone }: tone harm (a debuff on it), boon (a buff), weak (vulnerable to) or strong (resistant to). */
  effects?: ReadonlyArray<{ id: string; icon: string; label: string; tone: string }>;
}

/**
 * The targeted enemy: name, portrait, level, a health bar with its
 * figure and its status effects. Read only.
 */

export function RpgEnemyInfo(props: RpgEnemyInfoProps) {
  return (
    <View
      class={`flex-col items-center w-full text-fg text-stroke-3 text-stroke-bg border-rim bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-block border p-2 gap-y-2`}
      label={props.name}
    >
      <View class={`flex-row items-center justify-center w-full shrink-0 min-h-[2.286rem] py-[0.143rem] px-[0.857rem] shadow-[inset_0px_-1px_0px_#343d4f]`}>
        <Text
          role="heading"
          level={3}
          class={`text-[1rem] font-bold text-fg whitespace-nowrap text-stroke-3 text-stroke-bg`}
        >
          {t("rpg.enemy.name")}
        </Text>
      </View>
      <Text class={`text-[1rem] font-semibold text-fg text-center`}>{props.name}</Text>
      <Show when={props.portrait !== 0}>
        <View class={`w-[46%] max-w-[6.857rem] aspect-square overflow-hidden compact:hidden`}>
          <Image class={`size-full`} src={props.portrait} label="" />
        </View>
      </Show>
      <Text class={`text-[0.929rem] text-fg-muted text-center`}>{t("rpg.party.level", { level: props.level })}</Text>
      <View class="relative self-stretch mx-1">
        <HudBar value={props.health} max={props.healthMax} showValue />
      </View>
      <Show when={(props.effects ?? []).length}>
        <View class={`flex-row flex-wrap items-start w-full gap-1`} role="list">
          <For each={props.effects ?? []} key={(e) => e.id}>
            {(e) => (
            <View
              class={`group/tone flex-row items-center gap-2 w-full compact:w-fit`}
              role="listitem"
              data-harm={e().tone === "harm"}
              data-boon={e().tone === "boon"}
              data-weak={e().tone === "weak"}
              data-strong={e().tone === "strong"}
            >
              <Icon
                class={`size-[1.714rem] shrink-0 text-fg group-data-harm/tone:text-[#ff4d5e] group-data-boon/tone:text-[#55e39a] group-data-weak/tone:text-[#ffb547] group-data-strong/tone:text-[#4fd8ff]`}
                name={e().icon}
              />
              <Text
                class={`min-w-0 text-[0.929rem] whitespace-nowrap overflow-hidden text-ellipsis group-data-harm/tone:text-[#ff4d5e] group-data-boon/tone:text-[#55e39a] group-data-weak/tone:text-[#ffb547] group-data-strong/tone:text-[#4fd8ff]`}
              >
                {e().label}
              </Text>
            </View>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgEnemyInfoProps>> = {
  default: {"name":"Cultist","portrait":0,"level":6,"health":72,"healthMax":120,"effects":[{"id":"stun","icon":"bolt","label":"Stunned (1)","tone":"harm"},{"id":"fire","icon":"fire","label":"Vulnerable to Fire","tone":"weak"}]},
};
