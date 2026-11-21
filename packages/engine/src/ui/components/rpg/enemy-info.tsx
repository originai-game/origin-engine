import { For, Icon, Image, Show, Text, View, t } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgEnemyInfoProps {
  /** The enemy's name. */
  name: string;
  /** Its portrait (og.load.texture); 0 is none. */
  portrait: number;
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

/** The HUD's leaning health bar as this sheet uses it (a stand-in for HudBar). */
function EnemyBar(props: { value: number; max: number; label: string }) {
  const frac = props.max > 0 ? Math.min(Math.max(props.value / props.max, 0), 1) : 0;
  return (
    <View class={`relative self-stretch mx-1`}>
      <View
        class={`w-full h-[1.714rem] overflow-hidden skew-x-[-22deg] bg-[#04060cb8] shadow-[0px_0px_0px_1px_#ffffff38,inset_0px_1px_0px_#ffffff24,0px_2px_4px_#00000073]`}
        role="meter"
        label={props.label}
        valueMin={0}
        valueMax={props.max > 0 ? props.max : 1}
        valueNow={props.value <= 0 ? 0 : props.value > props.max ? props.max : props.value}
      >
        <View class={`relative h-full bg-[#de2634]`} style={{ width: `${frac * 100}%` }}>
          <View class={`absolute inset-0 bg-linear-to-b from-[#ffffff8c] to-transparent to-42%`} />
        </View>
      </View>
      <View class={`absolute inset-0 flex-row justify-center items-center`}>
        <Text class={`px-1 text-[0.929rem] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs text-shadow-bg/55 skew-x-[22deg]`}>
          {t("rpg.party.figure", { now: props.value, max: props.max })}
        </Text>
      </View>
    </View>
  );
}

export function RpgEnemyInfo(props: RpgEnemyInfoProps) {
  return (
    <View
      class={`flex-col items-center w-full text-fg text-shadow-xs text-shadow-bg/55 border-rim bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-block border p-2 gap-y-2`}
      label={props.name}
    >
      <View class={`flex-row items-center justify-center w-full shrink-0 min-h-[2.286rem] py-[0.143rem] px-[0.857rem] shadow-[inset_0px_-1px_0px_#343d4f]`}>
        <Text
          role="heading"
          level={3}
          class={`text-[1rem] font-bold text-fg whitespace-nowrap text-shadow-xs text-shadow-bg/55`}
        >
          {t("rpg.enemy.name")}
        </Text>
      </View>
      <Text class={`text-[1rem] font-semibold text-fg text-center`}>{props.name}</Text>
      <Show when={props.portrait !== 0}>
        <View class={`w-[46%] max-w-[6.857rem] aspect-square overflow-hidden compact:hidden`}>
          <Image class={`size-full`} src={String(props.portrait)} label="" />
        </View>
      </Show>
      <Text class={`text-[0.929rem] text-fg-muted text-center`}>{t("rpg.party.level", { level: props.level })}</Text>
      <EnemyBar value={props.health} max={props.healthMax} label={t("rpg.party.health")} />
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
