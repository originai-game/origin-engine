import {createSignal, For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Button } from "../controls/button";
import "./settings.en.ogt";

export interface SettingsScreenProps {
  /** A line under the title (Some changes apply after a restart); empty: none. */
  note: string;
  /** The settings' pages as a keyed collection of { id, name }: a tab each. */
  pages: ReadonlyArray<{ id: string; name: string }>;
  /** The settings as a keyed collection of { id, page, name, description, kind: toggle|range|select|binding, value, min, max, step, options, action }, shown on their page's tab; each value is written back as the player changes it (two-way via onChange). */
  entries: ReadonlyArray<{ id: string; page: string; name: string; description: string; kind: string; value: boolean | number | string; min?: number; max?: number; step?: number; options?: ReadonlyArray<string>; action?: string }>;
  /** The id of the page shown (two-way via onPage: the game writes it back as the player picks a tab); empty: no tree of pages, the `tree` content alone. */
  page: string;
  /** Changes wait to be applied: Apply and Revert take presses. */
  pending?: boolean;
  /** Content the caller supplies (was the tree snippet). */
  tree?: (() => JSX.Element) | undefined;
  /** The page tab picked, with the tab's id; the game writes props.page back. */
  onPage?: (e: UiEvent, id: string) => void;
  /** An entry's value changed, with the entry's id and its new value; the game writes entries' value back. */
  onChange?: (e: UiEvent, id: string, value: boolean | number | string) => void;
  /** The key captured while a binding row listens, with the entry's id and the key; the game writes entries' value back. Escape cancels without calling this. */
  onRebind?: (e: UiEvent, id: string, key: string) => void;
  /** The apply event, with the row's key where it comes from a row. */
  onApply?: (e: UiEvent, key?: string) => void;
  /** The back event, with the row's key where it comes from a row. */
  onBack?: (e: UiEvent, key?: string) => void;
  /** The defaults event, with the row's key where it comes from a row. */
  onDefaults?: (e: UiEvent, key?: string) => void;
  /** The revert event, with the row's key where it comes from a row. */
  onRevert?: (e: UiEvent, key?: string) => void;
}

