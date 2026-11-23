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
            <Text class={`text-base leading-4.5 text-fg text-center bg-tile py-2 px-3 m-0`}>
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
                  <Text class={`grow shrink basis-0 font-display text-3xl font-bold leading-tight tracking-tight text-start whitespace-nowrap m-0 overflow-hidden compact:text-xl`}>
                    {c().label}
                  </Text>
                  <View class={`flex-row items-center text-caption bg-tile inset-shadow-lit text-fg transition duration-317 py-0.5 px-2 gap-0.5 rounded-capsule`} label={t("puzzle.path.chapterStarsName", { earned: c().earned, total: c().total })}>
                    <Icon class={`text-gold size-3.5`} name="star" />
                    <Text class="text-base font-bold tabular-nums whitespace-nowrap shrink">
                      {t("puzzle.path.chapterStars", { earned: c().earned, total: c().total })}
                    </Text>
                  </View>
                </View>
                <View class={`w-full h-1.25 bg-track gap-2`}>
                  <View class="absolute top-1 left-1 h-full bg-linear-to-r from-gold to-gold-hi shadow-glow shadow-gold" style={{ width: `${(100 * (c().total > 0 ? (c().earned >= c().total ? 1 : c().earned / c().total) : 0))}%` }} />
                </View>
              </View>
              <View class="gap-0" role="listbox" label={c().label}>
                <For each={props.levels}>
                  {(l, i) => (
                  <Show when={l().chapter === c().id}>
                    <View class={`group/lane flex-row items-center w-full pt-1 pb-8.75 gap-0 data-state-current:pb-12.75`} data-starred={l().state === 'done'} data-state-current={(l().state) === 'current'} data-state-locked={(l().state) === 'locked'} data-state-open={(l().state) === 'open'} data-selected={l().id === props.selected}>
                      {/* The roads run across the track the discs' centres span: the lane less a stop, inset by half a stop. */}
                      <View class="absolute inset-y-0 left-18 right-18">
                        <Show when={i() > 0 && (props.levels ?? [])[i() - 1]['chapter'] === l().chapter}>
                          <View class="absolute inset-y-0" style={{ left: `${100 * Math.min((props.levels ?? [])[i() - 1]['x'], l().x)}%`, width: `${100 * Math.abs((props.levels ?? [])[i() - 1]['x'] - l().x)}%` }}>
                            <View class={`absolute -left-1.5 -right-1.5 compact:-left-1.25 compact:-right-1.25 bg-fg shadow-plate ring-3 ring-shade-deep -top-1.5 h-3 rounded-capsule data-lit:bg-linear-to-r data-lit:from-gold-hi data-lit:to-gold data-lit:bg-transparent compact:-top-1.25 compact:h-2.5`} data-lit={l().state === 'done' || l().state === 'current'} />
                          </View>
                          <View class="absolute inset-y-0 w-0" style={{ left: `${100 * l().x}%` }}>
                            <View class={`absolute -left-1.5 compact:-left-1.25 bg-fg shadow-plate ring-3 ring-shade-deep -top-1.5 h-14.75 w-3 rounded-capsule data-lit:bg-linear-to-r data-lit:from-gold-hi data-lit:to-gold data-lit:bg-transparent compact:-top-1.25 compact:h-12 compact:w-2.5`} data-lit={l().state === 'done' || l().state === 'current'} />
                          </View>
                        </Show>
                        <Show when={i() + 1 < (props.levels ?? []).length && (props.levels ?? [])[i() + 1]['chapter'] === l().chapter}>
                          <View class="absolute inset-y-0 w-0" style={{ left: `${100 * l().x}%` }}>
                            <View class={`absolute -left-1.5 compact:-left-1.25 bg-fg shadow-plate ring-3 ring-shade-deep top-13 w-3 rounded-capsule data-lit:bg-linear-to-r data-lit:from-gold-hi data-lit:to-gold data-lit:bg-transparent group-data-state-current/lane:top-auto group-data-state-current/lane:h-3 group-data-state-current/lane:bottom-0 compact:w-2.5`} data-lit={(props.levels ?? [])[i() + 1]['state'] === 'done' || (props.levels ?? [])[i() + 1]['state'] === 'current'} />
                          </View>
                        </Show>
                      </View>
                      <View class="shrink basis-0" style={{ flexGrow: String(String(l().x)) }} />
                      <View class="items-center max-w-105 w-36 h-24 mx-auto gap-2 compact:h-16.75">
                        <View class={`flex-row items-center justify-center gap-1 group/node min-h-11 min-w-11 text-fg shadow-float inset-ring-2 inset-ring-rim-strong transition duration-317 p-0 size-16 rounded-capsule compact:size-12.75 data-state-current:text-hud-accent-fg data-state-current:bg-transparent data-state-current:shadow-glow-lg data-state-current:ring-9 data-state-current:ring-entry-grain data-state-current:inset-ring-3 data-state-current:inset-ring-knob data-state-current:size-24 data-state-locked:text-fg-subtle data-state-locked:bg-transparent data-state-locked:shadow-plate data-state-locked:inset-ring-2 data-state-locked:inset-ring-rim data-state-open:bg-transparent data-state-open:shadow-float data-state-open:inset-ring-3 data-state-open:inset-ring-hud-accent hover:shadow-float hover:inset-ring-2 hover:inset-ring-rim-strong data-selected:inset-shadow-lit data-selected:ring-5 data-selected:ring-knob data-selected:inset-ring-2 data-selected:inset-ring-rim-strong focus-visible:ring-5 focus-visible:ring-rim-strong focus-visible:inset-ring-2 focus-visible:inset-ring-rim-strong compact:data-state-current:size-16.75`} role="option" selected={l().id === props.selected} disabled={l().state === 'locked'} label={l().state === 'locked' ? t("puzzle.path.levelLocked", { number: l().number }) : l().state === 'done' ? t("puzzle.path.levelDone", { number: l().number, stars: l().stars, max: l().maxStars }) : l().state === 'current' ? t("puzzle.path.levelCurrent", { number: l().number }) : t("puzzle.path.levelOpen", { number: l().number })} onPress={(e) => props.onChoose?.(e, l().id)} onAction={{ details: (e) => props.onDetails?.(e, l().id) }} data-state-current={(l().state) === 'current'} data-state-locked={(l().state) === 'locked'} data-state-open={(l().state) === 'open'} data-selected={l().id === props.selected}>
                          <View class="items-center max-w-105 mx-auto gap-2 size-full">
                            <Show when={l().state === 'locked'} fallback={
                                <Text class={`text-4xl leading-flat font-bold tabular-nums whitespace-nowrap text-shadow-xs text-shadow-bg group-data-state-current/node:text-display-lg group-data-state-current/node:text-shadow-none`} data-length-long={(l().number >= 10000 ? 'longer' : l().number >= 1000 ? 'long' : null) === 'long'} data-length-longer={(l().number >= 10000 ? 'longer' : l().number >= 1000 ? 'long' : null) === 'longer'}>
                                  {formatNumber(l().number)}
                                </Text>
                            }>
                              <Icon class="size-19/50" name="lock" />
                            </Show>
                          </View>
                        </View>
                        <Show when={l().state === 'done'}>
                          <View class={`flex-row items-end absolute left-1/2 -translate-x-1/2 bg-linear-to-b from-plate-hi to-plate-lo shadow-plate py-0.5 px-1 gap-0 rounded-capsule`} data-state-current={(l().state) === 'current'} data-state-locked={(l().state) === 'locked'} data-state-open={(l().state) === 'open'}>
                            <Icon class="text-fg-muted size-4.25 compact:size-3.25 data-lit:text-gold shrink" name="star" data-lit={l().stars >= 1} />
                            <Show when={l().maxStars >= 2}>
                              <Icon class="text-fg-muted size-5 compact:size-4 data-lit:text-gold shrink" name="star" data-lit={l().stars >= 2} />
                            </Show>
                            <Show when={l().maxStars >= 3}>
                              <Icon class="text-fg-muted size-4.25 compact:size-3.25 data-lit:text-gold shrink" name="star" data-lit={l().stars >= 3} />
                            </Show>
                          </View>
                        </Show>
                        <Show when={l().state === 'current'} fallback={
                            <Show when={l().id === props.selected} fallback={
                                <Show when={l().state === 'open'}>
                                  <Text class={`font-bold text-xs leading-loose text-fg absolute top-17 left-1/2 -translate-x-1/2 whitespace-nowrap tracking-label uppercase bg-tile py-0 px-1 compact:top-13.75`}>
                                    {t("puzzle.path.open")}
                                  </Text>
                                </Show>
                            }>
                              <Text class={`font-bold text-xs leading-loose text-fg absolute top-17 left-1/2 -translate-x-1/2 whitespace-nowrap tracking-label uppercase bg-tile py-0 px-1 compact:top-13.75`}>
                                {t("puzzle.path.selected")}
                              </Text>
                            </Show>
                        }>
                          <Text class={`absolute top-25 left-1/2 -translate-x-1/2 whitespace-nowrap font-bold tracking-label uppercase text-caption compact:top-17.75 text-hud-accent-fg bg-hud-accent-hi nine-chip-cut py-0.75 px-3 ${packClass("Chip", "frame")}`}>
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
