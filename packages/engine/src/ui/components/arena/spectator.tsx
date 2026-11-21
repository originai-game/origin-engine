import { For, Scroll, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./spectator.en.ogt";

export interface SpectatorTarget {
  id: string;
  /** What a reader calls them. */
  name: string;
  /** Their side as the game words it; empty hides it. */
  team?: string;
  /** A line about them (Alive, Respawning · 4s, Disconnected). */
  detail?: string;
  /** The game decides which subjects can be followed. */
  disabled?: boolean;
}

export interface ArenaSpectatorProps {
  /** Heading; empty uses Spectating. */
  title?: string;
  /** Match status, broadcast delay or disabled reason, as the game words it. */
  detail?: string;
  /** The subjects the game offers. follow reports their stable id. */
  targets?: ReadonlyArray<SpectatorTarget>;
  /** The followed subject id. Empty means no chosen subject; freeCamera takes precedence over its selected state. */
  selected?: string;
  /** Free camera active; choosing a subject only reports follow, leaving the game to change camera mode. */
  freeCamera?: boolean;
  /** Show and enable the free camera action; false omits it. Default true. */
  canFree?: boolean;
  /** Enable previous and next. The game owns cycling order and wrapping. Default true. */
  canCycle?: boolean;
  /** Enable Leave; false keeps the exit visible but disabled. Default true. */
  canLeave?: boolean;
  /** Waiting for subjects; hides stale rows and disables camera actions, but not Leave. */
  loading?: boolean;
  /** Follow a subject. */
  onFollow?: (e: UiEvent, id: string) => void;
  /** Previous subject. */
  onPrevious?: (e: UiEvent) => void;
  /** Next subject. */
  onNext?: (e: UiEvent) => void;
  /** Free camera. */
  onFree?: (e: UiEvent) => void;
  /** Leave. */
  onLeave?: (e: UiEvent) => void;
}

/**
 * The spectator's plate over the match: smoked glass with a cut corner,
 * the mode in condensed capitals, a row per player with a hairline edge,
 * the one being watched washed in the accent with its edge lit.
 * Approximations: the plate's cut corner is rounded, the chosen row's wash
 * is flat, the scroll fade at the list's foot is dropped, and the ghost
 * Leave wears the standard button look.
 */
const BUTTON = "flex-1 basis-40 flex-row items-center justify-center gap-x-1 min-h-11 px-3 text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg bg-linear-to-b from-[#2c3140eb] to-[#1b1f29f2] shadow-[inset_0px_0px_0px_1px_rgb(255_255_255/0.24),inset_0px_1px_0px_0px_rgb(255_255_255/0.6)] rounded-tl-[0.429rem] hover:bg-none hover:bg-linear-to-b hover:from-[#373d4ff2] hover:to-[#232837f5] disabled:opacity-45 data-selected:shadow-[inset_0px_0px_0px_0.286rem_#38e8ff,inset_0px_0.429rem_0px_0px_rgb(255_255_255/0.5)]";

export function ArenaSpectator(props: ArenaSpectatorProps) {
  const heading = () => (props.title ?? "") != "" ? props.title : t("spectator.title");
  const followed = (id: string) => !props.freeCamera && props.selected == id;
  return (
    <View
      class={`flex-col gap-y-2 w-full max-h-full p-3 rounded-tl-[0.714rem] bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-[inset_0px_0px_0px_1px_rgb(255_255_255/0.18),inset_0px_1px_0px_0px_rgb(255_255_255/0.55)] text-fg`}
      label={heading()}
      busy={props.loading}
    >
      <Text class="text-[0.857rem] leading-[1.3] font-semibold tracking-[0.14em] uppercase text-white/78">{heading()}</Text>
      <Show when={(props.detail ?? "") != ""}>
        <Text class="text-[0.929rem] font-semibold text-white/66">{props.detail}</Text>
      </Show>
      <Text class="text-[1.714rem] leading-[1.1] font-bold uppercase tracking-[0.04em] text-fg" role="status">
        {props.freeCamera ? t("spectator.free") : props.selected ? t("spectator.following") : t("spectator.choose")}
      </Text>
      <Show when={props.loading} fallback={
        <Show when={(props.targets ?? []).length}>
          <Scroll class="min-h-0">
            <View class="flex-col gap-y-1 pb-4" role="group" label={t("spectator.targets")}>
              <For each={props.targets ?? []} key={(p) => p.id}>
                {(p) => (
                  <View
                    class={`flex-col gap-y-[0.071rem] w-full min-h-11 py-1 px-3 text-start bg-linear-to-r from-[#282e3cc7] to-[#1c202bd1] shadow-[inset_3px_0px_0px_0px_rgb(255_255_255/0.3)] hover:bg-none hover:bg-linear-to-r hover:from-[#373d4ff2] hover:to-[#232837f5] data-selected:bg-[#38e8ff26] data-selected:shadow-[inset_0.429rem_0px_0px_0px_#38e8ff,inset_0px_1px_0px_0px_rgb(255_255_255/0.18)] disabled:text-white/66 transition duration-317`}
                    role="button"
                    label={p().name}
                    pressed={followed(p().id)}
                    disabled={p().disabled}
                    onPress={(e) => props.onFollow?.(e, p().id)}
                    data-selected={followed(p().id)}
                  >
                    <Text class="text-[1.071rem] leading-[1.15] font-bold text-fg">{p().name}</Text>
                    <Show when={(p().team ?? "") != ""}>
                      <Text class="text-[0.857rem] leading-[1.3] font-semibold tracking-[0.14em] uppercase text-white/78">{p().team}</Text>
                    </Show>
                    <Show when={(p().detail ?? "") != ""}>
                      <Text class="text-[0.929rem] text-white/66">{p().detail}</Text>
                    </Show>
                    <Show when={followed(p().id)}>
                      <Text class="text-[0.929rem] text-white/66">{t("spectator.current")}</Text>
                    </Show>
                  </View>
                )}
              </For>
            </View>
          </Scroll>
        </Show>
      }>
        <Text class="text-[0.929rem] font-semibold text-white/66" role="status">{t("spectator.loading")}</Text>
      </Show>
      <Show when={!props.loading && (props.targets ?? []).length == 0}>
        <Text class="text-[0.929rem] font-semibold text-white/66" role="status">{t("spectator.empty")}</Text>
      </Show>
      <View class="flex-row flex-wrap gap-x-1 gap-y-1">
        <View class={BUTTON} role="button" label={t("spectator.previous")} disabled={props.loading || !(props.canCycle ?? true) || (props.targets ?? []).length == 0} onPress={(e) => props.onPrevious?.(e)}>
          <Text class="text-fg">{t("spectator.previous")}</Text>
        </View>
        <View class={BUTTON} role="button" label={t("spectator.next")} disabled={props.loading || !(props.canCycle ?? true) || (props.targets ?? []).length == 0} onPress={(e) => props.onNext?.(e)}>
          <Text class="text-fg">{t("spectator.next")}</Text>
        </View>
        <Show when={props.canFree ?? true}>
          <View class={BUTTON} role="button" label={t("spectator.free")} pressed={props.freeCamera} disabled={props.loading} onPress={(e) => props.onFree?.(e)} data-selected={props.freeCamera === true}>
            <Text class="text-fg">{t("spectator.free")}</Text>
          </View>
        </Show>
        <View class={BUTTON} role="button" label={t("spectator.leave")} disabled={!(props.canLeave ?? true)} onPress={(e) => props.onLeave?.(e)}>
          <Text class="text-fg">{t("spectator.leave")}</Text>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ArenaSpectatorProps>> = {
  default: {"title":"","detail":"Ranked · Round 3 · Broadcast delay 10s","targets":[{"id":"rhea","name":"Rhea","team":"Blue","detail":"Alive","disabled":false},{"id":"kai","name":"Kai","team":"Red","detail":"Respawning · 4s","disabled":false},{"id":"ana","name":"Ana","team":"Blue","detail":"Disconnected","disabled":true}],"selected":"kai","freeCamera":false,"canFree":true,"canCycle":true,"canLeave":true,"loading":false},
  free: {"freeCamera":true},
  disabled: {"canFree":false,"canCycle":false,"canLeave":false,"detail":"Camera controls locked during replay transition"},
  loading: {"loading":true},
  empty: {"targets":[],"selected":"","detail":"Waiting for players to join"},
  overflow: {"title":"Championship Series · Grand Final spectator view","targets":[{"id":"long","name":"Maximilian Vandenberghe-Okonkwo","team":"The Northern Champions","detail":"Respawning at the eastern forward base in 12 seconds","disabled":false},{"id":"rhea","name":"Rhea","team":"Blue","detail":"Alive","disabled":false},{"id":"kai","name":"Kai","team":"Red","detail":"Alive","disabled":false}],"selected":"long"},
};