export function SettingsScreen(props: SettingsScreenProps) {
  const listening = createSignal("");
  const openChoice = createSignal("");
  return (
    <View class={`pt-8 pb-6 text-fg gap-y-4 px-12 overflow-hidden compact:pt-4 compact:pb-3 compact:gap-y-3 compact:px-4`}>
      <View class={`absolute bg-linear-to-b from-[#03060ee5] via-[#03060ed1] via-40% to-[#03060ef0] inset-0`} />
      <View class={`min-h-12 border-b border-rim-soft items-baseline gap-x-4 py-1 px-4 compact:flex-wrap compact:gap-y-1`}>
        <Button class={`text-fg/74 compact:-ml-2 shrink ${packClass("Button", "plate")}`} onPress={(e) => props.onBack?.(e)}>
          <View class="items-center gap-x-1">
            <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100 shrink" name="chevronLeft" />
            <Text class="shrink">{t("screen.common.back")}</Text>
          </View>
        </Button>
        <View class="grow shrink basis-0 items-baseline flex-wrap gap-x-4 gap-y-0.5 compact:grow compact:shrink-0 compact:basis-full compact:items-start compact:flex-col">
          <Text class={`font-display text-[2.857rem] font-bold leading-tight tracking-xs text-fg text-shadow-xs text-shadow-bg/60 m-0 compact:text-[1.857rem] shrink`}>
            {t("screen.settings.title")}
          </Text>
          <Show when={props.note !== ""}>
            <Text class="mr-0 ml-auto text-base text-[#ffb35c] line-clamp-2 my-0 compact:ml-0 compact:text-[0.929rem] shrink">
              {props.note}
            </Text>
          </Show>
        </View>
      </View>
      <View class="grow shrink basis-0 gap-y-4">
        <Show when={props.page}>
          <View class={`items-center gap-x-3 border-b border-white/14`} role="tablist">
            <Text class="flex-none pointer-coarse:hidden" glyph="tabPrev" />
            <Scroll class="grow shrink gap-x-6 py-0.5 px-4 compact:gap-x-4 compact:-mx-4">
              <For each={props.pages}>
                {(p) => (
                <View class={`text-fg-muted transition duration-317 data-selected:text-fg data-selected:bg-none data-selected:bg-raised-hover-lo hover:bg-none hover:bg-[#7379851d] compact:text-base`} role="tab" tabIndex={0} selected={props.page === p().id} label={p().name} onPress={(e) => props.onPage?.(e, p().id)}>
                  <Text>{p().name}</Text>
                </View>
                )}
              </For>
            </Scroll>
            <Text class="flex-none pointer-coarse:hidden" glyph="tabNext" />
          </View>
          <View class="grow shrink basis-0 grid grid-cols-[minmax(0,3fr)_minmax(0,2fr)] grid-rows-[minmax(0,1fr)] gap-x-12 compact:grid-cols-[minmax(0,1fr)]">
            <Scroll class="grow shrink basis-0 gap-y-0.5 p-0 m-0">
              <For each={props.entries}>
                {(entry) => {
                const value = entry().value;
                const num = typeof value === "number" ? value : undefined;
                const at = Math.max(0, Math.min(100, ((Math.min(Math.max(num ?? (entry().min ?? 0), entry().min ?? 0), entry().max ?? 100)) - (entry().min ?? 0)) / ((entry().max ?? 100) - (entry().min ?? 0) || 1)) * 100);
                /** The pointer's drag: the track's box rides in e.value ("x y width height"), pointerdown captures the moves. */
                const dragged = (e: UiEvent) => {
                  const [bx, , bw] = e.value.split(" ").map(Number);
                  const min = entry().min ?? 0;
                  const max = entry().max ?? 100;
                  const step = entry().step ?? 1;
                  const raw = min + Math.min(1, Math.max(0, (e.x - bx) / (bw || 1))) * (max - min);
                  props.onChange?.(e, entry().id, Math.min(max, Math.max(min, min + Math.round((raw - min) / step) * step)));
                };
                return (
                <Show when={entry().page === props.page}>
                  <View class={`group/row bg-linear-to-r from-[#151d2eb8] to-[#0b101c80] text-fg transition duration-317 hover:bg-transparent hover:bg-[#38e8ff3d] data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} label={entry().description}>
                    <Show when={entry().kind === "toggle"}>
                      <View class="group/switch min-h-16 text-[1.286rem] grow shrink text-fg/74 py-1 px-4 compact:text-[1.143rem] compact:gap-y-1 compact:py-2 compact:px-3 group-hover/row:text-fg" role="switch" tabIndex={0} checked={entry().value === true} label={entry().name} data-checked={entry().value === true} onPress={(e) => props.onChange?.(e, entry().id, !(entry().value === true))}>
                        <Text>{entry().name}</Text>
                        <View class="ml-auto relative h-4 w-7 rounded-capsule bg-tile shadow-[inset_0px_0px_0px_1px_#4e586c]">
                          <View class="absolute top-0.5 left-0.5 size-3 rounded-capsule bg-fg-muted transition-transform duration-200 ease-out group-data-checked/switch:translate-x-3 group-data-checked/switch:bg-accent" />
                        </View>
                      </View>
                    </Show>
                    <Show when={entry().kind === "range"}>
                      <View class="grid grid-cols-[minmax(182.9px,1fr)_minmax(0,365.7px)] items-center gap-x-6 min-h-16 text-[1.286rem] py-1 px-4 compact:grid-cols-[minmax(0,1fr)] compact:items-stretch compact:text-[1.143rem] compact:gap-y-1 compact:py-2 compact:px-3">
                        <Text class="grow shrink text-fg/74 m-0 group-hover/row:text-fg">
                          {entry().name}
                        </Text>
                        <View
                          class="relative flex-row items-center h-9"
                          role="slider"
                          tabIndex={0}
                          label={entry().name}
                          valueMin={entry().min ?? 0}
                          valueMax={entry().max ?? 100}
                          valueNow={num ?? (entry().min ?? 0)}
                          onKey={(e) => {
                            const now = num ?? (entry().min ?? 0);
                            const step = entry().step ?? 1;
                            if (e.key === "ArrowRight" || e.key === "ArrowUp") props.onChange?.(e, entry().id, Math.min(entry().max ?? 100, now + step));
                            else if (e.key === "ArrowLeft" || e.key === "ArrowDown") props.onChange?.(e, entry().id, Math.max(entry().min ?? 0, now - step));
                          }}
                        >
                          <View class="relative grow shrink basis-0 h-1 rounded-capsule bg-tile shadow-[0px_0px_0px_1px_#ffffff38,inset_0px_1px_0px_0px_#ffffff24]" onPointerDown={dragged} onPointerMove={dragged}>
                            <View class="absolute left-0 top-0 h-full rounded-capsule bg-accent" style={{ width: `${at}%` }} />
                            <View class="absolute top-0 size-2.5 -translate-y-1 rounded-capsule bg-accent shadow-panel" style={{ left: `calc(${at}% - 5px)` }} />
                          </View>
                        </View>
                      </View>
                    </Show>
                    <Show when={entry().kind === "select"}>
                      <View class="grid grid-cols-[minmax(182.9px,1fr)_minmax(0,365.7px)] items-center gap-x-6 min-h-16 text-[1.286rem] py-1 px-4 compact:grid-cols-[minmax(0,1fr)] compact:items-stretch compact:text-[1.143rem] compact:gap-y-1 compact:py-2 compact:px-3">
                        <Text class="grow shrink text-fg/74 m-0 group-hover/row:text-fg">
                          {entry().name}
                        </Text>
                        <View class="flex-col gap-1 min-w-0">
                          <View
                            class={`flex-row items-center justify-between gap-2 h-9 py-0 px-3 rounded-[4.576px] bg-tile shadow-[inset_0px_0px_0px_1px_#4e586c] data-open:shadow-[inset_0px_0px_0px_1px_#38e8ff]`}
                            role="combobox"
                            tabIndex={0}
                            label={entry().name}
                            expanded={openChoice() === entry().id}
                            valueText={String(entry().value)}
                            data-open={openChoice() === entry().id}
                            onKey={(e) => {
                              const options = entry().options ?? [];
                              const at = Math.max(0, options.indexOf(String(entry().value)));
                              const wrap = (next: number) => options[((next % options.length) + options.length) % options.length];
                              if (e.key === "ArrowRight" || e.key === "ArrowDown") props.onChange?.(e, entry().id, wrap(at + 1));
                              else if (e.key === "ArrowLeft" || e.key === "ArrowUp") props.onChange?.(e, entry().id, wrap(at - 1));
                            }}
                            onPress={(e) => openChoice.set(openChoice() === entry().id ? "" : entry().id)}
                          >
                            <Text class="text-start min-w-0 truncate">{String(entry().value)}</Text>
                            <Icon class={`size-4 shrink ${openChoice() === entry().id ? "rotate-180" : ""}`} name="chevronDown" />
                          </View>
                          <Show when={openChoice() === entry().id}>
                            <View class={`flex-col rounded-[4.576px] bg-[#0b101ce6] shadow-[inset_0px_0px_0px_1px_#4e586c] overflow-hidden`} role="listbox" label={entry().name} onAction={{ back: (e) => openChoice.set("") }}>
                              <For each={entry().options ?? []}>
                                {(opt) => (
                                  <View class="flex-row items-center justify-start gap-2 min-h-8 py-0 px-3 text-start text-sm font-semibold text-fg bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_1px_0px_0px_#ffffff1f] transition duration-317 hover:bg-none hover:bg-raised-hover-lo data-selected:text-accent data-selected:bg-[#38e8ff26] data-selected:shadow-[inset_0px_0px_0px_1px_#38e8ff]" role="option" tabIndex={0} selected={opt() === String(entry().value)} label={opt()} onPress={(e) => { props.onChange?.(e, entry().id, opt()); openChoice.set(""); }}>
                                    <Text class="whitespace-nowrap">{opt()}</Text>
                                  </View>
                                )}
                              </For>
                            </View>
                          </Show>
                        </View>
                      </View>
                    </Show>
                    <Show when={entry().kind === "binding"}>
                      <View class={`grid grid-cols-[minmax(182.9px,1fr)_minmax(0,365.7px)] items-center gap-x-6 min-h-16 text-[1.286rem] py-1 px-4 compact:grid-cols-[minmax(0,1fr)_auto] compact:items-center compact:text-[1.143rem] compact:gap-y-1 compact:py-2 compact:px-3 group-hover/row:text-fg data-listening:text-accent data-listening:bg-[#38e8ff14]`} role="button" tabIndex={0} label={entry().name} data-listening={listening() === entry().id} onPress={() => listening.set(listening() === entry().id ? "" : entry().id)} onKey={(e) => {
                        if (listening() !== entry().id) return;
                        if (e.key === "Escape") {
                          listening.set("");
                          return;
                        }
                        props.onRebind?.(e, entry().id, e.key);
                        listening.set("");
                      }}>
                        <Text class="grow shrink text-fg/74 m-0 group-hover/row:text-fg">
                          {entry().name}
                        </Text>
                        <Show when={listening() === entry().id} fallback={<Text class="justify-self-end" glyph={entry().action} />}>
                          <Text class="justify-self-end text-accent whitespace-nowrap">{t("screen.settings.pressAKey")}</Text>
                        </Show>
                      </View>
                    </Show>
                    <Show when={entry().description}>
                      <Text class={`hidden m-0`}>{entry().description}</Text>
                    </Show>
                  </View>
                </Show>
                );
                }}
              </For>
            </Scroll>
          </View>
          <View class="flex-wrap items-center justify-end gap-y-2 gap-x-3 compact:grid compact:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
            <Button class={`mr-auto compact:mr-0 compact:col-span-full compact:justify-self-start compact:-ml-2 shrink ${packClass("Button", "plate")}`} onPress={(e) => props.onDefaults?.(e)}>
              <Text>{t("screen.settings.defaults")}</Text>
            </Button>
            <Button class={`text-fg bg-[#171f30d6] nine-plate-cut min-w-[11.429rem] h-9 disabled:border-transparent active:border-transparent hover:bg-[#171f30d6] hover:bg-transparent hover:border-transparent compact:min-w-0 data-pressed:hover:border-transparent shrink ${packClass("Button", "plate")}`} onPress={(e) => props.onRevert?.(e)} disabled={!props.pending}>
              <Text>{t("screen.settings.revert")}</Text>
            </Button>
            <Button class={`text-accent nine-plate-cut min-w-[11.429rem] h-9 compact:min-w-0 shrink ${packClass("Button", "plate", "primary")}`} disabled={!props.pending} onPress={(e) => props.onApply?.(e)}>
              <Text>{t("screen.settings.apply")}</Text>
            </Button>
          </View>
        </Show>
        {props.tree?.()}
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<SettingsScreenProps>> = {
  default: {"note":"Some changes apply after a restart.","pages":[{"id":"general","name":"General"}],"page":"general","pending":true,"entries":[{"id":"subtitles","page":"general","name":"Subtitles","description":"Show dialogue captions.","kind":"toggle","value":true},{"id":"volume","page":"general","name":"Master volume","description":"Adjust all game audio.","kind":"range","value":75,"min":0,"max":100,"step":5},{"id":"quality","page":"general","name":"Quality","description":"Choose the rendering quality.","kind":"select","value":"High","options":["Low","High"]},{"id":"interact","page":"general","name":"Interact","description":"Choose the interaction key.","kind":"binding","value":"E","action":"interact"}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Select","back":"Back","tabNext":"Next page","tabPrev":"Previous page"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"settings","layer":"menu","focus":"Tree","under":"dim","input":"menu","pause":false,"capture":true,"remember":true,"back":"back"} as const;
