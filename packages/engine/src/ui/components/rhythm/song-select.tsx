import type { Texture } from "@origingame/engine";
import { For, Icon, Image, Scroll, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import { Spinner } from "../base/spinner";
import "./rhythm.en.ogt";

export interface RhythmSongSelectProps {
  /** The page's heading (a pack or folder name); empty takes the kit's (Songs). */
  title: string;
  /** The songs in the game's order, keyed rows of { id, title, artist, bpm, length, jacket, locked, fresh, cleared }: bpm and length as the game words them (128, 90–180; 2:14); jacket the cover art texture (og.load.texture, 0 none); locked the song cannot be played yet (its row is disabled); fresh newly added (New); cleared the player has cleared it (Cleared). */
  songs: ReadonlyArray<{ id: string; title: string; artist: string; bpm: string; length: string; jacket: Texture | 0; locked: boolean; fresh: boolean; cleared: boolean }>;
  /** The song in focus as the game holds it (its row selected, its details shown); empty is none. */
  selected: string;
  /** The song whose preview the game is playing (Previewing); empty is none. */
  previewing: string;
  /** The list's state: empty when the songs are there, loading while the game fetches them, error when it could not. */
  status: string;
  /** Choose the song; the row's key. A locked song's row is disabled and reports nothing. */
  onChoose?: (e: UiEvent, key: string) => void;
  /** Preview; the details action (X, pad Y) on a focused row, with that row's key, and the Preview button with the selected song's key. */
  onPreview?: (e: UiEvent, key: string) => void;
  /** Retry; offered only while status is error. */
  onRetry?: (e: UiEvent) => void;
}

/**
 * Choosing a song: the game's list (jacket, title, artist, BPM, with New,
 * Cleared, Locked, Selected and Previewing named in words) beside the
 * selected song's details. Each song is a cut plate of dark glass; the
 * selected one is washed in the HUD's accent and stepped in. A row reports
 * choose with its key; Preview (the button, or the details action on a row)
 * reports preview with its key. A locked song is a disabled row and reports
 * nothing. The game owns unlocking, the selection, the preview audio and
 * the loading; status shows its loading and error states, and the error
 * state offers Retry (retry).
 */
export function RhythmSongSelect(props: RhythmSongSelectProps) {
  return (
    <View
      class={`flex-col gap-3 w-full h-full min-w-0 p-4 text-fg bg-glass nine-panel-cut ${packClass("Panel", "frame")}`}
      label={props.title !== "" ? props.title : t("rhythm.songs.title")}
      busy={props.status === "loading"}
      data-status-loading={props.status === "loading"}
      data-status-error={props.status === "error"}
    >
      <View class={`flex-row shrink w-full min-h-0`}>
        <Text class={`text-display-lg compact:text-title leading-tight font-bold tracking-label uppercase min-w-0 whitespace-nowrap text-ellipsis overflow-hidden text-fg text-stroke-3 text-stroke-bg`}>
          {props.title !== "" ? props.title : t("rhythm.songs.title")}
        </Text>
      </View>
      <Show when={props.status === "loading"} fallback={
        <Show when={props.status === "error"} fallback={
          <Show when={(props.songs ?? []).length} fallback={
            <View role="status" class={`flex-col items-center justify-center grow min-h-0`}>
              <View class={`flex-col items-center gap-1 py-3 px-4 bg-plate-hi nine-plate-cut text-fg ${packClass("Plate", "frame")}`}>
                <Text class={`text-base text-center`}>{t("rhythm.songs.empty")}</Text>
              </View>
            </View>
          }>
            <View class={`flex-row gap-4 grow min-h-0 items-stretch compact:flex-col`}>
              <Scroll class={`flex-[5] min-w-0 min-h-0 compact:flex-1 compact:w-full`}>
                <View role="listbox" label={t("rhythm.songs.list")} class={`flex-col gap-1 w-full py-1 pl-0.5 pr-2 compact:pr-0.5`}>
                  <For each={props.songs ?? []} key={(s) => s.id}>
                    {(s) => (
                      <View
                        role="option"
                        label={s().title}
                        description={s().artist}
                        selected={s().id === props.selected}
                        disabled={s().locked}
                        onPress={(e) => props.onChoose?.(e, s().id)}
                        onAction={{ details: (e) => props.onPreview?.(e, s().id) }}
                        data-fresh={s().fresh}
                        data-cleared={s().cleared}
                        data-previewing={s().id === props.previewing}
                        data-locked={s().locked}
                        data-selected={s().id === props.selected}
                        class={`group/song transition duration-control self-stretch min-w-0 min-h-11 py-1 pl-1 pr-3 compact:data-selected:pl-1 nine-plate-cut bg-tile hover:bg-surface disabled:opacity-muted disabled:bg-tile data-selected:ml-6 compact:data-selected:ml-0 data-selected:py-2 data-selected:bg-surface-hover data-selected:inset-ring-regular data-selected:inset-ring-hud-accent`}
                      >
                        <View class={`flex-row items-center gap-4 w-full min-w-0`}>
                          <View class={`shrink-0 size-15 compact:size-12 group-data-selected/song:size-20 rounded-panel bg-linear-to-b from-rim-strong via-rim to-rim-soft p-0.5 group-data-selected/song:from-rim-strong group-data-selected/song:via-rim group-data-selected/song:to-hud-accent`}>
                            <Show when={s().jacket} fallback={
                              <View class={`size-full rounded-card bg-glass flex-row items-center justify-center`}>
                                <Icon class={`size-11/25 text-hud-accent-ink`} name="music" />
                              </View>
                            }>
                              <Image class={`size-full rounded-card object-cover`} src={s().jacket} />
                            </Show>
                          </View>
                          <View class={`flex-col items-start gap-0.5 min-w-0 grow`}>
                            <Text class={`max-w-full text-start text-xl compact:text-base leading-tight font-semibold whitespace-nowrap text-ellipsis overflow-hidden text-fg group-data-selected/song:text-3xl compact:group-data-selected/song:text-base group-data-selected/song:font-bold`}>
                              {s().title}
                            </Text>
                            <Text class={`max-w-full text-start text-caption whitespace-nowrap text-ellipsis overflow-hidden text-fg-muted`}>
                              {s().artist}
                            </Text>
                            <Show when={s().id === props.selected || s().id === props.previewing || s().fresh || s().cleared || s().locked}>
                              <View class={`flex flex-row flex-wrap gap-0.5 mt-0.75`}>
                                <Show when={s().id === props.selected}>
                                  <View class={`flex-row items-center shrink py-0 px-1 bg-well nine-chip-cut text-hud-accent-ink ${packClass("Chip", "frame")}`} data-tag-selected={true}>
                                    <Text class={`text-xs font-bold tracking-label uppercase whitespace-nowrap`}>{t("rhythm.songs.selected")}</Text>
                                  </View>
                                </Show>
                                <Show when={s().id === props.previewing}>
                                  <View class={`flex-row items-center gap-1 shrink py-0 px-1 bg-well nine-chip-cut text-hud-accent-ink ${packClass("Chip", "frame")}`} data-tag-previewing={true}>
                                    <View class={`shrink flex-row items-end gap-0.5 h-2.75`}>
                                      <View class={`w-0.75 h-3/5 bg-hud-accent animate-pulse`} />
                                      <View class={`w-0.75 h-full bg-hud-accent animate-pulse`} />
                                      <View class={`w-0.75 h-2/5 bg-hud-accent animate-pulse`} />
                                    </View>
                                    <Text class={`text-xs font-bold tracking-label uppercase whitespace-nowrap`}>{t("rhythm.songs.previewing")}</Text>
                                  </View>
                                </Show>
                                <Show when={s().fresh}>
                                  <View class={`flex-row items-center shrink py-0 px-1 bg-hud-accent nine-chip-cut text-hud-accent-fg`} data-tag-fresh={true}>
                                    <Text class={`text-xs font-bold tracking-label uppercase whitespace-nowrap`}>{t("rhythm.songs.fresh")}</Text>
                                  </View>
                                </Show>
                                <Show when={s().cleared}>
                                  <View class={`flex-row items-center shrink py-0 px-1 bg-well nine-chip-cut text-boon-ink ${packClass("Chip", "frame")}`} data-tag-cleared={true}>
                                    <Text class={`text-xs font-bold tracking-label uppercase whitespace-nowrap`}>{t("rhythm.songs.cleared")}</Text>
                                  </View>
                                </Show>
                                <Show when={s().locked}>
                                  <View class={`flex-row items-center gap-1 shrink py-0 px-1 bg-well nine-chip-cut text-fg ${packClass("Chip", "frame")}`} data-tag-locked={true}>
                                    <Icon class={`size-4 opacity-quiet`} name="lock" />
                                    <Text class={`text-xs font-bold tracking-label uppercase whitespace-nowrap opacity-quiet`}>{t("rhythm.songs.locked")}</Text>
                                  </View>
                                </Show>
                              </View>
                            </Show>
                          </View>
                          <Text class={`shrink text-title compact:text-base leading-none font-bold tabular-nums whitespace-nowrap text-fg`}>
                            {t("rhythm.songs.bpm", { bpm: s().bpm })}
                          </Text>
                        </View>
                      </View>
                    )}
                  </For>
                </View>
              </Scroll>
              <Show when={props.selected === ""}>
                <View label={t("rhythm.songs.none")} class={`flex-col items-center justify-center grow min-h-0 min-w-0`}>
                  <Text class={`py-2 px-3 text-base text-fg text-center bg-well`}>{t("rhythm.songs.none")}</Text>
                </View>
              </Show>
              <For each={props.songs ?? []} key={(s) => s.id}>
                {(s) => (
                  <Show when={s().id === props.selected}>
                    <View
                      label={s().title}
                      data-previewing={s().id === props.previewing}
                      data-locked={s().locked}
                      class={`group/detail flex-[4] min-w-0 min-h-0 p-1 compact:flex-none compact:w-full compact:p-0 ${packClass("Panel", "frame")}`}
                    >
                      <View class={`flex-row items-start gap-4 w-full min-w-0 compact:items-center`}>
                        <View class={`shrink-0 w-21/50 max-w-75 compact:w-23 compact:max-w-none aspect-square rounded-sheet bg-linear-to-b from-sheen to-hud-accent shadow-glow p-0.75 group-data-locked/detail:from-rim-strong group-data-locked/detail:via-rim group-data-locked/detail:to-rim-soft group-data-locked/detail:shadow-none`}>
                          <View class={`relative size-full rounded-panel bg-glass`}>
                            <Show when={s().jacket} fallback={
                              <View class={`absolute inset-0 flex-row items-center justify-center`}>
                                <Icon class={`size-11/25 text-hud-accent-ink`} name="music" />
                              </View>
                            }>
                              <Image class={`size-full rounded-panel object-cover`} src={s().jacket} />
                            </Show>
                            <View class={`absolute inset-0 rounded-panel bg-shade-deep opacity-0 group-data-locked/detail:opacity-100`} />
                          </View>
                        </View>
                        <View class={`flex-col items-start gap-1 min-w-0 grow`}>
                          <Show when={s().fresh || s().locked}>
                            <View class={`flex flex-row flex-wrap gap-0.5`}>
                              <Show when={s().fresh}>
                                <View class={`flex-row items-center shrink py-0 px-1 bg-hud-accent nine-chip-cut text-hud-accent-fg`} data-tag-fresh={true}>
                                  <Text class={`text-xs font-bold tracking-label uppercase whitespace-nowrap`}>{t("rhythm.songs.fresh")}</Text>
                                </View>
                              </Show>
                              <Show when={s().locked}>
                                <View class={`flex-row items-center gap-1 shrink py-0 px-1 bg-well nine-chip-cut text-fg ${packClass("Chip", "frame")}`} data-tag-locked={true}>
                                  <Icon class={`size-4 opacity-quiet`} name="lock" />
                                  <Text class={`text-xs font-bold tracking-label uppercase whitespace-nowrap opacity-quiet`}>{t("rhythm.songs.locked")}</Text>
                                </View>
                              </Show>
                            </View>
                          </Show>
                          <Text class={`w-full text-start text-display-xl compact:text-title leading-tight font-bold line-clamp-2 text-fg text-stroke-3 text-stroke-bg`}>
                            {s().title}
                          </Text>
                          <Text class={`w-full text-start text-base compact:text-caption whitespace-nowrap text-ellipsis overflow-hidden text-fg-muted`}>
                            {s().artist}
                          </Text>
                          <View class={`flex flex-row flex-wrap gap-2 py-1 items-stretch max-w-full compact:flex-nowrap compact:items-center compact:py-0`}>
                            <View class={`flex-col items-start gap-0 min-w-20 compact:min-w-0 compact:min-h-0 compact:flex-row compact:items-baseline compact:gap-1 py-1 px-2 bg-plate-hi nine-plate-cut text-fg ${packClass("Plate", "frame")}`}>
                              <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap opacity-quiet compact:hidden`}>{t("rhythm.songs.bpmLabel")}</Text>
                              <Text class={`text-display-md compact:text-base leading-flat font-bold tabular-nums whitespace-nowrap`}>{s().bpm}</Text>
                            </View>
                            <View class={`flex-col items-start gap-0 min-w-20 compact:min-w-0 compact:min-h-0 compact:flex-row compact:items-baseline compact:gap-1 py-1 px-2 bg-plate-hi nine-plate-cut text-fg ${packClass("Plate", "frame")}`}>
                              <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap opacity-quiet compact:hidden`}>{t("rhythm.songs.lengthLabel")}</Text>
                              <Text class={`text-display-md compact:text-base leading-flat font-bold tabular-nums whitespace-nowrap`}>{s().length}</Text>
                            </View>
                            <View
                              role="button"
                              label={s().id === props.previewing ? t("rhythm.songs.previewing") : t("rhythm.songs.preview")}
                              pressed={s().id === props.previewing}
                              disabled={s().locked}
                              onPress={(e) => props.onPreview?.(e, s().id)}
                              onAction={{ details: (e) => props.onPreview?.(e, s().id) }}
                              data-pressed={s().id === props.previewing}
                              class={`min-h-11 compact:min-h-8.75 min-w-30 compact:min-w-0 flex-row items-center justify-center gap-1 px-3 nine-plate-cut disabled:opacity-dim ${packClass("Button", "plate")} ${s().id === props.previewing ? packClass("Button", "plate", "primary") : "bg-layer-3 hover:bg-layer-4"}`}
                            >
                              <Text class={`shrink-0 pointer-coarse:hidden text-title leading-none`} glyph="details" />
                              <Show when={s().id === props.previewing}>
                                <View class={`shrink flex-row items-end gap-0.5 h-2.75 compact:hidden`}>
                                  <View class={`w-0.75 h-3/5 bg-current animate-pulse`} />
                                  <View class={`w-0.75 h-full bg-current animate-pulse`} />
                                  <View class={`w-0.75 h-2/5 bg-current animate-pulse`} />
                                </View>
                              </Show>
                              <Text class={`shrink-0 text-caption font-bold tracking-label uppercase whitespace-nowrap`}>
                                {s().id === props.previewing ? t("rhythm.songs.previewing") : t("rhythm.songs.preview")}
                              </Text>
                            </View>
                          </View>
                        </View>
                      </View>
                    </View>
                  </Show>
                )}
              </For>
            </View>
          </Show>
          }>
            <View role="alert" data-status-error={true} class={`flex-col items-center justify-center grow min-h-0`}>
              <View class={`flex-col items-center gap-1 py-3 px-4 bg-plate-hi nine-plate-cut text-fg ${packClass("Plate", "frame")}`}>
                <Text class={`text-base text-center`}>{t("rhythm.songs.error")}</Text>
                <View
                  role="button"
                  label={t("rhythm.songs.retry")}
                  onPress={(e) => props.onRetry?.(e)}
                  class={`min-h-11 min-w-30 flex-row items-center justify-center gap-1 px-3 bg-hud-accent nine-plate-cut hover:bg-hud-accent text-hud-accent-fg ${packClass("Button", "plate", "primary")}`}
                >
                  <Text class={`text-caption font-bold tracking-label uppercase`}>{t("rhythm.songs.retry")}</Text>
                </View>
              </View>
            </View>
          </Show>
        }>
          <View role="status" class={`flex-col items-center justify-center grow min-h-0`}>
            <View class={`flex-col items-center gap-1 py-3 px-4 bg-plate-hi nine-plate-cut text-fg ${packClass("Plate", "frame")}`}>
              <Spinner name={t("rhythm.songs.loading")} />
              <Text class={`text-base text-center`}>{t("rhythm.songs.loading")}</Text>
            </View>
          </View>
        </Show>
      </View>
  );
}

