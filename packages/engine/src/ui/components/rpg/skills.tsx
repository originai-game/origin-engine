import type { Texture } from "@origingame/engine";
import { For, Icon, Image, Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import "../hud/slot.en.ogt";
import "./rpg.en.ogt";

export interface RpgSkillsProps {
  /** The bar's slots in the game's order, keyed rows of { id, name, icon, glyph, key, kind, cooldown, time, count, ready, unaffordable, reason, charges, chargesMax, charge }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); key the input as the game names it (1, Q), on a key cap under the slot; kind skill, item (a consumable: a smaller slot with its count on a tag, set apart before the skills; a phone leaves items out) or ultimate (a larger hexagon inside a gold ring that fills with charge, 0 to 1, its share in figures until full; on data-kind); cooldown the share still to run, 0 to 1 (a dark radial sweep over the art with a lit leading edge), with time its figure as the game words it (8); count the items held (×5; 0 hides it); ready lit (a lit edge and glow); unaffordable it cannot be paid for (the art greys under a veil and reason, as the game words it (No mana), stands on it); chargesMax 0 hides the charge pips, else charges of them are lit over the slot. */
  slots?: ReadonlyArray<{ id: string; name: string; icon: Texture | 0; glyph: string; key: string; kind: string; cooldown: number; time: string; count: number; ready: boolean; unaffordable: boolean; reason: string; charges: number; chargesMax: number; charge: number }>;
}

/**
 * The ability bar at the foot of the screen, a row of HudSlot blocks:
 * the consumables in smaller slots, set apart before the skills, then each
 * skill, and the ultimate, larger, at the end. The slot draws itself (its
 * cooldown, cost, charges and key cap are HudSlot's); the bar only
 * sizes and spaces them. A game that wants another bar lays HudSlot
 * out itself. The game owns every cooldown, cost and charge; read only.
 *
 * HudSlot is drawn here: its own port does not exist in this wave, so the
 * bar renders each slot's block (its art, cooldown sweep, veil, charge
 * pips, count tag and key cap) beside it.
 */
export function RpgSkills(props: RpgSkillsProps) {
  return (
    <View class={`flex-row items-end max-w-full gap-x-[0.714rem] compact:gap-x-[0.429rem] text-fg`} role="list" label={t("rpg.skills.name")}>
      <For each={props.slots ?? []} key={(s) => s.id}>
        {(s, i) => (
          <View
            class={`flex-col items-center gap-y-[0.357rem] shrink min-w-0 data-after-items:ml-[0.714rem] compact:data-after-items:ml-0 data-kind-ultimate:ml-[0.429rem] compact:data-kind-ultimate:ml-[0.143rem] compact:data-kind-item:hidden`}
            role="listitem"
            label={s().unaffordable && s().reason ? t("hud.slot.state", { name: s().name, state: s().reason }) : s().name}
            data-kind={s().kind}
            data-after-items={i() > 0 && (props.slots ?? [])[i() - 1].kind === "item" && s().kind !== "item"}
          >
            <View
              class={`group/slot relative w-[3.657rem] h-[3.657rem] compact:w-[2.743rem] compact:h-[2.743rem] data-kind-item:w-[3.2rem] data-kind-item:h-[3.2rem] data-kind-ultimate:w-[5.943rem] data-kind-ultimate:h-[5.943rem] compact:data-kind-ultimate:w-[3.886rem] compact:data-kind-ultimate:h-[3.886rem] data-ready:shadow-[0px_0px_14px_#38e8ff] data-kind-ultimate:data-ready:shadow-[0px_0px_14px_#ffcf5a]`}
              data-kind={s().kind}
              data-ready={s().ready || (s().kind === "ultimate" && s().charge >= 1)}
              data-cooling={s().cooldown > 0 || (s().kind === "ultimate" && s().charge < 1)}
              data-unaffordable={s().unaffordable}
            >
              <Show when={s().chargesMax > 0}>
                <View class="absolute left-0 top-0 flex-row h-[0.357rem]" role="meter" label={t("hud.slot.charges")} valueMin={0} valueMax={s().chargesMax} valueNow={s().charges <= 0 ? 0 : s().charges}>
                  <For each={Array.from({ length: s().chargesMax }, (_, n) => n)} key={(n) => n}>
                    {(n) => (
                      <View class={`w-[0.857rem] h-full mr-[0.214rem] bg-[#ffffff38] data-on:bg-accent data-on:shadow-[0px_0px_6px_#38e8ff]`} data-on={n() < (s().charges <= 0 ? 0 : s().charges)} />
                    )}
                  </For>
                </View>
              </Show>
              <Show when={s().kind === "ultimate"}>
                <View class="absolute inset-0 overflow-hidden rounded-full border-[3px] border-[#ffffff24]" role="meter" label={t("hud.slot.charge")} valueMin={0} valueMax={100} valueNow={s().charge <= 0 ? 0 : s().charge >= 1 ? 100 : s().charge * 100}>
                  <View class="absolute inset-x-0 bottom-0 bg-gold/35" style={{ height: `${Math.min(1, s().charge <= 0 ? 0 : s().charge) * 100}%` }} />
                </View>
              </Show>
              <View class={`absolute inset-0 overflow-hidden rounded-full border border-[#ffffff59] bg-linear-to-b from-[#ffffffbf] via-[#ffffff38] via-45% to-[#ffffff14] shadow-[0px_4.571px_9.143px_#000000a6] data-ready:border-accent data-ready:bg-linear-to-b data-ready:from-[#ffffff] data-ready:to-[#38e8ff] data-kind-ultimate:inset-[10.5%]`}>
                <View class={`absolute inset-[0.143rem] overflow-hidden rounded-full bg-[#1e2637e5] data-cooling:opacity-45 data-unaffordable:opacity-45`}>
                  <Show when={s().icon !== 0} fallback={
                    <View class="absolute inset-0 flex-row items-center justify-center">
                      <Show when={s().glyph}>
                        <Icon class={`size-1/2 text-fg`} name={s().glyph} />
                      </Show>
                    </View>
                  }>
                    <Image class={`absolute inset-0 size-full`} src={s().icon} />
                  </Show>
                  <Show when={s().unaffordable}>
                    <View class="absolute inset-0 bg-[#06091080]" />
                  </Show>
                  <Show when={s().cooldown > 0}>
                    <View class="absolute inset-x-0 top-0 bg-[#060910ad]" style={{ height: `${Math.min(1, s().cooldown) * 100}%` }} />
                  </Show>
                </View>
              </View>
              <Show when={s().cooldown > 0 && s().time}>
                <View class="absolute inset-0 flex-row items-center justify-center">
                  <Text class={`text-[1.714rem] leading-none font-bold tabular-nums whitespace-nowrap text-fg text-stroke-3 text-stroke-bg compact:text-[1.143rem] data-kind-ultimate:text-[1.143rem] compact:data-kind-ultimate:text-[0.929rem]`}>
                    {s().time}
                  </Text>
                </View>
              </Show>
              <Show when={s().unaffordable && s().reason && s().cooldown <= 0}>
                <View class="absolute inset-0 flex-row items-center justify-center p-1">
                  <Text class={`max-w-full text-[0.857rem] leading-none font-bold uppercase text-center text-[#78b2ff] text-stroke-3 text-stroke-bg`}>
                    {s().reason}
                  </Text>
                </View>
              </Show>
              <Show when={s().kind === "ultimate" && s().charge < 1}>
                <View class="absolute inset-0 flex-row items-center justify-center">
                  <Text class={`text-[1.143rem] leading-none font-bold tabular-nums whitespace-nowrap text-fg text-stroke-3 text-stroke-bg compact:text-[0.929rem]`}>
                    {`${formatNumber((s().charge <= 0 ? 0 : s().charge) * 100, { decimals: 0 })}%`}
                  </Text>
                </View>
              </Show>
              <Show when={s().count > 0}>
                <Text class={`absolute -top-[0.429rem] -right-[0.286rem] max-w-[120%] py-0 px-[0.357rem] text-[0.857rem] font-bold tabular-nums whitespace-nowrap overflow-hidden text-fg bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`}>
                  {t("hud.slot.count", { count: formatNumber(s().count) })}
                </Text>
              </Show>
            </View>
            <Show when={s().key}>
              <Text class={`min-w-[1.714rem] max-w-[130%] py-0 px-1 leading-[1.6] text-[0.857rem] font-bold text-center whitespace-nowrap overflow-hidden text-[#0b0f18] bg-linear-to-b from-[#ffffff] to-[#c9d1e2] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] data-kind-ultimate:max-w-[80%] data-kind-ultimate:from-[#ffe9ae] data-kind-ultimate:to-[#ffcf5a]`}>
                {s().key}
              </Text>
            </Show>
          </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgSkillsProps>> = {
  default: {"slots":[{"id":"tonic","name":"Healing tonic","icon":0,"glyph":"potion","key":"Z","kind":"item","cooldown":0,"time":"","count":5,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"ether","name":"Ether","icon":0,"glyph":"drop","key":"X","kind":"item","cooldown":0,"time":"","count":2,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"slash","name":"Ember slash","icon":0,"glyph":"sword","key":"1","kind":"skill","cooldown":0,"time":"","count":0,"ready":true,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"frost","name":"Frost lance","icon":0,"glyph":"bolt","key":"2","kind":"skill","cooldown":0.65,"time":"8","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"arc","name":"Arc step","icon":0,"glyph":"star","key":"3","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":2,"chargesMax":3,"charge":0},{"id":"void","name":"Void rend","icon":0,"glyph":"skull","key":"4","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"No mana","charges":0,"chargesMax":0,"charge":0},{"id":"gale","name":"Gale","icon":0,"glyph":"map","key":"Q","kind":"skill","cooldown":0.18,"time":"2","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"ult","name":"Cataclysm","icon":0,"glyph":"fire","key":"R","kind":"ultimate","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":1}]},
  cooling: {"slots":[{"id":"slash","name":"Ember slash","icon":0,"glyph":"sword","key":"1","kind":"skill","cooldown":1,"time":"12","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"arc","name":"Arc step","icon":0,"glyph":"star","key":"3","kind":"skill","cooldown":0.4,"time":"3","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":3,"charge":0},{"id":"ult","name":"Cataclysm","icon":0,"glyph":"fire","key":"R","kind":"ultimate","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0.35}]},
  unaffordable: {"slots":[{"id":"void","name":"Void rend","icon":0,"glyph":"skull","key":"4","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"No mana","charges":0,"chargesMax":0,"charge":0},{"id":"tonic","name":"Healing tonic","icon":0,"glyph":"potion","key":"Z","kind":"item","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"","charges":0,"chargesMax":0,"charge":0}]},
  empty: {"slots":[]},
  overflow: {"slots":[{"id":"a","name":"A","icon":0,"glyph":"potion","key":"Left Shoulder","kind":"item","cooldown":0,"time":"","count":12345,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"b","name":"B","icon":0,"glyph":"bolt","key":"Right Trigger","kind":"skill","cooldown":0.5,"time":"1:02:44","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"c","name":"C","icon":0,"glyph":"skull","key":"Dpad Up","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"Not enough soul essence","charges":0,"chargesMax":0,"charge":0},{"id":"d","name":"D","icon":0,"glyph":"star","key":"Y","kind":"skill","cooldown":0,"time":"","count":0,"ready":true,"unaffordable":false,"reason":"","charges":7,"chargesMax":9,"charge":0},{"id":"e","name":"E","icon":0,"glyph":"map","key":"B","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"f","name":"F","icon":0,"glyph":"gem","key":"X","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},{"id":"u","name":"U","icon":0,"glyph":"fire","key":"Left Stick + Right Stick","kind":"ultimate","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0.999}]},
};
