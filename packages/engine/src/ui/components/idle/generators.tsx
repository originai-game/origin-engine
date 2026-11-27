import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, formatCompact, formatNumber, t} from "@origingame/engine/ui";
import "./idle.en.strings.ts";

export interface IdleGeneratorsProps {
  /** The list's heading; empty takes the group's (Generators). */
  title: string;
  /** The producers in the game's order, keyed rows of { id, icon, name, owned, rate, cycle, cycleText, level, boost, auto, locked }: owned how many the player has (a number, compacted; 0 hides it); rate what it yields as the game words it (+1.2K/s; on a locked row what unlocks it); cycle the running cycle 0 to 1 (negative hides the bar); cycleText the time left as the game words it (empty shows the percent); level its level (0 hides it); boost a multiplier as the game words it (×2; empty hides it); auto it runs by itself (named Auto); locked not yet available (named Locked, disabled). */
  generators?: ReadonlyArray<{ id: string; icon: string; name: string; owned: number; rate: string; cycle: number; cycleText: string; level: number; boost: string; auto: boolean; locked: boolean }>;
  /** The generator the player last chose, selected; empty is none. */
  selected: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
}

/**
 * The generator list of an idle game: one cut row per producer with its art
 * in a socket and its level on a hexagon at the socket's foot, its name and
 * how many are owned, its yield in green, its running cycle as a leaning
 * bar with the time left, and its Auto and boost tags. The game runs the
 * cycles and owns every number; the list reports choose (a row's key) and
 * the chosen row is washed in the accent. A locked row is disabled and
 * greyed, and says Locked with what unlocks it. It fills its height and
 * scrolls, its foot fading while there is more.
 */
