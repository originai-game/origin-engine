import { For, Icon, Image, Scroll, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./arena.en.ogt";

export interface LobbySlot {
  /** The seat's key, reported back with invite. */
  id: string;
  /** The seat's player; empty when open. */
  name?: string;
  /** The seat's portrait texture (0 shows the icon). */
  portrait?: Texture | 0;
  /** The seat's player's icon, shown while no portrait is set. */
  icon?: string;
  /** The seat is free. */
  open?: boolean;
  /** The seat's player hosts. */
  host?: boolean;
  /** The seat's player is ready, in words (Ready or Not ready). */
  ready?: boolean;
  /** The seat is the local player's. */
  local?: boolean;
}

export interface ArenaLobbyProps {
  /** The lobby's heading (the match's name); empty takes the kit's (Lobby). */
  title?: string;
  /** A line beside the heading as the game words it (Ranked · 2 v 2, Waiting for the host); empty hides it. */
  detail?: string;
  /** The lobby's seats in the game's order. */
  slots?: ReadonlyArray<LobbySlot>;
  /** Open seats offer Invite (false: they read Open slot only). */
  canInvite?: boolean;
  /** The local player is ready: the button reads Cancel ready (unready) instead of Ready up (ready). */
  isReady?: boolean;
  /** The ready button is offered (false: disabled). */
  canReady?: boolean;
  /** The local player hosts: Start match is shown. */
  isHost?: boolean;
  /** Start match is offered (false: disabled; the game decides when the match can start). */
  canStart?: boolean;
  /** Invite to an open seat. */
  onInvite?: (e: UiEvent, id: string) => void;
  /** Ready up. */
  onReady?: (e: UiEvent) => void;
  /** Cancel ready. */
  onUnready?: (e: UiEvent) => void;
  /** The host starts the match. */
  onStart?: (e: UiEvent) => void;
}

/**
 * The pre-match lobby: a heading and the match's line, one row per seat
 * (portrait, name, Host, You, and Ready or Not ready in words), an open
 * seat reading Open slot with an Invite button (invite, the seat's key),
 * and the local player's actions: Ready up (ready; also the details
 * action, X or pad Y) or Cancel ready (unready), and for the host Start
 * match (start), disabled until the game allows it. The game owns seats,
 * readiness and when a match may start. Approximations: the seat's and
 * frame's cut corners are rounded, the portraits' octagon edge is
 * rounded, the scroll fade at the list's foot is dropped, and a long
 * word wraps without breaking.
 */
const CARD = "w-full max-h-full p-4 flex-col gap-y-3 text-fg bg-linear-to-b from-[#232837ed] to-[#1a1e29f2] shadow-[inset_0px_0px_0px_1px_rgb(255_255_255/0.14),inset_0px_1px_0px_0px_rgb(255_255_255/0.5)] rounded-tl-[0.714rem] compact:p-2";
const BUTTON = "flex-row items-center justify-center gap-x-1 min-h-11 px-4 min-w-[8.571rem] text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg bg-linear-to-b from-[#2c3140eb] to-[#1b1f29f2] shadow-[inset_0px_0px_0px_1px_rgb(255_255_255/0.24),inset_0px_1px_0px_0px_rgb(255_255_255/0.6)] rounded-tl-[0.429rem] hover:bg-none hover:bg-linear-to-b hover:from-[#373d4ff2] hover:to-[#232837f5] disabled:opacity-45 compact:flex-1";
const KEYCAP = "shrink-0 size-6 min-w-6 text-center text-[0.929rem] leading-[1.6] font-bold text-[#262b38] bg-linear-to-b from-white to-[#d8dbe2] rounded-[0.286rem] shadow-[0px_0.143rem_0px_#0000008c,inset_0px_-0.143rem_0px_#0000002e] text-shadow-none pointer-coarse:hidden";
const TAG = "shrink-0 py-0 px-1 text-[0.857rem] leading-[1.5] font-bold tracking-[0.14em] uppercase whitespace-nowrap rounded-tl-[0.286rem]";
const PORTRAIT = "shrink-0 size-[3.714rem] p-[3px] rounded-[0.857rem] bg-linear-to-b from-white/75 via-white/22 via-45% to-white/14 compact:size-[2.857rem]";
const PORTRAIT_OPEN = "data-open:p-px data-open:bg-none data-open:bg-[#ffffff47]";
const SEAT = "flex-row flex-wrap items-center gap-x-1 gap-y-0.5 min-h-11 py-1 px-2 bg-linear-to-r from-[#282e3cc7] to-[#1c202bd1] shadow-[inset_0px_0px_0px_1px_rgb(255_255_255/0.1)] rounded-tl-[0.429rem] compact:min-h-9 compact:py-0.5 compact:px-1 compact:gap-x-1";

export function ArenaLobby(props: ArenaLobbyProps) {
  const heading = () => (props.title ?? "") != "" ? props.title : t("arena.lobby.title");
  const slots = () => props.slots ?? [];
  return (
    <View class={CARD} label={heading()}>
      <View class="flex-row flex-wrap items-baseline gap-x-3 gap-y-1">
        <Text class="min-w-0 text-[1.714rem] leading-[1.1] font-bold uppercase tracking-[0.04em] text-fg">{heading()}</Text>
        <Show when={(props.detail ?? "") != ""}>
          <Text class="min-w-0 text-[0.929rem] font-semibold text-white/66">{props.detail}</Text>
        </Show>
      </View>
      <Show when={slots().length} fallback={
        <Text class="p-4 text-center text-[1.143rem] text-white/66" role="status">{t("arena.lobby.empty")}</Text>
      }>
        <Scroll class="min-h-0">
          <View class="flex-col gap-y-1" role="list" label={t("arena.lobby.players")}>
            <For each={slots()} key={(s) => s.id}>
              {(s) => (
                <View
                  class={`${SEAT} ${PORTRAIT_OPEN} data-local:shadow-[inset_0px_0px_0px_1px_rgb(56_232_255/0.45),inset_0px_2px_0px_0px_#38e8ff] data-open:bg-none data-open:bg-[#ffffff0a]`}
                  role="listitem"
                  label={s().open ? t("arena.lobby.open") : s().name}
                  data-local={s().local === true}
                  data-open={s().open === true}
                >
                  <View class={`${PORTRAIT} ${s().open ? "p-px bg-none bg-[#ffffff47]" : ""}`}>
                    <View class="size-full rounded-[0.714rem] bg-[#1d212c] flex-row items-center justify-center overflow-hidden">
                      <Show when={(s().portrait ?? 0) != 0 && !s().open} fallback={
                        <Icon class={s().open ? "size-6 text-white/66" : "size-6 text-fg"} glyph={s().open ? "plus" : (s().icon ?? "")} />
                      }>
                        <Image class="size-full" src={s().portrait ?? 0} />
                      </Show>
                    </View>
                  </View>
                  <View class="flex-1 min-w-0 flex-col gap-y-0.5 compact:basis-[6.857rem]">
                    <Text class={`text-start text-[1.071rem] leading-[1.15] font-bold text-fg ${s().open ? "text-white/66" : ""}`}>{s().open ? t("arena.lobby.open") : s().name}</Text>
                    <Show when={s().host || s().local}>
                      <View class="flex-row flex-wrap gap-x-0.5 gap-y-0.5">
                        <Show when={s().host}>
                          <Text class={`${TAG} text-[#5a3d05] bg-linear-to-b from-[#ffe9bd] to-[#ffcf5a]`}>{t("arena.lobby.host")}</Text>
                        </Show>
                        <Show when={s().local}>
                          <Text class={`${TAG} text-[#15181f] bg-[#38e8ff]`}>{t("arena.you")}</Text>
                        </Show>
                      </View>
                    </Show>
                  </View>
                  <Show when={!s().open}>
                    <View
                      class={`flex-row items-center gap-x-1 whitespace-nowrap text-[0.929rem] font-bold tracking-[0.14em] uppercase compact:ml-auto ${s().ready ? "text-[#55e39a] shadow-[0px_0px_0.571rem_rgb(85_227_154/0.7)]" : "text-white/66"}`}
                    >
                      <Icon class="size-4" glyph={s().ready ? "check" : "clock"} />
                      <Text class="whitespace-nowrap">{s().ready ? t("arena.lobby.ready") : t("arena.lobby.notReady")}</Text>
                    </View>
                  </Show>
                  <Show when={s().open && props.canInvite}>
                    <View
                      class="flex-none flex-row items-center gap-x-0.5 min-h-8 px-4 text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg bg-linear-to-b from-[#2c3140eb] to-[#1b1f29f2] shadow-[inset_0px_0px_0px_1px_rgb(255_255_255/0.24),inset_0px_1px_0px_0px_rgb(255_255_255/0.6)] rounded-tl-[0.429rem] hover:bg-none hover:bg-linear-to-b hover:from-[#373d4ff2] hover:to-[#232837f5] compact:ml-auto"
                      role="button"
                      label={t("arena.lobby.invite")}
                      onPress={(e) => props.onInvite?.(e, s().id)}
                    >
                      <Icon class="size-4" glyph="plus" />
                      <Text class="text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg">{t("arena.lobby.invite")}</Text>
                    </View>
                  </Show>
                </View>
              )}
            </For>
          </View>
        </Scroll>
      </Show>
      <View class="flex-row flex-wrap justify-end gap-x-1 gap-y-1 pb-[calc(env(hud-inset-bottom)+0px)] compact:flex-wrap">
        <Show when={props.isReady} fallback={
          <View class={BUTTON} role="button" label={t("arena.lobby.readyUp")} disabled={!(props.canReady ?? false)} onPress={(e) => props.onReady?.(e)}>
            <Text class={KEYCAP} glyph="details" />
            <Text class="text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg">{t("arena.lobby.readyUp")}</Text>
          </View>
        }>
          <View class={BUTTON} role="button" label={t("arena.lobby.unready")} disabled={!(props.canReady ?? false)} onPress={(e) => props.onUnready?.(e)}>
            <Text class={KEYCAP} glyph="details" />
            <Text class="text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg">{t("arena.lobby.unready")}</Text>
          </View>
        </Show>
        <Show when={props.isHost}>
          <View
            class="flex-row items-center justify-center gap-x-1 min-h-11 px-4 min-w-[8.571rem] text-[0.929rem] font-bold tracking-[0.14em] uppercase text-[#15181f] bg-[#38e8ff] hover:bg-[#5defff] rounded-tl-[0.429rem] disabled:opacity-45 compact:flex-1"
            role="button"
            label={t("arena.lobby.start")}
            disabled={!(props.canStart ?? false)}
            onPress={(e) => props.onStart?.(e)}
          >
            <Text class="text-[0.929rem] font-bold tracking-[0.14em] uppercase text-[#15181f]">{t("arena.lobby.start")}</Text>
          </View>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ArenaLobbyProps>> = {
  default: {"title":"","detail":"Ranked · 2 v 2","canInvite":true,"isReady":false,"canReady":true,"isHost":true,"canStart":false,"slots":[{"id":"s1","name":"Rhea","portrait":0,"icon":"sword","open":false,"host":true,"ready":false,"local":true},{"id":"s2","name":"Kai","portrait":0,"icon":"shield","open":false,"host":false,"ready":true,"local":false},{"id":"s3","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false},{"id":"s4","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false}]},
  full: {"title":"","detail":"","canInvite":true,"isReady":true,"canReady":true,"isHost":true,"canStart":true,"slots":[{"id":"s1","name":"Rhea","portrait":0,"icon":"sword","open":false,"host":true,"ready":true,"local":true},{"id":"s2","name":"Kai","portrait":0,"icon":"shield","open":false,"host":false,"ready":true,"local":false},{"id":"s3","name":"Ana","portrait":0,"icon":"heart","open":false,"host":false,"ready":true,"local":false},{"id":"s4","name":"Rook","portrait":0,"icon":"bolt","open":false,"host":false,"ready":true,"local":false}]},
  guest: {"title":"","detail":"Waiting for the host","canInvite":false,"isReady":true,"canReady":true,"isHost":false,"canStart":false,"slots":[{"id":"s1","name":"Kai","portrait":0,"icon":"shield","open":false,"host":true,"ready":false,"local":false},{"id":"s2","name":"Rhea","portrait":0,"icon":"sword","open":false,"host":false,"ready":true,"local":true},{"id":"s3","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false}]},
  disabled: {"title":"","detail":"Matchmaking is down for maintenance","canInvite":false,"isReady":false,"canReady":false,"isHost":true,"canStart":false,"slots":[{"id":"s1","name":"Rhea","portrait":0,"icon":"sword","open":false,"host":true,"ready":false,"local":true},{"id":"s2","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false}]},
  overflow: {"title":"Championship Series · Grand Finals · Deciding match lobby","detail":"Custom · 5 v 5 · Draft pick · Tournament rules · Spectators allowed","canInvite":true,"isReady":false,"canReady":true,"isHost":true,"canStart":false,"slots":[{"id":"s1","name":"Maximilian Vandenberghe-Okonkwo","portrait":0,"icon":"crown","open":false,"host":true,"ready":true,"local":false},{"id":"s2","name":"Rhea","portrait":0,"icon":"sword","open":false,"host":false,"ready":false,"local":true},{"id":"s3","name":"Ana","portrait":0,"icon":"heart","open":false,"host":false,"ready":true,"local":false},{"id":"s4","name":"Kai","portrait":0,"icon":"shield","open":false,"host":false,"ready":false,"local":false},{"id":"s5","name":"Jo","portrait":0,"icon":"music","open":false,"host":false,"ready":true,"local":false},{"id":"s6","name":"Abernathy-Whitcombe","portrait":0,"icon":"bolt","open":false,"host":false,"ready":true,"local":false},{"id":"s7","name":"Wren","portrait":0,"icon":"leaf","open":false,"host":false,"ready":false,"local":false},{"id":"s8","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false},{"id":"s9","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false},{"id":"s10","name":"","portrait":0,"icon":"","open":true,"host":false,"ready":false,"local":false}]},
};
