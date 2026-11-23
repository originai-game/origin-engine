import { For, Scroll, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
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
const BUTTON = `flex-1 basis-40 flex-row items-center justify-center gap-x-1 min-h-11 px-3 text-caption font-bold tracking-label uppercase text-fg nine-plate-cut bg-glass-hi hover:bg-raised-hi disabled:opacity-45 ${packClass("Button", "plate")}`;

/**
 * A spectator's controls: the subjects the game offers to follow, the one
 * followed, previous and next, free camera and Leave. It reports intents;
 * the game moves the camera.
 */
export function ArenaSpectator(props: ArenaSpectatorProps) {
  const heading = () => (props.title ?? "") != "" ? props.title : t("spectator.title");
  const followed = (id: string) => !props.freeCamera && props.selected == id;
  return (
    <View
      class={`flex-col gap-y-2 w-full max-h-full p-3 nine-panel-cut bg-layer-1 text-fg ${packClass("Panel", "frame")}`}
      label={heading()}
      busy={props.loading}
    >
      <Text class="text-xs leading-snug font-semibold tracking-label uppercase text-fg-muted">{heading()}</Text>
      <Show when={(props.detail ?? "") != ""}>
        <Text class="text-caption font-semibold text-fg-muted">{props.detail}</Text>
      </Show>
      <Text class="text-title leading-tight font-bold uppercase tracking-sm text-fg" role="status">
        {props.freeCamera ? t("spectator.free") : props.selected ? t("spectator.following") : t("spectator.choose")}
      </Text>
      <Show when={props.loading} fallback={
        <Show when={(props.targets ?? []).length}>
          <Scroll class="min-h-0">
            <View class="flex-col gap-y-1 pb-4" role="group" label={t("spectator.targets")}>
              <For each={props.targets ?? []} key={(p) => p.id}>
                {(p) => (
                  <View
                    class={`flex-col gap-y-0.25 w-full min-h-11 py-1 px-3 text-start hover:bg-surface-hover ${followed(p().id) ? packClass("Row", "frame", "selected") : packClass("Row", "frame")} disabled:text-fg-muted transition duration-317`}
                    role="button"
                    label={p().name}
                    pressed={followed(p().id)}
                    disabled={p().disabled}
                    onPress={(e) => props.onFollow?.(e, p().id)}
                    data-selected={followed(p().id)}
                  >
                    <Text class="text-reading leading-tight font-bold text-fg">{p().name}</Text>
                    <Show when={(p().team ?? "") != ""}>
                      <Text class="text-xs leading-snug font-semibold tracking-label uppercase text-fg-muted">{p().team}</Text>
                    </Show>
                    <Show when={(p().detail ?? "") != ""}>
                      <Text class="text-caption text-fg-muted">{p().detail}</Text>
                    </Show>
                    <Show when={followed(p().id)}>
                      <Text class="text-caption text-fg-muted">{t("spectator.current")}</Text>
                    </Show>
                  </View>
                )}
              </For>
            </View>
          </Scroll>
        </Show>
      }>
        <Text class="text-caption font-semibold text-fg-muted" role="status">{t("spectator.loading")}</Text>
      </Show>
      <Show when={!props.loading && (props.targets ?? []).length == 0}>
        <Text class="text-caption font-semibold text-fg-muted" role="status">{t("spectator.empty")}</Text>
      </Show>
      <View class="flex-row flex-wrap gap-x-1 gap-y-1">
        <View class={BUTTON} role="button" label={t("spectator.previous")} disabled={props.loading || !(props.canCycle ?? true) || (props.targets ?? []).length == 0} onPress={(e) => props.onPrevious?.(e)}>
          <Text>{t("spectator.previous")}</Text>
        </View>
        <View class={BUTTON} role="button" label={t("spectator.next")} disabled={props.loading || !(props.canCycle ?? true) || (props.targets ?? []).length == 0} onPress={(e) => props.onNext?.(e)}>
          <Text>{t("spectator.next")}</Text>
        </View>
        <Show when={props.canFree ?? true}>
          <View class={BUTTON} role="button" label={t("spectator.free")} pressed={props.freeCamera} disabled={props.loading} onPress={(e) => props.onFree?.(e)} data-selected={props.freeCamera === true}>
            <Text>{t("spectator.free")}</Text>
          </View>
        </Show>
        <View class={BUTTON} role="button" label={t("spectator.leave")} disabled={!(props.canLeave ?? true)} onPress={(e) => props.onLeave?.(e)}>
          <Text>{t("spectator.leave")}</Text>
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