export function IdleGenerators(props: IdleGeneratorsProps) {
  return (
    <View class={`grow shrink basis-0 w-full p-4 gap-3 disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title !== '' ? props.title : t("idle.generators.name")}>
      <View class="absolute left-4 right-3/10 top-0 h-px bg-linear-to-r from-sheen to-transparent to-70% pointer-events-none" />
      <View class={`min-h-12 border-b border-border-muted w-full items-start py-1 px-4 gap-0`}>
        <Text class={`whitespace-nowrap text-ellipsis text-xs font-semibold tracking-kicker uppercase text-fg-muted overflow-hidden`}>
          {t("idle.generators.kicker")}
        </Text>
        <Text class={`font-display text-base font-bold leading-tight text-fg max-w-full whitespace-nowrap text-ellipsis m-0 overflow-hidden`}>
          {props.title !== '' ? props.title : t("idle.generators.name")}
        </Text>
      </View>
      <Show when={(props.generators ?? []).length} fallback={
          <Text class={`grow shrink basis-0 text-center p-4 m-0`} role="status">
            {t("idle.generators.empty")}
          </Text>
      }>
        <View class="grow shrink basis-0 gap-0">
          <Scroll class="grow shrink basis-0 gap-0 ">
            <View class="pt-0.5 pb-4 px-0.5 gap-1" role="listbox" label={props.title !== '' ? props.title : t("idle.generators.name")}>
              <For each={props.generators}>
                {(g) => (
                <View class={`flex-row items-center justify-center gap-1 text-fg transition duration-control w-full min-h-11 py-1 px-2 data-locked:text-fg data-down:opacity-dim compact:p-1 ${g().id === props.selected ? packClass("Row", "frame", "selected") : packClass("Row", "frame")}`} role="option" selected={g().id === props.selected} label={g().locked ? t("idle.generators.lockedName", { name: g().name, what: g().rate }) : g().level > 0 ? t("idle.generators.levelName", { name: g().name, level: formatNumber(g().level) }) : g().name} disabled={g().locked} data-auto={g().auto} data-boosted={!!g().boost} onPress={(e) => props.onChoose?.(e, g().id)} data-selected={g().id === props.selected} data-locked={g().locked}>
                  <View class="group/face flex-row items-center w-full gap-3 compact:gap-1">
                    <View class="size-12 group-data-locked/face:opacity-dim compact:size-8">
                      <View class={`group/socket grow shrink basis-0 justify-center items-center absolute inset-0 gap-2 ${packClass("Slot", "frame")}`}>
                        <Icon class="size-19/25" name={g().icon} />
                      </View>
                      <Show when={!g().locked && g().level > 0}>
                        <View class={`items-center max-w-105 absolute -left-0.5 -bottom-1 w-4.75 h-4 text-center text-xs font-bold text-hud-accent-fg bg-linear-to-b from-hud-accent-hi to-hud-accent p-0 mx-auto gap-2`}>
                          <Text class="whitespace-nowrap tabular-nums">{formatNumber(g().level)}</Text>
                        </View>
                      </Show>
                    </View>
                    <View class="grow shrink basis-0 gap-0.5">
                      <View class="group/line flex-row items-center w-full gap-1">
                        <Text class={`text-fg shrink text-start whitespace-nowrap text-ellipsis text-lg font-semibold overflow-hidden group-data-locked/line:opacity-dim`}>
                          {g().name}
                        </Text>
                        <Show when={!g().locked && g().owned > 0}>
                          <Text class={`whitespace-nowrap text-xs font-bold tabular-nums text-fg-muted`}>
                            {t("idle.generators.owned", { owned: formatCompact(g().owned) })}
                          </Text>
                        </Show>
                        <Text class="grow shrink basis-0" />
                        <Show when={!g().locked}>
                          <Text class={`max-w-1/2 whitespace-nowrap text-ellipsis text-lg font-bold tabular-nums text-success overflow-hidden`}>
                            {g().rate}
                          </Text>
                        </Show>
                      </View>
                      <View class="flex-row items-center w-full gap-1">
                        <Show when={!g().locked && g().cycle >= 0} fallback={
                            <Text class="grow shrink basis-0" />
                        }>
                          <View class={`grow shrink basis-0 min-w-12 bg-track shadow-sunk h-2 rounded-capsule overflow-hidden`} role="progressbar" label={t("idle.generators.cycle")} valueMin={0} valueMax={100} valueNow={g().cycle * 100} valueText={g().cycleText ? g().cycleText : `${formatNumber((g().cycle) * 100, { decimals: 0 })}%`}>
                            <View class={`bg-hud-accent h-full`} style={{ width: `${(100 * (g().cycle > 1 ? 1 : g().cycle))}%` }} />
                          </View>
                          <Text class={`whitespace-nowrap text-xs font-bold tabular-nums text-fg-muted`}>
                            {g().cycleText ? g().cycleText : `${formatNumber((g().cycle) * 100, { decimals: 0 })}%`}
                          </Text>
                        </Show>
                        <Show when={!g().locked && g().boost}>
                          <Text class={`text-shadow-tag whitespace-nowrap text-xs font-semibold tracking-label uppercase text-fg-muted py-0 px-1 data-tag-boost:bg-gold data-tag-boost:bg-none data-tag-boost:nine-none data-tag-boost:text-gold-fg ${packClass("Chip", "frame")}`} data-tag-boost={true}>
                            {g().boost}
                          </Text>
                        </Show>
                        <Show when={!g().locked && g().auto}>
                          <Text class={`text-shadow-tag whitespace-nowrap text-xs font-semibold tracking-label uppercase text-fg-muted py-0 px-1 data-tag-auto:bg-success data-tag-auto:bg-none data-tag-auto:nine-none data-tag-auto:text-success-fg ${packClass("Chip", "frame")}`} data-tag-auto={true}>
                            {t("idle.generators.auto")}
                          </Text>
                        </Show>
                        <Show when={g().locked}>
                          <View class={`flex-row items-center text-xs font-semibold tracking-label uppercase text-fg-muted py-0 px-1 gap-0.5 data-tag-locked:items-center data-tag-locked:min-w-0 data-tag-locked:max-w-full ${packClass("Chip", "frame")}`} data-tag-locked={true}>
                            <Text class="whitespace-nowrap text-shadow-tag shrink">
                              {t("idle.generators.locked")}
                            </Text>
                            <Text class="bg-current whitespace-nowrap text-shadow-tag size-0.75" />
                            <Text class="whitespace-nowrap text-ellipsis text-shadow-tag overflow-hidden shrink">
                              {g().rate}
                            </Text>
                          </View>
                        </Show>
                      </View>
                    </View>
                  </View>
                </View>
                )}
              </For>
            </View>
          </Scroll>
          <View class={`absolute bottom-0 h-4 pointer-events-none bg-linear-to-b from-transparent to-surface inset-x-0`} />
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<IdleGeneratorsProps>> = {
  default: {"title":"","generators":[{"id":"farm","icon":"wheat","name":"Farm","owned":25,"rate":"+48/s","cycle":0.62,"cycleText":"0.4s","level":3,"boost":"","auto":true,"locked":false},{"id":"mine","icon":"gem","name":"Gem Mine","owned":4,"rate":"+1.2K/s","cycle":0.18,"cycleText":"8.2s","level":1,"boost":"×2","auto":false,"locked":false},{"id":"forge","icon":"fire","name":"Forge","owned":0,"rate":"10K gold","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true}],"selected":"mine"},
  empty: {"title":"","generators":[],"selected":""},
  disabled: {"title":"","generators":[{"id":"forge","icon":"fire","name":"Forge","owned":0,"rate":"Reach 1M gold","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true},{"id":"tower","icon":"crown","name":"Mage Tower","owned":0,"rate":"Reach 1B gold","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true}],"selected":""},
  selected: {"title":"","generators":[{"id":"farm","icon":"wheat","name":"Farm","owned":25,"rate":"+48/s","cycle":1,"cycleText":"","level":3,"boost":"","auto":false,"locked":false},{"id":"mine","icon":"gem","name":"Gem Mine","owned":4,"rate":"+1.2K/s","cycle":0,"cycleText":"","level":1,"boost":"","auto":false,"locked":false}],"selected":"farm"},
  overflow: {"title":"Generators of the Sprawling Clockwork Empire","generators":[{"id":"g1","icon":"leaf","name":"Overgrown Terraced Hydroponic Garden Complex","owned":1250000000000000,"rate":"+4.56 quintillion per second","cycle":0.5,"cycleText":"1h 22m 10s","level":999,"boost":"×1,024","auto":true,"locked":false},{"id":"g2","icon":"gem","name":"Mine","owned":12,"rate":"+1.2K/s","cycle":0.1,"cycleText":"3s","level":2,"boost":"","auto":false,"locked":false},{"id":"g3","icon":"fire","name":"Forge","owned":3,"rate":"+30K/s","cycle":0.9,"cycleText":"0.1s","level":1,"boost":"×3","auto":true,"locked":false},{"id":"g4","icon":"bolt","name":"Dynamo","owned":1,"rate":"+400K/s","cycle":0.33,"cycleText":"20s","level":1,"boost":"","auto":false,"locked":false},{"id":"g5","icon":"crown","name":"Mage Tower","owned":0,"rate":"+9M/s","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true},{"id":"g6","icon":"star","name":"Star Engine","owned":0,"rate":"+1B/s","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true},{"id":"g7","icon":"skull","name":"Necropolis","owned":0,"rate":"+80B/s","cycle":-1,"cycleText":"","level":0,"boost":"","auto":false,"locked":true}],"selected":"g1"},
};
