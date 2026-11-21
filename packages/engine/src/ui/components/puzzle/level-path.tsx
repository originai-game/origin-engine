import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, formatNumber, t} from "@origingame/engine/ui";
import "./puzzle.en.ogt";

export interface PuzzleLevelPathProps {
  /** The chapters in path order, keyed rows of { id, label, earned, total }: earned and total the chapter's stars as the game counts them. */
  chapters?: ReadonlyArray<{ id: string; label: string; earned: number; total: number }>;
  /** The levels in path order, keyed rows of { id, chapter, number, state, stars, maxStars, x }: chapter a chapter's id; state locked, open, current or done; stars earned (0 to 3) of maxStars (1 to 3); x the node's place across the path, 0 (start) to 1 (end), as the game lays the path out. */
  levels?: ReadonlyArray<{ id: string; chapter: string; number: number; state: string; stars: number; maxStars: number; x: number }>;
  /** The level the player has picked (its details shown elsewhere); empty is none. */
  selected: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The details event, with the row's key where it comes from a row. */
  onDetails?: (e: UiEvent, key?: string) => void;
}

/**
 * A scrolling level path over the game's own map art, which the component
 * leaves at full brightness: chapters in order, each a glass header with
 * its name, its stars and a gold line of them, then its levels as discs on
 * a road, each at the place across the path the game gives (x). The road
 * is lit gold up to the level to play and pale past it. A disc shows its
 * number, a lock when it is locked, its stars on a dark capsule at its
 * foot once done, and the Play plate under the current one, the road leaving
 * only below it after a gap; the chosen
 * one is ringed and says Selected, an open one says Open. Reports choose
 * (a level's key; a locked level is disabled and sends nothing) and, with
 * X or pad Y on the focused disc, details. The game decides what is open
 * and counts the stars. The path opens scrolled to the level to play,
 * each chapter's header held at the top while its levels scroll under it.
 */
