import { For, Icon, Image, Show, Text, View, t } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgEffectsProps {
  /** The buffs and debuffs on a character in the game's order, keyed rows of { id, name, icon, glyph, remaining, label, harmful }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); remaining the share of its time still to run, 0 to 1 (a dark radial sweep over the art; negative: no timer); label its time or stacks as the game words it, on a tag (0:12, ×3), empty for none; harmful a debuff (on data-harmful, never colour alone: the edge and the tag's ink both change). */
  effects?: ReadonlyArray<{ id: string; name: string; icon: number; glyph: string; remaining: number; label: string; harmful: boolean }>;
}

/**
 * The buffs and debuffs on the leader, under the vitals: a row of effect
 * tiles, each a small tile with two corners cut, its edge green for a boon
 * and red for a harm, the time still to run as a dark sweep over its art
 * and its time or stacks on a tag under it. The row only spaces them; a
 * game that wants another arrangement lays the tile out itself. The game
 * owns every timer; read only.
 */

/** The HUD's effect tile as this row uses it (a stand-in for HudEffect). */
function EffectTile(props: { name: string; icon: number; glyph: string; remaining: number; label: string; harmful: boolean }) {
  return (
    <View
      class={`group/effect flex-col items-center gap-y-[0.214rem] w-fit max-w-full text-fg text-shadow-xs text-shadow-bg/55`}
      role="group"
      label={props.name}
      description={props.label}
      data-harmful={props.harmful}
    >
      <View
        class={`relative size-[2.143rem] rounded-tl-[5.714px] rounded-tr-none rounded-br-[5.714px] rounded-bl-none bg-[#55e39a] group-data-harmful/effect:bg-[#ff4d5e]`}
      >
        <Show when={props.icon !== 0} fallback={
            <View class={`absolute inset-[2px] flex-row justify-center items-center bg-linear-to-b from-[#171f30d6] to-[#080c16e0] rounded-tl-[5.714px] rounded-tr-none rounded-br-[5.714px] rounded-bl-none`}>
              <Show when={props.glyph}>
                <Icon class={`w-[60%] h-[60%] text-fg`} name={props.glyph} />
              </Show>
            </View>
        }>
          <Image class={`absolute inset-[2px] rounded-tl-[5.714px] rounded-tr-none rounded-br-[5.714px] rounded-bl-none`} src={String(props.icon)} label="" />
        </Show>
        <Show when={props.remaining >= 0}>
          <View
            class={`absolute inset-x-0 top-0 bg-[#060910ad] rounded-bl-[5.714px]`}
            style={{ height: `${(1 - Math.min(props.remaining >= 1 ? 1 : props.remaining, 1)) * 100}%` }}
          />
        </Show>
      </View>
      <Show when={props.label}>
        <Text
          class={`max-w-[4.571rem] px-[0.357rem] text-[0.857rem] font-bold tabular-nums whitespace-nowrap overflow-hidden bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none group-data-harmful/effect:text-[#ff4d5e] compact:px-[0.214rem]`}
        >
          {props.label}
        </Text>
      </Show>
    </View>
  );
}

export function RpgEffects(props: RpgEffectsProps) {
  return (
    <View
      class={`flex-row flex-wrap max-w-full text-fg border-rim bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-block border p-2 gap-x-2 gap-y-1 compact:gap-x-[0.429rem]`}
      role="list"
      label={t("rpg.effects.name")}
    >
      <For each={props.effects ?? []} key={(e) => e.id}>
        {(e) => (
        <View role="listitem">
          <EffectTile name={e().name} icon={e().icon} glyph={e().glyph} remaining={e().remaining} label={e().label} harmful={e().harmful} />
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgEffectsProps>> = {
  default: {"effects":[{"id":"haste","name":"Haste","icon":0,"glyph":"bolt","remaining":0.7,"label":"0:12","harmful":false},{"id":"ward","name":"Rune ward","icon":0,"glyph":"shield","remaining":0.42,"label":"×3","harmful":false},{"id":"hex","name":"Hex","icon":0,"glyph":"skull","remaining":0.3,"label":"4.2","harmful":true}]},
  untimed: {"effects":[{"id":"blessed","name":"Blessed","icon":0,"glyph":"star","remaining":-1,"label":"","harmful":false}]},
  expiring: {"effects":[{"id":"haste","name":"Haste","icon":0,"glyph":"bolt","remaining":0.04,"label":"0:01","harmful":false},{"id":"burn","name":"Burning","icon":0,"glyph":"fire","remaining":1,"label":"8.0","harmful":true}]},
  empty: {"effects":[]},
  overflow: {"effects":[{"id":"e1","name":"A","icon":0,"glyph":"bolt","remaining":0.9,"label":"12:34:56","harmful":false},{"id":"e2","name":"B","icon":0,"glyph":"fire","remaining":0.1,"label":"×99 59.9","harmful":true},{"id":"e3","name":"C","icon":0,"glyph":"drop","remaining":0.5,"label":"1","harmful":true},{"id":"e4","name":"D","icon":0,"glyph":"heart","remaining":0.2,"label":"2","harmful":false},{"id":"e5","name":"E","icon":0,"glyph":"star","remaining":0.4,"label":"3","harmful":false},{"id":"e6","name":"F","icon":0,"glyph":"skull","remaining":0.6,"label":"4","harmful":true},{"id":"e7","name":"G","icon":0,"glyph":"gem","remaining":0.8,"label":"5","harmful":false},{"id":"e8","name":"H","icon":0,"glyph":"key","remaining":1,"label":"6","harmful":false},{"id":"e9","name":"I","icon":0,"glyph":"map","remaining":0.3,"label":"7","harmful":false},{"id":"e10","name":"J","icon":0,"glyph":"coin","remaining":0.7,"label":"8","harmful":true}]},
};
