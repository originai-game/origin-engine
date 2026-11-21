import type { Texture } from "@origingame/engine";
import {For, Image, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { EmptyState } from "../base/empty-state";
import { ErrorState } from "../base/error-state";
import { Spinner } from "../base/spinner";
import "./meta.en.ogt";

export interface NewsProps {
  /** Heading; empty uses News and updates. */
  title?: string;
  /** Keyed { id, title, date, kind, summary, body, image, unread, pinned, actionLabel } articles in the game's order. kind is game-formatted text (Patch notes, Event); body is plain multiline text, never HTML. image is a loaded texture handle or 0. Default []. */
  entries?: ReadonlyArray<{ id: string; title: string; date: string; kind: string; summary: string; body: string; image: Texture | 0; unread: boolean; pinned: boolean; actionLabel: string }>;
  /** Expanded article id, game-owned. A toggle reports choose with that key; the game may clear it to collapse. */
  selected?: string;
  /** Loading articles; no article actions are present. Default false. */
  loading?: boolean;
  /** Disable article toggles and actions. Default false. */
  disabled?: boolean;
  /** Failure explanation with retry; empty is no error. */
  error?: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The open event, with the row's key where it comes from a row. */
  onOpen?: (e: UiEvent, key?: string) => void;
}

export function News(props: NewsProps) {
  return (
    <View class={`border-rim bg-linear-to-b from-glass-hi to-glass-lo shadow-panel text-fg w-full max-h-full p-4 gap-3 rounded-panel border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title || t("meta.news.title")} busy={(props.loading ?? false)}>
      <View class={`min-h-12 border-b border-rim-soft py-1 px-4`}>
        <Text class={`font-display text-base font-bold leading-tight tracking-tight text-fg m-0`}>
          {props.title || t("meta.news.title")}
        </Text>
      </View>
      <Show when={(props.loading ?? false)} fallback={
          <Show when={props.error} fallback={
              <Show when={!(props.entries ?? []).length} fallback={
                  <Scroll class="shrink gap-2">
                    <View class={`w-full p-0 m-0 gap-3`}>
                      <For each={(props.entries ?? [])}>
                        {(entry) => (
                        <View>
                          <View class={`border-rim-soft nine-plate-cut border border-rim transition duration-317 items-start py-3 px-4 gap-2 border data-selected:border-accent data-down:opacity-45`} data-unread={entry().unread} data-pinned={entry().pinned}>
                            <View class="flex-row flex-wrap items-center w-full gap-2">
                              <Text class={`font-bold text-caption text-info tracking-label uppercase bg-well py-0 px-1 rounded-capsule shrink`}>
                                {entry().kind}
                              </Text>
                              <Text class={`text-caption text-fg-subtle shrink`}>
                                {entry().date}
                              </Text>
                              <Show when={entry().pinned}>
                                <Text class={`font-bold text-caption tabular-nums nine-chip-cut bg-gold border border-rim text-gold-fg transition duration-317 tracking-label uppercase py-0 px-1 shrink ${packClass("Chip", "frame")}`}>
                                  {t("meta.news.pinned")}
                                </Text>
                              </Show>
                              <Show when={entry().unread}>
                                <Text class={`font-bold text-caption tabular-nums nine-chip-cut bg-gold border border-rim text-gold-fg transition duration-317 tracking-label uppercase py-0 px-1 shrink ${packClass("Chip", "frame")}`}>
                                  {t("meta.news.unread")}
                                </Text>
                              </Show>
                            </View>
                            <View class={`tracking-xs text-fg-muted w-full`} role="heading" level={3}>
                              <View class={`flex-row items-center justify-center gap-1 w-full min-h-11 text-start px-0`} role="button" expanded={entry().id === props.selected} label={entry().title} disabled={(props.disabled ?? false)} onPress={(e) => props.onChoose?.(e, entry().id)}>
                                <Text class={`font-bold text-xl leading-6.5 text-fg w-full whitespace-normal`}>
                                  {entry().title}
                                </Text>
                              </View>
                            </View>
                            <Show when={entry().summary}>
                              <Text class={`text-caption text-fg-muted m-0`}>
                                {entry().summary}
                              </Text>
                            </Show>
                            <Show when={entry().id === props.selected}>
                              <Show when={entry().image}>
                                <Image class={`w-full max-h-60 aspect-2/1 object-cover nine-plate-cut`} src={entry().image} />
                              </Show>
                              <Show when={entry().body}>
                                <Text class={`text-fg-muted m-0`}>
                                  {entry().body}
                                </Text>
                              </Show>
                              <Show when={entry().actionLabel}>
                                <View class={`flex-row items-center justify-center gap-1 max-w-full min-h-11`} role="button" disabled={(props.disabled ?? false)} onPress={(e) => props.onOpen?.(e, entry().id)}>
                                  <Text class="whitespace-normal">{entry().actionLabel}</Text>
                                </View>
                              </Show>
                            </Show>
                          </View>
                        </View>
                        )}
                      </For>
                    </View>
                  </Scroll>
              }>
                <EmptyState title={t("meta.news.empty")} icon="flag" />
              </Show>
          }>
            <ErrorState body={props.error ?? ''} />
          </Show>
      }>
        <View class="flex-row items-center p-4 gap-2" role="status">
          <Spinner name={t("meta.loading")} />
          <Text class="shrink">{t("meta.loading")}</Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<NewsProps>> = {
  default: {"selected":"patch","entries":[{"id":"patch","title":"Expedition update 1.4","date":"16 Nov 2026","kind":"Patch notes","summary":"New destinations, equipment balance and accessibility improvements.","body":"New destinations\nExplore the northern coast with your party.\n\nAccessibility\nController navigation now remembers the last focused item.\n\nBalance\nHeavy weapons recover faster after a missed attack.","image":0,"unread":true,"pinned":true,"actionLabel":"View full release notes"},{"id":"event","title":"Double rewards weekend","date":"20–22 Nov","kind":"Event","summary":"Earn extra expedition rewards all weekend.","body":"All expedition rewards are doubled during the event.","image":0,"unread":false,"pinned":false,"actionLabel":""}]},
  empty: {},
  loading: {"loading":true},
  error: {"error":"Updates are temporarily unavailable. Try again when connected."},
  overflow: {"selected":"long","entries":[{"id":"long","title":"The northern expedition and the return of the Watcher of the Ninefold Gate","date":"16 November 2026 · 09:00 UTC","kind":"Community event and balance notes","summary":"An announcement with a long title and detailed changes that remain readable at phone widths.","body":"Expedition changes\nThe new route includes additional checkpoints and improved navigation markers.\n\nEquipment changes\nLong equipment descriptions wrap instead of hiding important information.","image":0,"unread":true,"pinned":true,"actionLabel":"Read the complete expedition update"}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Toggle an article by stable key","open":"Take the selected article's optional action by stable key","retry":"Retry loading articles"} as const;
