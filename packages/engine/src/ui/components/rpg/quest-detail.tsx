import type { Texture } from "@origingame/engine";
import { For, Icon, Image, Scroll, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgQuestDetailProps {
  /** The quest's name, in the heading band. */
  name: string;
  /** Its story so far, as the game writes it. */
  summary: string;
  /** Keyed rows of { id, text, done, optional, step }: step 0 is a stage, 1 a task under it; done ticks it, optional marks it so. */
  objectives?: ReadonlyArray<{ id: string; text: string; done: boolean; optional: boolean; step: number }>;
  /** Keyed rows of { id, icon, label, detail }: detail a second, quieter line (Unique relic); empty hides it. */
  rewards?: ReadonlyArray<{ id: string; icon: string; label: string; detail: string }>;
  /** What kind of quest it is, as the game words it (Main quest), over the name; empty hides it. */
  label: string;
  /** The HUD follows this quest: its action reads Stop tracking. */
  tracked: boolean;
  /** Show the Track (or Stop tracking) action, which reports track. */
  canTrack: boolean;
  /** Show the Show on map action, which reports locate. */
  canLocate: boolean;
  /** A texture beside the rewards (og.load.texture), with its own space; 0 is none. */
  scenery: Texture | 0;
  /** Track or Stop tracking. */
  onTrack?: (e: UiEvent) => void;
  /** Show on map. */
  onLocate?: (e: UiEvent) => void;
}

/**
 * One quest: its kind in gold small capitals over its name on the heading
 * band (over the scenery, shaded into the glass, when the game gives one),
 * the story so far, its objectives as stages with their tasks on a rail
 * under them, and its rewards as cut chips, each with its icon in an
 * octagon socket (one with a detail, a unique reward, lit gold). A done
 * objective is struck with its diamond filled in the success colour; the
 * first stage not done is lit gold. The body scrolls inside the frame's
 * height, fading out above the actions (Show on map, Track or Stop
 * tracking), which stay at its foot; they report locate and track. Track
 * is the primary action; once the quest is tracked it reads Stop tracking
 * on a quiet secondary button. With neither action it is read only.
 */
export function RpgQuestDetail(props: RpgQuestDetailProps) {
  return (
    <View class={`group/frame flex-col size-full min-h-0 overflow-hidden rounded-tl-[11.429px] rounded-tr-none rounded-br-[11.429px] rounded-bl-none bg-[#101725f0] shadow-[inset_0px_0px_0px_1px_#ffffff1f,inset_0px_1px_0px_0px_#ffffff2e,0px_6.857px_20.571px_#000000a6] text-fg`} label={props.name} data-with-scenery={props.scenery !== 0}>
      {/* Band */}
      <View class={`relative shrink-0 w-full min-h-[6.857rem] group-data-with-scenery/frame:min-h-[13.714rem]`}>
        <Show when={props.scenery !== 0}>
          <Image class={`absolute inset-0 size-full`} src={props.scenery} />
          <View class="absolute inset-0 bg-linear-to-b from-transparent via-[#060d1cb3] via-55% to-[#060d1c]" />
        </Show>
        <View class="absolute bottom-0 left-0 flex-col gap-0.5 min-w-0 pt-3 pb-2 px-4 compact:px-3">
          <Show when={props.label}>
            <Text class={`text-caption font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-gold`}>{props.label}</Text>
          </Show>
          <Text class={`min-w-0 text-[2.429rem] leading-tight text-fg text-stroke-3 text-stroke-bg compact:text-[1.714rem]`} role="heading" level={2}>
            {props.name}
          </Text>
        </View>
      </View>
      {/* Body */}
      <View class="relative grow min-h-0 w-full">
        <Scroll class="absolute inset-0">
          <View class={`flex-col gap-3 pt-1 px-4 pb-4 compact:px-3`}>
            <Show when={props.summary}>
              <Text class={`max-w-[30rem] text-fg-muted`}>{props.summary}</Text>
            </Show>
            <Show when={(props.objectives ?? []).length}>
              <View class={`flex-col gap-1`} role="group" label={t("rpg.quest.objectives")}>
                <View class="flex-row items-center gap-1" role="heading">
                  <Text class={`text-caption font-semibold tracking-[0.14em] uppercase text-fg-muted`}>{t("rpg.quest.objectives")}</Text>
                  <View class="grow h-px bg-rim" />
                </View>
                <View class={`flex-col`} role="list">
                  <For each={props.objectives ?? []} key={(o) => o.id}>
                    {(o, i) => (
                      <View class={`group/obj flex-row items-start gap-3 py-0.5 min-w-0 data-step1:ml-1 data-step1:pl-4 data-step1:border-l data-step1:border-rim`} role="listitem" data-optional={o().optional} data-step1={o().step === 1} data-current={o().step === 0 && !o().done && (i() === 0 || (props.objectives ?? [])[i() - 1].done)} data-done={o().done}>
                        <View class={`shrink-0 rotate-45 size-2 mx-[0.143rem] mt-1 data-step1:size-1 shadow-[inset_0px_0px_0px_2.288px_#ffcf5a] data-current:bg-gold data-current:shadow-[inset_0px_0px_0px_2.288px_#ffcf5a,0px_0px_10px_#ffcf5acc] data-done:bg-success data-done:shadow-[inset_0px_0px_0px_2.288px_#30d158,0px_0px_6px_#30d15899]`} />
                        <Text class={`grow min-w-0 group-data-step1/obj:text-fg-muted group-data-step1/obj:font-semibold group-data-done/obj:text-fg-subtle group-data-done/obj:line-through`}>
                          {o().text}
                        </Text>
                        <Show when={o().optional}>
                          <Text class={`shrink py-0 px-1 whitespace-nowrap text-caption tracking-[0.14em] uppercase bg-[#0c10178c] shadow-[inset_0px_0px_0px_1px_#ffffff2e] rounded-capsule`}>
                            {t("rpg.quest.optional")}
                          </Text>
                        </Show>
                      </View>
                    )}
                  </For>
                </View>
              </View>
            </Show>
            <Show when={(props.rewards ?? []).length}>
              <View class={`flex-col gap-1 p-2 rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none bg-[#0c1017b8] shadow-[inset_0px_0px_0px_1px_#ffffff14,inset_0px_1px_0px_0px_#ffffff1f]`} role="group" label={t("rpg.quest.rewards")}>
                <View class="flex-row items-center gap-1" role="heading">
                  <Text class={`text-caption font-semibold tracking-[0.14em] uppercase text-fg-muted`}>{t("rpg.quest.rewards")}</Text>
                  <View class="grow h-px bg-rim" />
                </View>
                <View class={`flex-row flex-wrap items-stretch gap-2`} role="list">
                  <For each={props.rewards ?? []} key={(r) => r.id}>
                    {(r) => (
                      <View class={`group/reward flex-row items-center gap-3 grow min-w-[17.829rem] compact:min-w-0 compact:w-full py-1 pl-1 pr-3 rounded-tl-[11.429px] rounded-tr-none rounded-br-[11.429px] rounded-bl-none bg-[#101725d9] shadow-[inset_0px_0px_0px_1px_#ffffff24,inset_0px_1px_0px_0px_#ffffff09] data-unique:shadow-[inset_0px_0px_0px_1px_#ffcf5a73,inset_3.429px_0px_0px_0px_#ffcf5a]`} role="listitem" data-unique={r().detail !== ""}>
                        <View class={`shrink-0 size-12 rounded-[6.858px] p-[1.5px] bg-linear-to-b from-[#ffffff80] via-[#ffffff1f] via-50% to-[#ffffff0f] data-unique:from-[#fff3cf] data-unique:via-[#ffcf5a] data-unique:via-45% data-unique:to-[#ffcf5a66]`}>
                          <View class={`size-full rounded-[5.143px] flex-row items-center justify-center bg-[#1f2430] data-unique:bg-[#3a2f0e]`}>
                            <Icon class={`size-6 text-fg data-unique:text-gold`} name={r().icon} />
                          </View>
                        </View>
                        <View class={`flex-col min-w-0`}>
                          <Text class={`font-bold text-fg`}>{r().label}</Text>
                          <Show when={r().detail}>
                            <Text class={`text-caption text-fg-muted group-data-unique/reward:text-gold`}>{r().detail}</Text>
                          </Show>
                        </View>
                      </View>
                    )}
                  </For>
                </View>
              </View>
            </Show>
          </View>
        </Scroll>
        <Show when={props.canTrack || props.canLocate}>
          <View class="absolute bottom-0 inset-x-0 h-6 bg-linear-to-b from-transparent to-[#101725f0]" />
        </Show>
      </View>
      <Show when={props.canTrack || props.canLocate}>
        <View class={`shrink flex-row flex-wrap justify-end gap-2 pt-2 px-4 pb-3 compact:px-3`}>
          <Show when={props.canLocate}>
            <View class={`min-h-11 flex-row items-center justify-center gap-1 px-2 rounded-[4.576px] border border-rim text-fg hover:bg-[#ffffff0f] disabled:opacity-45`} role="button" label={t("rpg.quest.locate")} onPress={(e) => props.onLocate?.(e)}>
              <Icon class="shrink size-4" name="map" />
              <Text class={`text-[0.929rem] font-bold tracking-[0.14em] uppercase`}>{t("rpg.quest.locate")}</Text>
            </View>
          </Show>
          <Show when={props.canTrack && props.tracked} fallback={
            <Show when={props.canTrack}>
              <View class={`min-h-11 flex-row items-center justify-center gap-1 px-3 rounded-[4.576px] bg-accent text-accent-ink shadow-[inset_0px_2px_0px_#ffffff59] hover:bg-white`} role="button" label={t("rpg.quest.track")} onPress={(e) => props.onTrack?.(e)}>
                <Text class={`text-[0.929rem] font-bold tracking-[0.14em] uppercase`}>{t("rpg.quest.track")}</Text>
              </View>
            </Show>
          }>
            <View class={`min-h-11 flex-row items-center justify-center gap-1 px-2 rounded-[4.576px] border border-rim text-fg hover:bg-[#ffffff0f] disabled:opacity-45`} role="button" label={t("rpg.quest.untrack")} onPress={(e) => props.onTrack?.(e)}>
              <Text class={`text-[0.929rem] font-bold tracking-[0.14em] uppercase`}>{t("rpg.quest.untrack")}</Text>
            </View>
          </Show>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgQuestDetailProps>> = {
  default: {"name":"The Hollow Crown","summary":"The crown is said to lie within the ruins.","objectives":[{"id":"o1","text":"Enter the ruins","done":false,"optional":false,"step":0},{"id":"o2","text":"Find the crown","done":true,"optional":false,"step":1}],"rewards":[{"id":"g","icon":"coin","label":"1,200 Gold","detail":""}],"scenery":0,"label":"Main quest","tracked":false,"canTrack":true,"canLocate":true},
  long: {"name":"The Hollow Crown of Caer Velin","label":"Main quest","summary":"The crown is said to lie within the ruins of Caer Velin, where the old kings were buried with their oaths. The abbot’s map marks a way in beneath the eastern tower, but the gravekeeper warns that the oath-bound do not sleep, and that whoever wears the crown inherits every promise its kings broke.","objectives":[{"id":"o1","text":"Reach the highland ruins","done":true,"optional":false,"step":0},{"id":"o2","text":"Enter the ruins of Caer Velin","done":false,"optional":false,"step":0},{"id":"o3","text":"Find the abbot’s map","done":true,"optional":false,"step":1},{"id":"o4","text":"Light the three rune pillars (1 / 3)","done":false,"optional":false,"step":1},{"id":"o5","text":"Speak with the gravekeeper before nightfall at the chapel gate","done":false,"optional":true,"step":1},{"id":"o6","text":"Descend into the oath-crypt","done":false,"optional":false,"step":0},{"id":"o7","text":"Find the crown","done":false,"optional":false,"step":0}],"rewards":[{"id":"g","icon":"coin","label":"1,200 Gold","detail":""},{"id":"x","icon":"star","label":"3,400 Experience","detail":""},{"id":"c","icon":"crown","label":"Crown of Caer Velin","detail":"Unique relic"}],"scenery":0,"tracked":true,"canTrack":true,"canLocate":true},
};
