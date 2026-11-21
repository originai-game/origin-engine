import { For, Icon, Image, Scroll, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import { Spinner } from "../base/spinner";
import "./rhythm.en.ogt";

export interface RhythmSongSelectProps {
  /** The page's heading (a pack or folder name); empty takes the kit's (Songs). */
  title: string;
  /** The songs in the game's order, keyed rows of { id, title, artist, bpm, length, jacket, locked, fresh, cleared }: bpm and length as the game words them (128, 90–180; 2:14); jacket the cover art texture (og.load.texture, 0 none); locked the song cannot be played yet (its row is disabled); fresh newly added (New); cleared the player has cleared it (Cleared). */
  songs: ReadonlyArray<{ id: string; title: string; artist: string; bpm: string; length: string; jacket: number; locked: boolean; fresh: boolean; cleared: boolean }>;
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
      class={`flex-col gap-3 w-full h-full min-w-0 text-fg`}
      label={props.title !== "" ? props.title : t("rhythm.songs.title")}
      busy={props.status === "loading"}
      data-status={props.status}
    >
      <View class={`flex-row shrink w-full min-h-0`}>
        <Text class={`text-[2.857rem] compact:text-[1.714rem] leading-[1.1] font-bold tracking-[0.14em] uppercase min-w-0 whitespace-nowrap text-ellipsis overflow-hidden text-fg text-shadow-xs text-shadow-bg/55`}>
          {props.title !== "" ? props.title : t("rhythm.songs.title")}
        </Text>
      </View>
      <Show when={props.status === "loading"} fallback={
        <Show when={props.status === "error"} fallback={
          <Show when={(props.songs ?? []).length} fallback={
            <View role="status" class={`flex-col items-center justify-center grow min-h-0`}>
              <View class={`flex-col items-center gap-1 py-3 px-4 bg-[#1e2637e5] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none`}>
                <Text class={`text-[1.143rem] text-fg text-center`}>{t("rhythm.songs.empty")}</Text>
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
                        class={`transition duration-317 w-full min-w-0 min-h-[3.143rem] py-1 pl-1 pr-3 compact:data-selected:pl-1 rounded-tl-[11.43px] rounded-tr-none rounded-br-[11.43px] rounded-bl-none bg-linear-to-r from-[#0b0f1adb] to-[#0b0f1a80] shadow-[inset_0px_1px_0px_#ffffff24] hover:from-[#141c2eea] hover:to-[#0f15268f] hover:shadow-[inset_0px_0px_0px_1px_#ffffff38,inset_0px_1px_0px_#ffffff6b] disabled:opacity-62 disabled:from-[#0b0f1adb] disabled:to-[#0b0f1a80] disabled:shadow-[inset_0px_1px_0px_#ffffff24] data-selected:ml-[24px] compact:data-selected:ml-0 data-selected:py-2 data-selected:from-[#38e8ff3d] data-selected:to-[#38e8ff08] data-selected:shadow-[inset_3px_0px_0px_#38e8ff,inset_0px_1px_0px_#ffffff2e]`}
                      >
                        <View class={`flex-row items-center gap-4 w-full min-w-0`}>
                          <View class={`shrink size-[4.286rem] compact:size-[3.429rem] data-selected:size-[5.714rem] rounded-[0.8rem] bg-linear-to-b from-[#ffffffbf] via-[#ffffff38] to-[#ffffff24] p-[2px] data-selected:from-white data-selected:via-[#ffffff59] data-selected:to-[#38e8ff]`}>
                            <Show when={s().jacket} fallback={
                              <View class={`size-full rounded-[0.7rem] bg-[#12182a] flex-row items-center justify-center`}>
                                <Icon class={`size-[44%] text-[#38e8ff]`} name="music" />
                              </View>
                            }>
                              <Image class={`size-full rounded-[0.7rem] object-cover`} src={String(s().jacket)} />
                            </Show>
                          </View>
                          <View class={`flex-col items-start gap-0.5 min-w-0 grow`}>
                            <Text class={`max-w-full text-start text-[1.429rem] compact:text-[1.143rem] leading-[1.2] font-semibold whitespace-nowrap text-ellipsis overflow-hidden text-fg data-selected:text-[1.857rem] compact:data-selected:text-[1.143rem] data-selected:font-bold`}>
                              {s().title}
                            </Text>
                            <Text class={`max-w-full text-start text-[0.929rem] whitespace-nowrap text-ellipsis overflow-hidden text-fg-muted`}>
                              {s().artist}
                            </Text>
                            <Show when={s().id === props.selected || s().id === props.previewing || s().fresh || s().cleared || s().locked}>
                              <View class={`flex flex-row flex-wrap gap-0.5 mt-[3px]`}>
                                <Show when={s().id === props.selected}>
                                  <View class={`flex-row items-center shrink py-0 px-1 bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} data-tag="selected">
                                    <Text class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-[#38e8ff]`}>{t("rhythm.songs.selected")}</Text>
                                  </View>
                                </Show>
                                <Show when={s().id === props.previewing}>
                                  <View class={`flex-row items-center gap-1 shrink py-0 px-1 bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} data-tag="previewing">
                                    <View class={`shrink flex-row items-end gap-0.5 h-[11px]`}>
                                      <View class={`w-[3px] h-[60%] bg-[#38e8ff] animate-pulse`} />
                                      <View class={`w-[3px] h-full bg-[#38e8ff] animate-pulse`} />
                                      <View class={`w-[3px] h-[40%] bg-[#38e8ff] animate-pulse`} />
                                    </View>
                                    <Text class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-[#38e8ff]`}>{t("rhythm.songs.previewing")}</Text>
                                  </View>
                                </Show>
                                <Show when={s().fresh}>
                                  <View class={`flex-row items-center shrink py-0 px-1 bg-[#ff3d8a] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} data-tag="fresh">
                                    <Text class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-white`}>{t("rhythm.songs.fresh")}</Text>
                                  </View>
                                </Show>
                                <Show when={s().cleared}>
                                  <View class={`flex-row items-center shrink py-0 px-1 bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} data-tag="cleared">
                                    <Text class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-[#55e39a]`}>{t("rhythm.songs.cleared")}</Text>
                                  </View>
                                </Show>
                                <Show when={s().locked}>
                                  <View class={`flex-row items-center gap-1 shrink py-0 px-1 bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} data-tag="locked">
                                    <Icon class={`size-[1.143rem] text-fg-muted`} name="lock" />
                                    <Text class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-fg-muted`}>{t("rhythm.songs.locked")}</Text>
                                  </View>
                                </Show>
                              </View>
                            </Show>
                          </View>
                          <Text class={`shrink text-[1.714rem] compact:text-[1.143rem] leading-none font-bold tabular-nums whitespace-nowrap text-fg`}>
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
                  <Text class={`py-2 px-3 text-[1.143rem] text-fg text-center bg-[#060910e0]`}>{t("rhythm.songs.none")}</Text>
                </View>
              </Show>
              <For each={props.songs ?? []} key={(s) => s.id}>
                {(s) => (
                  <Show when={s().id === props.selected}>
                    <View
                      label={s().title}
                      data-previewing={s().id === props.previewing}
                      data-locked={s().locked}
                      class={`group/detail flex-[4] min-w-0 min-h-0 p-1 compact:flex-none compact:w-full compact:p-0`}
                    >
                      <View class={`flex-row items-start gap-4 w-full min-w-0 compact:items-center`}>
                        <View class={`shrink w-[42%] max-w-[21.429rem] compact:w-[6.571rem] compact:max-w-none aspect-square rounded-[1.2rem] bg-linear-to-b from-white to-[#38e8ff] shadow-[0px_0px_10px_#38e8ff] p-[3px] group-data-locked/detail:from-[#ffffffbf] group-data-locked/detail:via-[#ffffff38] group-data-locked/detail:to-[#ffffff24] group-data-locked/detail:shadow-none`}>
                          <View class={`relative size-full rounded-[1rem] bg-[#12182a]`}>
                            <Show when={s().jacket} fallback={
                              <View class={`absolute inset-0 flex-row items-center justify-center`}>
                                <Icon class={`size-[44%] text-[#38e8ff]`} name="music" />
                              </View>
                            }>
                              <Image class={`size-full rounded-[1rem] object-cover`} src={String(s().jacket)} />
                            </Show>
                            <View class={`absolute inset-0 rounded-[1rem] bg-bg/55 opacity-0 group-data-locked/detail:opacity-100`} />
                          </View>
                        </View>
                        <View class={`flex-col items-start gap-1 min-w-0 grow`}>
                          <Show when={s().fresh || s().locked}>
                            <View class={`flex flex-row flex-wrap gap-0.5`}>
                              <Show when={s().fresh}>
                                <View class={`flex-row items-center shrink py-0 px-1 bg-[#ff3d8a] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} data-tag="fresh">
                                  <Text class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-white`}>{t("rhythm.songs.fresh")}</Text>
                                </View>
                              </Show>
                              <Show when={s().locked}>
                                <View class={`flex-row items-center gap-1 shrink py-0 px-1 bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none`} data-tag="locked">
                                  <Icon class={`size-[1.143rem] text-fg-muted`} name="lock" />
                                  <Text class={`text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-fg-muted`}>{t("rhythm.songs.locked")}</Text>
                                </View>
                              </Show>
                            </View>
                          </Show>
                          <Text class={`w-full text-start text-[3.571rem] compact:text-[1.714rem] leading-[1.05] font-bold whitespace-nowrap text-ellipsis overflow-hidden text-fg text-shadow-xs text-shadow-bg/55`}>
                            {s().title}
                          </Text>
                          <Text class={`w-full text-start text-[1.143rem] compact:text-[0.929rem] whitespace-nowrap text-ellipsis overflow-hidden text-fg-muted`}>
                            {s().artist}
                          </Text>
                          <View class={`flex flex-row flex-wrap gap-2 py-1 items-stretch max-w-full compact:flex-nowrap compact:items-center compact:py-0`}>
                            <View class={`flex-col items-start gap-0 min-w-[5.714rem] compact:min-w-0 compact:min-h-0 compact:flex-row compact:items-baseline compact:gap-1 py-1 px-2 bg-[#1e2637e5] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none`}>
                              <Text class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ffffffc7] compact:hidden`}>{t("rhythm.songs.bpmLabel")}</Text>
                              <Text class={`text-[2.429rem] compact:text-[1.143rem] leading-[1.05] font-bold tabular-nums whitespace-nowrap text-fg`}>{s().bpm}</Text>
                            </View>
                            <View class={`flex-col items-start gap-0 min-w-[5.714rem] compact:min-w-0 compact:min-h-0 compact:flex-row compact:items-baseline compact:gap-1 py-1 px-2 bg-[#1e2637e5] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none`}>
                              <Text class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ffffffc7] compact:hidden`}>{t("rhythm.songs.lengthLabel")}</Text>
                              <Text class={`text-[2.429rem] compact:text-[1.143rem] leading-[1.05] font-bold tabular-nums whitespace-nowrap text-fg`}>{s().length}</Text>
                            </View>
                            <View
                              role="button"
                              label={s().id === props.previewing ? t("rhythm.songs.previewing") : t("rhythm.songs.preview")}
                              pressed={s().id === props.previewing}
                              disabled={s().locked}
                              onPress={(e) => props.onPreview?.(e, s().id)}
                              onAction={{ details: (e) => props.onPreview?.(e, s().id) }}
                              data-selected={s().id === props.previewing}
                              class={`min-h-[3.143rem] compact:min-h-[2.514rem] min-w-[8.571rem] compact:min-w-0 flex-row items-center justify-center gap-1 px-3 rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none bg-[#141b2de6] shadow-[inset_0px_0px_0px_1px_#38e8ff80,inset_0px_2px_0px_#38e8ff] hover:bg-[#1c2438ee] data-selected:from-[#dffbff] data-selected:to-[#38e8ff] data-selected:bg-linear-to-b data-selected:shadow-[0px_0px_18px_#38e8ff8c] disabled:opacity-50 disabled:bg-[#141b2de6] disabled:shadow-[inset_0px_0px_0px_1px_#38e8ff80]`}
                            >
                              <Text class={`shrink h-[1.75rem] min-w-[1.75rem] px-1 text-center rounded-[4.576px] bg-linear-to-b from-[#ffffff] to-[#c9d1e2] text-[#0b0f18] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] pointer-coarse:hidden`} glyph="details" />
                              <Show when={s().id === props.previewing}>
                                <View class={`shrink flex-row items-end gap-0.5 h-[11px] compact:hidden`}>
                                  <View class={`w-[3px] h-[60%] bg-[#38e8ff] animate-pulse`} />
                                  <View class={`w-[3px] h-full bg-[#38e8ff] animate-pulse`} />
                                  <View class={`w-[3px] h-[40%] bg-[#38e8ff] animate-pulse`} />
                                </View>
                              </Show>
                              <Text class={`text-[0.929rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-fg data-selected:text-[#0b0f18] disabled:text-fg`}>
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
            <View role="alert" data-status="error" class={`flex-col items-center justify-center grow min-h-0`}>
              <View class={`flex-col items-center gap-1 py-3 px-4 bg-[#1e2637e5] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none`}>
                <Text class={`text-[1.143rem] text-fg text-center`}>{t("rhythm.songs.error")}</Text>
                <View
                  role="button"
                  label={t("rhythm.songs.retry")}
                  onPress={(e) => props.onRetry?.(e)}
                  class={`min-h-[3.143rem] min-w-[8.571rem] flex-row items-center justify-center gap-1 px-3 rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none bg-[#38e8ff] shadow-[inset_0px_2px_0px_#ffffff59] hover:bg-[#6df0ff]`}
                >
                  <Text class={`text-[0.929rem] font-bold tracking-[0.14em] uppercase text-[#141a02]`}>{t("rhythm.songs.retry")}</Text>
                </View>
              </View>
            </View>
          </Show>
        }>
          <View role="status" class={`flex-col items-center justify-center grow min-h-0`}>
            <View class={`flex-col items-center gap-1 py-3 px-4 bg-[#1e2637e5] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none`}>
              <Spinner name={t("rhythm.songs.loading")} />
              <Text class={`text-[1.143rem] text-fg text-center`}>{t("rhythm.songs.loading")}</Text>
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