export function PuzzleLevelPath(props: PuzzleLevelPathProps) {
  return (
    <View class={`grow shrink basis-0 border-rim bg-linear-to-b from-plate-hi to-plate-lo text-fg w-full p-2 gap-2 rounded-block border`} label={t("puzzle.path.name")}>
      <Show when={(props.levels ?? []).length && (props.chapters ?? []).length} fallback={
          <View class={`grow shrink basis-0 justify-center items-center p-6 gap-2`} role="status">
            <Text class={`text-[1.143rem] leading-4.5 text-fg text-center bg-tile py-2 px-3 m-0`}>
              {t("puzzle.path.empty")}
            </Text>
          </View>
      }>
        <Scroll class="grow shrink basis-0 pt-2 pb-6 px-2 gap-2 ">
          <For each={props.chapters}>
            {(c) => (
            <View class="mb-4 gap-0" label={c().label}>
              <View class={`sticky top-0 z-1 w-full pt-2 pb-3 bg-plate-hi border border-rim nine-plate-cut px-3 gap-1 text-fg ${packClass("Plate", "frame")}`}>
                <View class="flex-row items-center w-full gap-1">
                  <Text class={`grow shrink basis-0 font-display text-[1.857rem] font-bold leading-[1.15] tracking-[-0.01em] text-start whitespace-nowrap m-0 overflow-hidden compact:text-[1.429rem]`}>
                    {c().label}
                  </Text>
                  <View class={`flex-row items-center text-caption bg-tile shadow-[inset_0px_1px_0px_#ffffff38] text-fg transition duration-317 py-[0.143rem] px-2 gap-0.5 rounded-capsule`} label={t("puzzle.path.chapterStarsName", { earned: c().earned, total: c().total })}>
                    <Icon class={`text-gold size-[14.3px]`} name="star" />
                    <Text class="text-[1.143rem] font-bold tabular-nums whitespace-nowrap shrink">
                      {t("puzzle.path.chapterStars", { earned: c().earned, total: c().total })}
                    </Text>
                  </View>
                </View>
                <View class={`w-full h-[0.357rem] bg-lit gap-2`}>
                  <View class="absolute top-1 left-1 h-full bg-linear-to-r from-gold to-gold-hi shadow-[0px_0px_11.424px_#ffcf5ab2]" style={{ width: `${(100 * (c().total > 0 ? (c().earned >= c().total ? 1 : c().earned / c().total) : 0))}%` }} />
                </View>
              </View>
              <View class="gap-0" role="listbox" label={c().label}>
                <For each={props.levels}>
                  {(l, i) => (
                  <Show when={l().chapter === c().id}>
                    <View class={`group/lane flex-row items-center w-full pt-1 pb-[34.743px] gap-0 data-state-current:pb-[50.743px]`} data-starred={l().state === 'done'} data-state-current={(l().state) === 'current'} data-state-locked={(l().state) === 'locked'} data-state-open={(l().state) === 'open'} data-selected={l().id === props.selected}>
                      <Show when={i() > 0 && (props.levels ?? [])[i() - 1]['chapter'] === l().chapter}>
                        <View class={`absolute bg-fg shadow-[0px_0px_0px_3.424px_#04060c61,0px_3.424px_6.864px_#00000059] top-[-0.429rem] h-[0.857rem] rounded-capsule data-lit:bg-linear-to-r data-lit:from-gold-hi data-lit:to-gold data-lit:bg-transparent compact:top-[-0.343rem] compact:h-[0.686rem]`} data-lit={l().state === 'done' || l().state === 'current'} />
                        <View class={`absolute bg-fg shadow-[0px_0px_0px_3.424px_#04060c61,0px_3.424px_6.864px_#00000059] top-[-0.429rem] h-[58.857px] w-[0.857rem] rounded-capsule data-lit:bg-linear-to-r data-lit:from-gold-hi data-lit:to-gold data-lit:bg-transparent compact:top-[-0.343rem] compact:h-[47.886px] compact:w-[0.686rem]`} data-lit={l().state === 'done' || l().state === 'current'} />
                      </Show>
                      <Show when={i() + 1 < (props.levels ?? []).length && (props.levels ?? [])[i() + 1]['chapter'] === l().chapter}>
                        <View class={`absolute bg-fg shadow-[0px_0px_0px_3.424px_#04060c61,0px_3.424px_6.864px_#00000059] top-13 w-[0.857rem] rounded-capsule data-lit:bg-linear-to-r data-lit:from-gold-hi data-lit:to-gold data-lit:bg-transparent group-data-state-current/lane:top-auto group-data-state-current/lane:h-3 group-data-state-current/lane:bottom-0 compact:w-[0.686rem]`} data-lit={(props.levels ?? [])[i() + 1]['state'] === 'done' || (props.levels ?? [])[i() + 1]['state'] === 'current'} />
                      </Show>
                      <View class="shrink basis-0" style={{ flexGrow: String(String(l().x)) }} />
                      <View class="items-center max-w-105 w-36 h-24 mx-auto gap-2 compact:h-[4.8rem]">
                        <View class={`flex-row items-center justify-center gap-1 group/node min-h-11 min-w-11 text-fg shadow-[inset_0px_0px_0px_2.288px_#ffffff59,inset_0px_3.424px_0px_#ffffff80,0px_9.136px_18.288px_#00000080] transition duration-317 p-0 size-[4.571rem] rounded-capsule compact:size-[3.657rem] data-state-current:text-hud-accent-fg data-state-current:bg-transparent data-state-current:shadow-[inset_0px_0px_0px_3.424px_#ffffff,0px_0px_0px_9.136px_#38e8ff4d,0px_0px_0px_18.288px_#38e8ff24,0px_0px_41.136px_#38e8ffcc] data-state-current:size-24 data-state-locked:text-fg-subtle data-state-locked:bg-transparent data-state-locked:shadow-[inset_0px_0px_0px_2.288px_#ffffff29,0px_6.864px_13.712px_#00000073] data-state-open:bg-transparent data-state-open:shadow-[inset_0px_0px_0px_3.424px_#38e8ff,inset_0px_4.576px_0px_#ffffff80,0px_9.136px_18.288px_#00000080] hover:shadow-[inset_0px_0px_0px_2.288px_#ffffffb2,inset_0px_3.424px_0px_#ffffff99,0px_11.424px_22.864px_#0000008c] data-selected:shadow-[inset_0px_0px_0px_2.288px_#ffffff59,inset_0px_3.424px_0px_#ffffff80,0px_0px_0px_4.576px_#ffffff,0px_0px_0px_9.136px_#38e8ff] focus-visible:shadow-[inset_0px_0px_0px_2.288px_#ffffff59,0px_0px_0px_4.576px_#fffffff2] compact:data-state-current:size-[4.8rem]`} role="option" selected={l().id === props.selected} disabled={l().state === 'locked'} label={l().state === 'locked' ? t("puzzle.path.levelLocked", { number: l().number }) : l().state === 'done' ? t("puzzle.path.levelDone", { number: l().number, stars: l().stars, max: l().maxStars }) : l().state === 'current' ? t("puzzle.path.levelCurrent", { number: l().number }) : t("puzzle.path.levelOpen", { number: l().number })} onPress={(e) => props.onChoose?.(e, l().id)} onAction={{ details: (e) => props.onDetails?.(e, l().id) }} data-state-current={(l().state) === 'current'} data-state-locked={(l().state) === 'locked'} data-state-open={(l().state) === 'open'} data-selected={l().id === props.selected}>
                          <View class="items-center max-w-105 mx-auto gap-2 size-full">
                            <Show when={l().state === 'locked'} fallback={
                                <Text class={`text-[2.143rem] leading-flat font-bold tabular-nums whitespace-nowrap text-shadow-xs text-shadow-bg/55 group-data-state-current/node:text-[2.857rem] group-data-state-current/node:text-shadow-none`} data-length-long={(l().number >= 10000 ? 'longer' : l().number >= 1000 ? 'long' : null) === 'long'} data-length-longer={(l().number >= 10000 ? 'longer' : l().number >= 1000 ? 'long' : null) === 'longer'}>
                                  {formatNumber(l().number)}
                                </Text>
                            }>
                              <Icon class="size-[38%]" name="lock" />
                            </Show>
                          </View>
                        </View>
                        <Show when={l().state === 'done'}>
                          <View class={`flex-row items-end absolute left-1/2 translate-x-[-50%] bg-linear-to-b from-plate-hi to-plate-lo shadow-[inset_0px_1px_0px_#ffffff59,0px_3.424px_9.136px_#00000080] py-[0.143rem] px-1 gap-0 rounded-capsule`} data-state-current={(l().state) === 'current'} data-state-locked={(l().state) === 'locked'} data-state-open={(l().state) === 'open'}>
                            <Icon class="text-fg-muted size-[1.189rem] compact:size-[0.951rem] data-lit:text-gold shrink" name="star" data-lit={l().stars >= 1} />
                            <Show when={l().maxStars >= 2}>
                              <Icon class="text-fg-muted size-[1.463rem] compact:size-[1.17rem] data-lit:text-gold shrink" name="star" data-lit={l().stars >= 2} />
                            </Show>
                            <Show when={l().maxStars >= 3}>
                              <Icon class="text-fg-muted size-[1.189rem] compact:size-[0.951rem] data-lit:text-gold shrink" name="star" data-lit={l().stars >= 3} />
                            </Show>
                          </View>
                        </Show>
                        <Show when={l().state === 'current'} fallback={
                            <Show when={l().id === props.selected} fallback={
                                <Show when={l().state === 'open'}>
                                  <Text class={`font-bold text-[0.857rem] leading-[1.6] text-fg absolute left-1/2 translate-x-[-50%] whitespace-nowrap tracking-[0.14em] uppercase bg-tile py-0 px-1`}>
                                    {t("puzzle.path.open")}
                                  </Text>
                                </Show>
                            }>
                              <Text class={`font-bold text-[0.857rem] leading-[1.6] text-fg absolute left-1/2 translate-x-[-50%] whitespace-nowrap tracking-[0.14em] uppercase bg-tile py-0 px-1`}>
                                {t("puzzle.path.selected")}
                              </Text>
                            </Show>
                        }>
                          <Text class={`absolute left-1/2 translate-x-[-50%] whitespace-nowrap font-bold tracking-[0.14em] uppercase text-[0.929rem] text-hud-accent-fg bg-hud-accent-hi nine-chip-cut py-[0.214rem] px-3 ${packClass("Chip", "frame")}`}>
                            {t("puzzle.path.play")}
                          </Text>
                        </Show>
                      </View>
                      <View class="shrink basis-0" style={{ flexGrow: String(`${(1 + (-(l().x)))}`) }} />
                    </View>
                  </Show>
                  )}
                </For>
              </View>
            </View>
            )}
          </For>
        </Scroll>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<PuzzleLevelPathProps>> = {
  default: {"selected":"","chapters":[{"id":"c1","label":"Candy Meadow","earned":7,"total":12}],"levels":[{"id":"l1","chapter":"c1","number":1,"state":"done","stars":3,"maxStars":3,"x":0.5},{"id":"l2","chapter":"c1","number":2,"state":"done","stars":2,"maxStars":3,"x":0.75},{"id":"l3","chapter":"c1","number":3,"state":"done","stars":2,"maxStars":3,"x":0.5},{"id":"l4","chapter":"c1","number":4,"state":"current","stars":0,"maxStars":3,"x":0.25},{"id":"l5","chapter":"c1","number":5,"state":"locked","stars":0,"maxStars":3,"x":0.5}]},
  empty: {"selected":"","chapters":[],"levels":[]},
  selected: {"selected":"l2","chapters":[{"id":"c1","label":"Candy Meadow","earned":7,"total":12}],"levels":[{"id":"l1","chapter":"c1","number":1,"state":"done","stars":3,"maxStars":3,"x":0.5},{"id":"l2","chapter":"c1","number":2,"state":"done","stars":2,"maxStars":3,"x":0.75},{"id":"l3","chapter":"c1","number":3,"state":"open","stars":0,"maxStars":3,"x":0.5},{"id":"l4","chapter":"c1","number":4,"state":"current","stars":0,"maxStars":3,"x":0.25}]},
  locked: {"selected":"","chapters":[{"id":"c9","label":"Frosted Peaks","earned":0,"total":9}],"levels":[{"id":"l41","chapter":"c9","number":41,"state":"locked","stars":0,"maxStars":3,"x":0.5},{"id":"l42","chapter":"c9","number":42,"state":"locked","stars":0,"maxStars":3,"x":0.25},{"id":"l43","chapter":"c9","number":43,"state":"locked","stars":0,"maxStars":3,"x":0.5}]},
  overflow: {"selected":"l12","chapters":[{"id":"c1","label":"Candy Meadow","earned":12,"total":12},{"id":"c2","label":"The Very Long Caramel Causeway Beyond the Whispering Lemonade Falls","earned":4,"total":18}],"levels":[{"id":"l9","chapter":"c1","number":9,"state":"done","stars":3,"maxStars":3,"x":0.25},{"id":"l10","chapter":"c1","number":10,"state":"done","stars":3,"maxStars":3,"x":0.5},{"id":"l11","chapter":"c1","number":11,"state":"done","stars":3,"maxStars":3,"x":0.75},{"id":"l12","chapter":"c1","number":12,"state":"done","stars":3,"maxStars":3,"x":0.5},{"id":"l13","chapter":"c2","number":13,"state":"done","stars":1,"maxStars":3,"x":0.25},{"id":"l14","chapter":"c2","number":14,"state":"done","stars":3,"maxStars":3,"x":0.5},{"id":"l15","chapter":"c2","number":15,"state":"open","stars":0,"maxStars":3,"x":0.75},{"id":"l16","chapter":"c2","number":1016,"state":"current","stars":0,"maxStars":3,"x":0.5},{"id":"l17","chapter":"c2","number":1017,"state":"locked","stars":0,"maxStars":3,"x":0.25},{"id":"l18","chapter":"c2","number":1018,"state":"locked","stars":0,"maxStars":3,"x":0.5}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"details":"Play"} as const;
