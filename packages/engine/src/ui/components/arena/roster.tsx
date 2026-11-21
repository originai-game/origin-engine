import { For, Icon, Image, Scroll, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./arena.en.ogt";

export interface RosterCharacter {
  /** The character's key, reported back with choose. */
  id: string;
  /** What a reader calls them. */
  name: string;
  /** The character's portrait texture (og.load.texture, 0 shows the icon instead). */
  portrait?: Texture | 0;
  /** An icon name, shown while no portrait is set. */
  icon?: string;
  /** A key for the pack (tank, damage, support…). */
  role?: string;
  /** The role's word. */
  roleLabel?: string;
  /** The player cannot pick it yet. */
  locked?: boolean;
  /** Who already holds it (empty: free). */
  takenBy?: string;
}

export interface RosterPlayer {
  /** The player's key. */
  id: string;
  /** 1 to 4, the mark P1…P4. */
  slot?: number;
  /** The character id the player is on (empty: none). */
  at?: string;
  /** The player has locked in (a check beside the mark). */
  ready?: boolean;
}

export interface ArenaRosterProps {
  /** The screen's heading; empty takes the kit's (Choose your champion). */
  title?: string;
  /** The roster in the game's order. */
  characters?: ReadonlyArray<RosterCharacter>;
  /** The local player's character (selected, named under the grid); empty is none. */
  selected?: string;
  /** Who is choosing. */
  players?: ReadonlyArray<RosterPlayer>;
  /** Its place is narrow (under about 380 px): the tiles, padding and chosen name tighten. The place decides, not the screen. */
  compact?: boolean;
  /** The local player chooses a character. */
  onChoose?: (e: UiEvent, id: string) => void;
}

/**
 * Character selection: a heading, a grid of character cells (portrait or
 * icon, name, the role in words) and, on each cell, the marks of the
 * players standing on it (P1…P4, a check once ready). A free cell reports
 * choose with its key; a locked or taken one is disabled and names why
 * (Locked, Taken by …). The local player's character is selected and
 * named under the grid. The game decides what can be picked; the roster
 * only reports the choice. Approximations: the portraits' ratio edges are
 * rounded, the chosen tile's accent wash is a flat tint and keeps no rim
 * (a cut takes one fill), a locked or taken portrait's greying filter
 * is an opacity, and the scroll fade at the grid's foot is dropped.
 */
const CARD = `w-full max-h-full p-4 flex-col gap-y-3 text-fg nine-panel-cut bg-plate-hi data-compact:p-2 ${packClass("Panel", "frame")}`;
const MARK = "flex-row items-center gap-x-0.5 py-0 px-1 whitespace-nowrap text-xs leading-[1.5] font-bold text-key-fg bg-linear-to-b from-white to-key-hi shadow-[0px_0.143rem_0px_#0000008c,inset_0px_-0.143rem_0px_#0000002e] rounded-[0.286rem] data-ready:bg-none data-ready:bg-success data-ready:text-boon-fg";
const TAG = `flex-none max-w-full self-center py-0 px-1 text-xs leading-[1.5] font-bold tracking-[0.14em] uppercase text-center text-fg nine-chip-cut bg-raised-pressed/88 ${packClass("Chip", "frame")}`;
const CELL = "flex-col gap-y-0.5 pb-2 nine-panel-cut bg-glass-hi/93 hover:bg-raised-hi/90 data-selected:bg-hud-accent/15 data-disabled:text-fg/66 data-disabled:bg-glass-hi/93";

export function ArenaRoster(props: ArenaRosterProps) {
  const heading = () => (props.title ?? "") != "" ? props.title : t("arena.roster.title");
  const characters = () => props.characters ?? [];
  const players = () => props.players ?? [];
  const chosen = () => characters().find((c) => c.id == props.selected);
  return (
    <View class={`group/roster ${CARD}`} label={heading()} data-compact={props.compact === true}>
      <Text class="text-[1.714rem] leading-[1.1] font-bold uppercase tracking-[0.04em] text-fg">{heading()}</Text>
      <Show when={characters().length} fallback={
        <Text class="p-4 text-center text-base text-fg/66" role="status">{t("arena.roster.empty")}</Text>
      }>
        <Scroll class="min-h-0">
          <View class="grid grid-cols-[repeat(auto-fill,minmax(132px,1fr))] gap-1 p-0.5 pb-4 group-data-compact/roster:grid-cols-[repeat(auto-fill,minmax(100px,1fr))] group-data-compact/roster:gap-1" role="listbox" label={t("arena.roster.characters")}>
            <For each={characters()} key={(c) => c.id}>
              {(c) => (
                <View
                  class={CELL}
                  role="option"
                  label={c().name}
                  description={c().locked ? t("arena.roster.locked") : ((c().takenBy ?? "") != "" ? t("arena.roster.taken", { name: c().takenBy }) : (c().roleLabel ?? ""))}
                  selected={c().id == props.selected}
                  disabled={c().locked || (c().takenBy ?? "") != ""}
                  data-locked={c().locked === true}
                  data-taken={(c().takenBy ?? "") != ""}
                  onPress={(e) => props.onChoose?.(e, c().id)}
                >
                  <View class="relative w-full h-[2.714rem] overflow-hidden group-data-compact/roster:h-[2.286rem]">
                    <Show when={(c().portrait ?? 0) != 0} fallback={
                      <View class="size-full flex-row items-center justify-center">
                        <Icon class={`size-12 text-fg data-disabled:text-fg/66 group-data-compact/roster:size-8`} name={c().icon ?? ""} data-disabled={c().locked || (c().takenBy ?? "") != ""} />
                      </View>
                    }>
                      <Image class="size-full data-disabled:opacity-40" src={c().portrait ?? 0} data-disabled={c().locked || (c().takenBy ?? "") != ""} />
                    </Show>
                    <Show when={(players() ?? []).some((p) => p.at == c().id)}>
                      <View class="absolute left-1 right-1 top-1 flex-row flex-wrap gap-x-0.5 gap-y-0.5">
                        <For each={(players() ?? []).filter((p) => p.at == c().id)} key={(p) => p.id}>
                          {(p) => (
                            <View
                              class={MARK}
                              data-ready={p().ready === true}
                              label={p().ready ? t("arena.roster.slotReady", { slot: p().slot ?? 0 }) : t("arena.roster.slotName", { slot: p().slot ?? 0 })}
                            >
                              <Text class="text-xs leading-[1.5] font-bold">{t("arena.roster.slot", { slot: p().slot ?? 0 })}</Text>
                              <Show when={p().ready}>
                                <Icon class="size-4" name="check" />
                              </Show>
                            </View>
                          )}
                        </For>
                      </View>
                    </Show>
                  </View>
                  <Text class="px-1 text-center text-reading leading-[1.15] font-bold text-inherit">{c().name}</Text>
                  <Show when={c().locked || (c().takenBy ?? "") != ""}>
                    <Text class={`${TAG} ${c().locked ? "text-fg/66" : "text-fg/66"}`}>{c().locked ? t("arena.roster.locked") : t("arena.roster.taken", { name: c().takenBy })}</Text>
                  </Show>
                </View>
              )}
            </For>
          </View>
        </Scroll>
        <Show when={(props.selected ?? "") != "" && chosen()}>
          <View class="flex-none flex-row flex-wrap items-center gap-x-1 gap-y-0.5 py-2 px-3 nine-plate-cut bg-entry-hi group-data-compact/roster:gap-y-1 group-data-compact/roster:py-1 px-2" role="status">
            <Text class={`${TAG} bg-hud-accent text-glass shadow-none`}>{t("arena.roster.chosen")}</Text>
            <Text class="min-w-0 text-3xl leading-[1.1] font-bold uppercase text-fg group-data-compact/roster:text-xl">{chosen()?.name}</Text>
            <Text class="shrink min-w-0 max-w-full text-xs leading-[1.3] font-semibold tracking-[0.14em] uppercase text-fg/78">{chosen()?.roleLabel}</Text>
          </View>
        </Show>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ArenaRosterProps>> = {
  default: {"title":"","selected":"vex","characters":[{"id":"brenna","name":"Brenna","portrait":0,"icon":"shield","role":"tank","roleLabel":"Vanguard","locked":false,"takenBy":""},{"id":"vex","name":"Vex","portrait":0,"icon":"sword","role":"damage","roleLabel":"Striker","locked":false,"takenBy":""},{"id":"sela","name":"Sela","portrait":0,"icon":"heart","role":"support","roleLabel":"Mender","locked":false,"takenBy":""},{"id":"idris","name":"Idris","portrait":0,"icon":"bolt","role":"damage","roleLabel":"Arcanist","locked":false,"takenBy":"Kai"},{"id":"maren","name":"Maren","portrait":0,"icon":"leaf","role":"support","roleLabel":"Warden","locked":false,"takenBy":""},{"id":"oarsman","name":"Oarsman","portrait":0,"icon":"skull","role":"tank","roleLabel":"Bulwark","locked":true,"takenBy":""}],"players":[{"id":"p1","slot":1,"at":"vex","ready":false},{"id":"p2","slot":2,"at":"sela","ready":true}]},
  none: {"title":"Ranked · Pick your fighter","selected":"","characters":[{"id":"brenna","name":"Brenna","portrait":0,"icon":"shield","role":"tank","roleLabel":"Vanguard","locked":false,"takenBy":""},{"id":"vex","name":"Vex","portrait":0,"icon":"sword","role":"damage","roleLabel":"Striker","locked":false,"takenBy":""},{"id":"sela","name":"Sela","portrait":0,"icon":"heart","role":"support","roleLabel":"Mender","locked":false,"takenBy":""}],"players":[]},
  full: {"title":"","selected":"brenna","characters":[{"id":"brenna","name":"Brenna","portrait":0,"icon":"shield","role":"tank","roleLabel":"Vanguard","locked":false,"takenBy":""},{"id":"vex","name":"Vex","portrait":0,"icon":"sword","role":"damage","roleLabel":"Striker","locked":false,"takenBy":""},{"id":"sela","name":"Sela","portrait":0,"icon":"heart","role":"support","roleLabel":"Mender","locked":false,"takenBy":""},{"id":"idris","name":"Idris","portrait":0,"icon":"bolt","role":"damage","roleLabel":"Arcanist","locked":false,"takenBy":""}],"players":[{"id":"p1","slot":1,"at":"brenna","ready":true},{"id":"p2","slot":2,"at":"brenna","ready":false},{"id":"p3","slot":3,"at":"sela","ready":true},{"id":"p4","slot":4,"at":"idris","ready":true}]},
  disabled: {"title":"","selected":"","characters":[{"id":"brenna","name":"Brenna","portrait":0,"icon":"shield","role":"tank","roleLabel":"Vanguard","locked":false,"takenBy":"Kai"},{"id":"vex","name":"Vex","portrait":0,"icon":"sword","role":"damage","roleLabel":"Striker","locked":true,"takenBy":""},{"id":"sela","name":"Sela","portrait":0,"icon":"heart","role":"support","roleLabel":"Mender","locked":false,"takenBy":"Ana"},{"id":"idris","name":"Idris","portrait":0,"icon":"bolt","role":"damage","roleLabel":"Arcanist","locked":true,"takenBy":""}],"players":[{"id":"p1","slot":1,"at":"","ready":false}]},
  overflow: {"title":"Championship Series · Grand Finals · Choose your champion for the deciding match","selected":"c7","characters":[{"id":"c1","name":"Brenna","portrait":0,"icon":"shield","role":"tank","roleLabel":"Vanguard","locked":false,"takenBy":""},{"id":"c2","name":"Vex","portrait":0,"icon":"sword","role":"damage","roleLabel":"Striker","locked":false,"takenBy":""},{"id":"c3","name":"Sela","portrait":0,"icon":"heart","role":"support","roleLabel":"Mender","locked":false,"takenBy":"Maximilian Vandenberghe"},{"id":"c4","name":"Idris","portrait":0,"icon":"bolt","role":"damage","roleLabel":"Arcanist","locked":false,"takenBy":""},{"id":"c5","name":"Maren","portrait":0,"icon":"leaf","role":"support","roleLabel":"Warden","locked":false,"takenBy":""},{"id":"c6","name":"Oarsman","portrait":0,"icon":"skull","role":"tank","roleLabel":"Bulwark","locked":true,"takenBy":""},{"id":"c7","name":"Ysolde the Unbroken, Warden of the Ninefold Gate","portrait":0,"icon":"crown","role":"tank","roleLabel":"Guardian of the Eastern Marches","locked":false,"takenBy":""},{"id":"c8","name":"Lurker","portrait":0,"icon":"drop","role":"damage","roleLabel":"Assassin","locked":false,"takenBy":""},{"id":"c9","name":"Ashen Pyre","portrait":0,"icon":"fire","role":"damage","roleLabel":"Pyromancer","locked":true,"takenBy":""},{"id":"c10","name":"Tamsin","portrait":0,"icon":"music","role":"support","roleLabel":"Bard","locked":false,"takenBy":""},{"id":"c11","name":"Gorrum","portrait":0,"icon":"bomb","role":"tank","roleLabel":"Sapper","locked":false,"takenBy":""},{"id":"c12","name":"Quill","portrait":0,"icon":"star","role":"support","roleLabel":"Oracle","locked":false,"takenBy":""},{"id":"c13","name":"Rook","portrait":0,"icon":"key","role":"damage","roleLabel":"Duelist","locked":false,"takenBy":""},{"id":"c14","name":"Hollow","portrait":0,"icon":"gem","role":"damage","roleLabel":"Hexer","locked":true,"takenBy":""},{"id":"c15","name":"Wren","portrait":0,"icon":"apple","role":"support","roleLabel":"Herbalist","locked":false,"takenBy":""},{"id":"c16","name":"Kestrel","portrait":0,"icon":"flag","role":"damage","roleLabel":"Ranger","locked":false,"takenBy":""}],"players":[{"id":"p1","slot":1,"at":"c7","ready":false},{"id":"p2","slot":2,"at":"c7","ready":true},{"id":"p3","slot":3,"at":"c7","ready":false},{"id":"p4","slot":4,"at":"c7","ready":true}]},
};