export const fixtures: Record<string, Partial<RhythmSongSelectProps>> = {
  default: {"title":"","selected":"skyline","previewing":"","status":"","songs":[{"id":"skyline","title":"Skyline Drive","artist":"Neon Harbor","bpm":"128","length":"2:14","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"ember","title":"Ember Waltz","artist":"Lumen & Vale","bpm":"96","length":"3:02","jacket":0,"locked":false,"fresh":true,"cleared":false},{"id":"static","title":"Static Bloom","artist":"Kiri","bpm":"174","length":"1:58","jacket":0,"locked":false,"fresh":false,"cleared":false},{"id":"vault","title":"Vault of Echoes","artist":"Orchid Unit","bpm":"90–180","length":"2:47","jacket":0,"locked":true,"fresh":false,"cleared":false}]},
  selected: {"title":"","selected":"ember","previewing":"ember","status":"","songs":[{"id":"skyline","title":"Skyline Drive","artist":"Neon Harbor","bpm":"128","length":"2:14","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"ember","title":"Ember Waltz","artist":"Lumen & Vale","bpm":"96","length":"3:02","jacket":0,"locked":false,"fresh":true,"cleared":false},{"id":"static","title":"Static Bloom","artist":"Kiri","bpm":"174","length":"1:58","jacket":0,"locked":false,"fresh":false,"cleared":false}]},
  disabled: {"title":"","selected":"vault","previewing":"","status":"","songs":[{"id":"skyline","title":"Skyline Drive","artist":"Neon Harbor","bpm":"128","length":"2:14","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"vault","title":"Vault of Echoes","artist":"Orchid Unit","bpm":"90–180","length":"2:47","jacket":0,"locked":true,"fresh":false,"cleared":false}]},
  empty: {"title":"","selected":"","previewing":"","status":"","songs":[]},
  loading: {"title":"","selected":"","previewing":"","status":"loading","songs":[]},
  error: {"title":"","selected":"","previewing":"","status":"error","songs":[]},
  overflow: {"title":"Extended Festival Collection · Volume Twelve · Encore Edition","selected":"s7","previewing":"s2","status":"","songs":[{"id":"s1","title":"Skyline Drive","artist":"Neon Harbor","bpm":"128","length":"2:14","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"s2","title":"Ember Waltz","artist":"Lumen & Vale","bpm":"96","length":"3:02","jacket":0,"locked":false,"fresh":true,"cleared":false},{"id":"s3","title":"Static Bloom","artist":"Kiri","bpm":"174","length":"1:58","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"s4","title":"Vault of Echoes","artist":"Orchid Unit","bpm":"90–180","length":"2:47","jacket":0,"locked":true,"fresh":false,"cleared":false},{"id":"s5","title":"Paper Lanterns","artist":"Mira Okonkwo-Lindqvist","bpm":"112","length":"2:31","jacket":0,"locked":false,"fresh":false,"cleared":false},{"id":"s6","title":"Glasswing","artist":"Teleport Choir","bpm":"200","length":"2:05","jacket":0,"locked":false,"fresh":true,"cleared":false},{"id":"s7","title":"The Unbearably Long Extended Twelve-Minute Midnight Orchestral Remix of a Song","artist":"The Philharmonic Synthesizer Ensemble of the Northern Coast featuring Everyone","bpm":"60–240","length":"12:48","jacket":0,"locked":false,"fresh":true,"cleared":true},{"id":"s8","title":"Rust Belt","artist":"Hollow Pines","bpm":"140","length":"2:22","jacket":0,"locked":true,"fresh":false,"cleared":false},{"id":"s9","title":"Moth Light","artist":"Sela","bpm":"88","length":"3:40","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"s10","title":"Overclock","artist":"Byte Parade","bpm":"185","length":"1:49","jacket":0,"locked":false,"fresh":false,"cleared":false},{"id":"s11","title":"Tidewater","artist":"Juniper","bpm":"104","length":"2:58","jacket":0,"locked":false,"fresh":false,"cleared":false},{"id":"s12","title":"Last Train","artist":"Neon Harbor","bpm":"150","length":"2:36","jacket":0,"locked":true,"fresh":false,"cleared":false}]},
};
