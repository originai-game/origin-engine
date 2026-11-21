import { For, Icon, Image, Scroll, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import { HudTimer } from "../hud/hud-timer";
import "./arena.en.ogt";

export interface DraftSlot {
  /** The slot's key. */
  id: string;
  /** a or b. */
  team?: string;
  /** ban or pick. */
  kind?: string;
  /** The character placed (empty: the slot is open). */
  name?: string;
  /** The character's portrait texture (0 shows the icon). */
  portrait?: Texture | 0;
  /** An icon name, shown while no portrait is set. */
  icon?: string;
  /** Who the pick belongs to (empty hides). */
  player?: string;
  /** The slot being filled now (Banning, Picking). */
  active?: boolean;
}

export interface ArenaDraftProps {
  /** The draft's heading; empty takes the kit's (Draft). */
  title?: string;
  /** The phase the game is in: ban, pick or done (each named in words in the band). */
  phase?: string;
  /** The team choosing now: a, b, or empty for neither (its column is marked current and named in the band). */
  active?: string;
  /** The first team's name; empty takes the kit's (Blue team). */
  nameA?: string;
  /** The second team's name; empty takes the kit's (Red team). */
  nameB?: string;
  /** Both teams' ban and pick slots in the game's order. */
  slots?: ReadonlyArray<DraftSlot>;
  /** Seconds left in the turn as the game counts them (a HudTimer); negative hides it. Default -1. */
  timeLeft?: number;
  /** The turn's whole time, for the timer's ring (0: no ring). */
  timeTotal?: number;
  /** Lock in is offered (false: disabled; the game decides when the local player may lock in). */
  canConfirm?: boolean;
  /** Lock in. */
  onConfirm?: (e: UiEvent) => void;
}

/**
 * A ban and pick draft between two teams on a smoked cut card: the band
 * names the phase (Ban phase, Pick phase, Draft complete) and the team to
 * choose, with the turn's countdown as the game counts it (HudTimer); a
 * column per team holds its bans (struck, Banned) and its picks (the
 * character and its player, or Open), the slot being filled marked
 * Banning or Picking and lit white. Lock in (confirm; also the details
 * action, X or pad Y) reports once and is disabled when the game refuses.
 * The draft decides no turn, legality or timing. Approximations: the
 * portraits' octagon edges are rounded, a banned portrait's greying
 * filter is an opacity, the active slot's glow filter is dropped (its rim
 * is kept), the team inks are the default blue and red (the game's
 * seed-team colours have no class surface), the figures' condensed
 * stretch is dropped, and the scroll fade at the teams' foot is dropped,
 * and a long word wraps without breaking.
 */
const CARD = `w-full max-h-full p-4 flex-col gap-y-3 text-fg nine-panel-cut bg-[#232837ed] compact:p-2 ${packClass("Panel", "frame")}`;
const TAG = `shrink-0 py-0 px-1 text-xs leading-[1.5] font-bold tracking-[0.14em] uppercase text-fg whitespace-nowrap nine-chip-cut ${packClass("Chip", "frame")}`;
const KEYCAP = "shrink-0 size-6 min-w-6 text-center text-caption leading-[1.6] font-bold text-[#262b38] bg-linear-to-b from-white to-[#d8dbe2] rounded-[0.286rem] shadow-[0px_0.143rem_0px_#0000008c,inset_0px_-0.143rem_0px_#0000002e] text-shadow-none pointer-coarse:hidden";
const CONFIRM = `flex-row items-center justify-center gap-x-1 min-h-11 px-4 min-w-[8.571rem] text-caption font-bold tracking-[0.14em] uppercase text-[#15181f] nine-plate-cut bg-[#38e8ff] hover:bg-[#5defff] disabled:opacity-45 compact:flex-1 ${packClass("Button", "plate", "primary")}`;

function Portrait(props: { slot: () => DraftSlot; small?: boolean }) {
  const s = props.slot;
  const hasName = () => (s().name ?? "") != "";
  return (
    <View
      class={`relative shrink-0 ${props.small ? "size-[3.143rem]" : "size-[3.714rem]"} compact:size-[2.857rem] p-[3px] rounded-[0.857rem] bg-linear-to-b from-white/75 via-white/22 via-45% to-white/14 overflow-hidden data-empty:p-px data-empty:bg-none data-empty:bg-white/28`}
      data-empty={!hasName()}
    >
      <View class="size-full rounded-[0.571rem] overflow-hidden flex-row items-center justify-center bg-[#1a1e29]">
        <Show when={(s().portrait ?? 0) != 0} fallback={
          <Show when={(s().icon ?? "") != ""}>
            <Icon class="size-4 text-fg data-banned:text-fg/66 data-banned:opacity-40" name={s().icon ?? ""} data-banned={hasName()} />
          </Show>
        }>
          <Image class="size-full data-banned:opacity-40" src={s().portrait ?? 0} data-banned={hasName()} />
        </Show>
      </View>
      <Show when={hasName()}>
        <Icon class="absolute inset-0 size-full text-danger" name="close" />
      </Show>
    </View>
  );
}

function DraftTeam(props: { team: "a" | "b"; name: string; active: boolean; slots: ReadonlyArray<DraftSlot> }) {
  const blue = props.team == "a";
  const ink = blue ? "text-team1" : "text-team2";
  const rule = blue ? "shadow-[inset_0px_-2px_0px_0px_#50aaff]" : "shadow-[inset_0px_-2px_0px_0px_#ff6a5e]";
  const wash = blue
    ? "bg-linear-to-r from-team1/16 to-entry-lo to-40% shadow-[inset_3px_0px_0px_0px_#50aaff]"
    : "bg-linear-to-r from-team2/16 to-entry-lo to-40% shadow-[inset_3px_0px_0px_0px_#ff6a5e]";
  const bans = () => (props.slots ?? []).filter((s) => s.kind == "ban");
  const picks = () => (props.slots ?? []).filter((s) => s.kind == "pick");
  return (
    <View class="min-w-0 flex-1 flex-col gap-y-1 compact:flex-none" role="group" label={props.name} current={props.active === true}>
      <Text role="heading" level={3} class={`pb-1 text-lg leading-[1.15] font-bold tracking-[0.04em] uppercase ${ink} ${rule}`}>{props.name}</Text>
      <Text class="text-xs leading-[1.3] font-semibold tracking-[0.14em] uppercase text-fg/78 whitespace-nowrap">{t("arena.draft.bans")}</Text>
      <View class="flex-row flex-wrap gap-1" role="list" label={t("arena.draft.bans")}>
        <For each={bans()} key={(s) => s.id}>
          {(s) => (
            <View class="shrink basis-auto min-w-16 max-w-full flex-col items-center gap-y-0.5" role="listitem" current={s().active === true}>
              <Portrait slot={s} small={true} />
              <Text
                class={`${TAG} data-banning:bg-[#38e8ff] data-banning:text-[#15181f] data-banning:shadow-none data-open:text-fg/66`}
                data-banning={s().active === true}
                data-open={!s().active && (s().name ?? "") == ""}
              >
                {s().active ? t("arena.draft.banning") : (s().name ?? "") != "" ? t("arena.draft.banned") : t("arena.draft.open")}
              </Text>
              <Show when={(s().name ?? "") != ""}>
                <Text class="max-w-full text-center text-caption leading-[1.2] text-fg/66">{s().name}</Text>
              </Show>
            </View>
          )}
        </For>
      </View>
      <Text class="text-xs leading-[1.3] font-semibold tracking-[0.14em] uppercase text-fg/78 whitespace-nowrap">{t("arena.draft.picks")}</Text>
      <View class="flex-col gap-y-1" role="list" label={t("arena.draft.picks")}>
        <For each={picks()} key={(s) => s.id}>
          {(s) => (
            <View
              class={`flex-row items-center gap-x-2 min-h-11 py-1 px-2 compact:gap-x-1 compact:py-0.5 compact:px-1 ${wash} data-active:shadow-[inset_0px_0px_0px_2px_#38e8ff,inset_0px_3px_0px_0px_rgb(255_255_255/0.5)]`}
              role="listitem"
              current={s().active === true}
              data-active={s().active === true}
            >
              <Portrait slot={s} />
              <View class="flex-1 min-w-0 flex-col">
                <Text
                  class="min-w-0 text-start text-reading leading-[1.15] font-bold text-fg data-empty:text-fg/66"
                  data-empty={(s().name ?? "") == ""}
                >
                  {(s().name ?? "") != "" ? s().name : t("arena.draft.open")}
                </Text>
                <View class="min-w-0 flex-row flex-wrap gap-x-1">
                  <Show when={(s().player ?? "") != ""}>
                    <Text class="min-w-0 text-start text-caption text-fg/66 data-active:text-fg/78" data-active={s().active === true}>{s().player}</Text>
                  </Show>
                  <Show when={s().active === true}>
                    <Text class={`${TAG} bg-[#38e8ff] text-[#15181f] shadow-none`}>{t("arena.draft.picking")}</Text>
                  </Show>
                </View>
              </View>
            </View>
          )}
        </For>
      </View>
    </View>
  );
}

export function ArenaDraft(props: ArenaDraftProps) {
  const heading = () => (props.title ?? "") != "" ? props.title : t("arena.draft.title");
  const slots = () => props.slots ?? [];
  const nameA = () => props.nameA || t("arena.team.a");
  const nameB = () => props.nameB || t("arena.team.b");
  return (
    <View class={CARD} label={heading()}>
      <View class="flex-none flex-row flex-wrap items-start justify-between gap-x-3 gap-y-1 w-full">
        <View class="flex-1 min-w-0 flex-row flex-wrap items-center gap-x-3 gap-y-1">
          <Text class="min-w-0 text-[1.714rem] leading-[1.1] font-bold tracking-[0.04em] uppercase text-fg">{heading()}</Text>
          <Show when={props.phase == "ban" || props.phase == "pick" || props.phase == "done"}>
            <Text class="shrink-0 py-0 px-1 text-xs leading-[1.5] font-bold tracking-[0.14em] uppercase whitespace-nowrap nine-chip-cut bg-gold text-[#1e1604]">
              {props.phase == "ban" ? t("arena.draft.ban") : props.phase == "pick" ? t("arena.draft.pick") : t("arena.draft.done")}
            </Text>
          </Show>
          <Show when={props.active == "a" || props.active == "b"}>
            <Text class="min-w-0 text-caption font-semibold text-fg" role="status">
              {t("arena.draft.turn", { team: props.active == "a" ? nameA() : nameB() })}
            </Text>
          </Show>
        </View>
        <Show when={(props.timeLeft ?? -1) >= 0}>
          <View class="flex-none">
            <HudTimer seconds={Math.max(props.timeLeft ?? 0, 0)} total={props.timeTotal ?? 0} countdown={true} warnAt={10} paused={false} label={t("arena.draft.timeLeft")} />
          </View>
        </Show>
      </View>
      <Show when={slots().length} fallback={
        <Text class="p-4 text-center text-base text-fg/66" role="status">{t("arena.draft.empty")}</Text>
      }>
        <Scroll class="min-h-0">
          <View class="flex-row items-start gap-x-3 w-full compact:flex-col compact:items-stretch">
            <DraftTeam team="a" name={nameA()} active={props.active == "a"} slots={slots().filter((s) => s.team == "a")} />
            <DraftTeam team="b" name={nameB()} active={props.active == "b"} slots={slots().filter((s) => s.team == "b")} />
          </View>
        </Scroll>
      </Show>
      <Show when={props.phase != "done"}>
        <View class="flex-none flex-row justify-end w-full pb-[calc(env(hud-inset-bottom)+0px)]">
          <View
            class={CONFIRM}
            role="button"
            label={t("arena.draft.confirm")}
            disabled={!(props.canConfirm ?? false) || props.phase == "done"}
            onPress={(e) => props.onConfirm?.(e)}
          >
            <Text class={KEYCAP} glyph="details" />
            <Text class="text-caption font-bold tracking-[0.14em] uppercase text-[#15181f]">{t("arena.draft.confirm")}</Text>
          </View>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ArenaDraftProps>> = {
  default: {"title":"","phase":"pick","active":"a","nameA":"","nameB":"","timeLeft":21,"timeTotal":30,"canConfirm":true,"slots":[{"id":"ab1","team":"a","kind":"ban","name":"Oarsman","portrait":0,"icon":"skull","player":"","active":false},{"id":"ab2","team":"a","kind":"ban","name":"Lurker","portrait":0,"icon":"drop","player":"","active":false},{"id":"ap1","team":"a","kind":"pick","name":"Brenna","portrait":0,"icon":"shield","player":"Kai","active":false},{"id":"ap2","team":"a","kind":"pick","name":"","portrait":0,"icon":"","player":"You","active":true},{"id":"ap3","team":"a","kind":"pick","name":"","portrait":0,"icon":"","player":"Ana","active":false},{"id":"bb1","team":"b","kind":"ban","name":"Sela","portrait":0,"icon":"heart","player":"","active":false},{"id":"bb2","team":"b","kind":"ban","name":"Idris","portrait":0,"icon":"bolt","player":"","active":false},{"id":"bp1","team":"b","kind":"pick","name":"Vex","portrait":0,"icon":"sword","player":"Rook","active":false},{"id":"bp2","team":"b","kind":"pick","name":"Maren","portrait":0,"icon":"leaf","player":"Wren","active":false},{"id":"bp3","team":"b","kind":"pick","name":"","portrait":0,"icon":"","player":"Quill","active":false}]},
  ban: {"title":"","phase":"ban","active":"b","nameA":"","nameB":"","timeLeft":6,"timeTotal":20,"canConfirm":false,"slots":[{"id":"ab1","team":"a","kind":"ban","name":"Oarsman","portrait":0,"icon":"skull","player":"","active":false},{"id":"ab2","team":"a","kind":"ban","name":"","portrait":0,"icon":"","player":"","active":false},{"id":"ap1","team":"a","kind":"pick","name":"","portrait":0,"icon":"","player":"Kai","active":false},{"id":"ap2","team":"a","kind":"pick","name":"","portrait":0,"icon":"","player":"You","active":false},{"id":"bb1","team":"b","kind":"ban","name":"","portrait":0,"icon":"","player":"","active":true},{"id":"bb2","team":"b","kind":"ban","name":"","portrait":0,"icon":"","player":"","active":false},{"id":"bp1","team":"b","kind":"pick","name":"","portrait":0,"icon":"","player":"Rook","active":false},{"id":"bp2","team":"b","kind":"pick","name":"","portrait":0,"icon":"","player":"Wren","active":false}]},
  done: {"title":"","phase":"done","active":"","nameA":"Iron Wolves","nameB":"Ember Court","timeLeft":-1,"timeTotal":0,"canConfirm":false,"slots":[{"id":"ab1","team":"a","kind":"ban","name":"Oarsman","portrait":0,"icon":"skull","player":"","active":false},{"id":"ap1","team":"a","kind":"pick","name":"Brenna","portrait":0,"icon":"shield","player":"Kai","active":false},{"id":"ap2","team":"a","kind":"pick","name":"Vex","portrait":0,"icon":"sword","player":"You","active":false},{"id":"bb1","team":"b","kind":"ban","name":"Sela","portrait":0,"icon":"heart","player":"","active":false},{"id":"bp1","team":"b","kind":"pick","name":"Idris","portrait":0,"icon":"bolt","player":"Rook","active":false},{"id":"bp2","team":"b","kind":"pick","name":"Maren","portrait":0,"icon":"leaf","player":"Wren","active":false}]},
  overflow: {"title":"Championship Series · Grand Finals · Deciding match draft","phase":"pick","active":"b","nameA":"The Unbroken Wardens of the Ninefold Gate","nameB":"Ember Court Invitational Reserve","timeLeft":3,"timeTotal":30,"canConfirm":false,"slots":[{"id":"ab1","team":"a","kind":"ban","name":"Ysolde the Unbroken, Warden of the Ninefold Gate","portrait":0,"icon":"crown","player":"","active":false},{"id":"ab2","team":"a","kind":"ban","name":"Lurker","portrait":0,"icon":"drop","player":"","active":false},{"id":"ab3","team":"a","kind":"ban","name":"Hollow","portrait":0,"icon":"gem","player":"","active":false},{"id":"ap1","team":"a","kind":"pick","name":"Brenna","portrait":0,"icon":"shield","player":"Maximilian Vandenberghe-Okonkwo","active":false},{"id":"ap2","team":"a","kind":"pick","name":"Vex","portrait":0,"icon":"sword","player":"You","active":false},{"id":"ap3","team":"a","kind":"pick","name":"Tamsin","portrait":0,"icon":"music","player":"Ana","active":false},{"id":"ap4","team":"a","kind":"pick","name":"Gorrum","portrait":0,"icon":"bomb","player":"Kai","active":false},{"id":"ap5","team":"a","kind":"pick","name":"Quill","portrait":0,"icon":"star","player":"Jo","active":false},{"id":"bb1","team":"b","kind":"ban","name":"Sela","portrait":0,"icon":"heart","player":"","active":false},{"id":"bb2","team":"b","kind":"ban","name":"Idris","portrait":0,"icon":"bolt","player":"","active":false},{"id":"bb3","team":"b","kind":"ban","name":"Ashen Pyre","portrait":0,"icon":"fire","player":"","active":false},{"id":"bp1","team":"b","kind":"pick","name":"Maren","portrait":0,"icon":"leaf","player":"Rook","active":false},{"id":"bp2","team":"b","kind":"pick","name":"Kestrel","portrait":0,"icon":"flag","player":"Wren","active":false},{"id":"bp3","team":"b","kind":"pick","name":"Wren","portrait":0,"icon":"apple","player":"Abernathy-Whitcombe","active":false},{"id":"bp4","team":"b","kind":"pick","name":"","portrait":0,"icon":"","player":"Moreau","active":true},{"id":"bp5","team":"b","kind":"pick","name":"","portrait":0,"icon":"","player":"Laurent","active":false}]},
};
