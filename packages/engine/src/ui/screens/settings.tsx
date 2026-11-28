import {createSignal, For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Backdrop } from "../controls/backdrop";
import { Button } from "../controls/button";
import { HintBar } from "../controls/hint-bar";
import { InputGlyph } from "../controls/input-glyph";
import { Select } from "../controls/select";
import { Slider } from "../controls/slider";
import { Switch } from "../controls/switch";
import { Tab, Tabs } from "../controls/tabs";
import "./settings.en.strings.ts";

export interface SettingsScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** A line under the title (Some changes apply after a restart); empty: none. */
  note: string;
  /** The settings' pages as a keyed collection of { id, name, icon? }: a tab each, with the icon (an icon set's name) before its name. */
  pages: ReadonlyArray<{ id: string; name: string; icon?: string }>;
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
  onApply?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The back event, with the row's key where it comes from a row. */
  onBack?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The defaults event, with the row's key where it comes from a row. */
  onDefaults?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The revert event, with the row's key where it comes from a row. */
  onRevert?: ((e: UiEvent, key?: string) => void) | undefined;
}

/**
 * Settings: a tab per page and its entries (toggles, ranges, choices and
 * key bindings), Apply and Revert while changes wait, and Defaults. The
 * game owns every value and writes back what the events report.
 */
export function SettingsScreen(props: SettingsScreenProps) {
  const listening = createSignal("");
  const openChoice = createSignal("");
  const activeEntryId = createSignal("");
  const stepPage = (e: UiEvent, delta: number) => {
    const pages = props.pages;
    if (!pages.length) return;
    const i = pages.findIndex((p) => p.id === props.page);
    const next = pages[((i < 0 ? 0 : i) + delta + pages.length) % pages.length];
    if (next) props.onPage?.(e, next.id);
  };
  const pageEntries = () => props.entries.filter((e) => e.page === props.page);
  const inspected = () => {
    const list = pageEntries();
    return list.find((e) => e.id === activeEntryId()) ?? list.find((e) => e.description) ?? list[0];
  };
  return (
    <View
      class={`absolute inset-0 items-center justify-center pt-[calc(env(hud-inset-top)+2rem)] pb-[calc(env(hud-inset-bottom)+4.5rem)] pl-[calc(env(hud-inset-left)+3rem)] pr-[calc(env(hud-inset-right)+3rem)] text-fg overflow-hidden portrait:justify-start portrait:pl-[calc(env(hud-inset-left)+1.5rem)] portrait:pr-[calc(env(hud-inset-right)+1.5rem)] compact:pt-[calc(env(hud-inset-top)+1rem)] compact:pb-[calc(env(hud-inset-bottom)+3rem)] compact:pl-[calc(env(hud-inset-left)+1rem)] compact:pr-[calc(env(hud-inset-right)+1rem)] ${packClass("Screen", "frame")} ${props.class ?? ""}`}
      onAction={{
        back: (e) => {
          if (listening()) {
            listening.set("");
            return;
          }
          if (openChoice()) {
            openChoice.set("");
            return;
          }
          props.onBack?.(e);
        },
        tabPrev: (e) => stepPage(e, -1),
        tabNext: (e) => stepPage(e, 1),
      }}
    >
      <Backdrop art={0} over veil="full" />
      <View class="flex-col gap-y-4 w-full max-w-240 min-h-0 max-h-full portrait:grow compact:grow compact:gap-y-3">
        <View class="flex-row min-h-12 items-end gap-x-5 compact:flex-wrap compact:gap-y-1">
          <Show when={props.onBack}>
            <Button size="sm" class="shrink self-center" autofocus onPress={(e) => props.onBack?.(e)}>
              <View class="flex-row items-center gap-x-1">
                <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100 shrink" name="chevronLeft" />
                <Text class="shrink">{t("screen.common.back")}</Text>
              </View>
            </Button>
          </Show>
          <Text class={`m-0 shrink ${packClass("Screen", "heading")} compact:text-display-md`}>
            {t("screen.settings.title")}
          </Text>
        </View>
        <View class={`shrink min-h-0 flex-col gap-y-4 px-7 pt-5 pb-6 compact:px-4 compact:pt-4 compact:pb-4 ${packClass("Panel", "frame")}`}>
          <Show when={props.page}>
            <View class="flex-row items-center gap-x-3">
              <InputGlyph class="flex-none" action="tabPrev" size="lg" />
              <Scroll class="grow shrink compact:-mx-2">
                <Tabs class="border-b-0 gap-x-2 px-1" label={t("screen.settings.title")}>
                  <For each={props.pages}>
                    {(p) => (
                      <Tab class="shrink-0 gap-x-2" selected={props.page === p().id} onPress={(e) => props.onPage?.(e, p().id)}>
                        <Show when={p().icon}>
                          <Icon class="size-5" name={p().icon!} />
                        </Show>
                        <Text class="whitespace-nowrap text-base">{p().name}</Text>
                      </Tab>
                    )}
                  </For>
                </Tabs>
              </Scroll>
              <InputGlyph class="flex-none" action="tabNext" size="lg" />
            </View>
            <Scroll class={`shrink min-h-0 gap-y-1.5 p-0 m-0 ${packClass("List", "cascade")}`}>
              <For each={props.entries} key={(entry) => entry.id}>
                {(entry) => (
                  <Show when={entry().page === props.page}>
                    <View
                      class={`group/row text-fg transition duration-control data-selected:text-fg disabled:text-fg-subtle ${packClass("Row", "frame")} data-selected:${packClass("Row", "frame", "selected")} ${packClass("List", "item")}`}
                      label={entry().description}
                      data-selected={inspected()?.id === entry().id}
                      onFocus={() => activeEntryId.set(entry().id)}
                      onPointerDown={() => activeEntryId.set(entry().id)}
                    >
                      <Show when={entry().kind === "toggle"}>
                        <Switch class="min-h-14 text-lg py-2 px-5 compact:text-base compact:px-3" checked={entry().value === true} onChange={(e, on) => { activeEntryId.set(entry().id); props.onChange?.(e, entry().id, on); }}>
                          <Text class="grow shrink font-semibold">{entry().name}</Text>
                        </Switch>
                      </Show>
                      <Show when={entry().kind === "range"}>
                        <View class="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] items-center gap-x-6 min-h-14 py-2 px-5 text-lg compact:grid-cols-1 compact:items-stretch compact:gap-y-1 compact:px-3 compact:text-base">
                          <Text class="grow shrink font-semibold m-0">{entry().name}</Text>
                          <View class="flex-row items-center gap-x-4">
                            <Slider
                              class="grow shrink"
                              value={typeof entry().value === "number" ? (entry().value as number) : (entry().min ?? 0)}
                              min={entry().min}
                              max={entry().max}
                              step={entry().step}
                              label={entry().name}
                              onChange={(e, v) => { activeEntryId.set(entry().id); props.onChange?.(e, entry().id, v); }}
                            />
                            <Text class="m-0 w-12 text-end tabular-nums font-bold">{String(entry().value)}</Text>
                          </View>
                        </View>
                      </Show>
                      <Show when={entry().kind === "select"}>
                        <View class="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] items-center gap-x-6 min-h-14 py-2 px-5 text-lg compact:grid-cols-1 compact:items-stretch compact:gap-y-1 compact:px-3 compact:text-base">
                          <Text class="grow shrink font-semibold m-0">{entry().name}</Text>
                          <Select
                            value={String(entry().value)}
                            options={(entry().options ?? []).map((o) => ({ value: o, label: o }))}
                            open={openChoice() === entry().id}
                            label={entry().name}
                            onChange={(e, v) => { activeEntryId.set(entry().id); props.onChange?.(e, entry().id, v); }}
                            onOpenChange={(_, open) => { activeEntryId.set(entry().id); openChoice.set(open ? entry().id : ""); }}
                          />
                        </View>
                      </Show>
                      <Show when={entry().kind === "binding"}>
                        <View class={`grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] items-center gap-x-6 min-h-14 py-2 px-5 text-lg compact:grid-cols-1 compact:items-stretch compact:gap-y-1 compact:px-3 compact:text-base compact:grid-cols-[minmax(0,1fr)_auto] data-listening:bg-hud-accent data-listening:text-hud-accent-fg`} role="button" tabIndex={0} label={entry().name} data-listening={listening() === entry().id} onPress={() => { activeEntryId.set(entry().id); listening.set(listening() === entry().id ? "" : entry().id); }} onKey={(e) => {
                          if (listening() !== entry().id) return;
                          if (e.key === "Escape") {
                            listening.set("");
                            return;
                          }
                          props.onRebind?.(e, entry().id, e.key);
                          listening.set("");
                        }}>
                          <Text class="grow shrink font-semibold m-0 group-data-listening/row:text-hud-accent-fg">{entry().name}</Text>
                          <Show when={listening() === entry().id} fallback={<InputGlyph class="justify-self-end" action={entry().action} size="lg" />}>
                            <Text class="justify-self-end font-semibold text-hud-accent-fg whitespace-nowrap">{t("screen.settings.pressAKey")}</Text>
                          </Show>
                        </View>
                      </Show>
                    </View>
                  </Show>
                )}
              </For>
            </Scroll>
            <Show when={inspected()}>
              {(item) => (
                <Show when={item().description}>
                  <View class={`flex-col gap-y-1 py-3 px-5 ${packClass("Slot", "frame")}`}>
                    <Text class="font-display text-lg font-bold leading-snug m-0">{item().name}</Text>
                    <Text class="text-base leading-normal m-0">{item().description}</Text>
                  </View>
                </Show>
              )}
            </Show>
            <View class="flex-row flex-wrap items-center justify-end gap-y-2 gap-x-3 compact:grid compact:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
              <Show when={props.note !== ""}>
                <Text class="mr-auto text-base text-fg-muted m-0 shrink line-clamp-2 portrait:w-full compact:col-span-full">{props.note}</Text>
              </Show>
              <Show when={props.onDefaults}>
                <Button class="shrink" onPress={(e) => props.onDefaults?.(e)}>
                  <Text>{t("screen.settings.defaults")}</Text>
                </Button>
              </Show>
              <Show when={props.onRevert}>
                <Button class="min-w-36 compact:min-w-0 shrink" onPress={(e) => props.onRevert?.(e)} disabled={!props.pending}>
                  <Text>{t("screen.settings.revert")}</Text>
                </Button>
              </Show>
              <Show when={props.onApply}>
                <Button variant="primary" class="min-w-36 compact:min-w-0 shrink" disabled={!props.pending} onPress={(e) => props.onApply?.(e)}>
                  <Text>{t("screen.settings.apply")}</Text>
                </Button>
              </Show>
            </View>
          </Show>
          {props.tree?.()}
        </View>
      </View>
      <View class="absolute inset-x-0 bottom-0 flex-row pointer-events-none pb-[calc(env(hud-inset-bottom)+1.5rem)] pl-[calc(env(hud-inset-left)+3rem)] portrait:pl-[calc(env(hud-inset-left)+1.5rem)] compact:pb-[calc(env(hud-inset-bottom)+0.75rem)] compact:pl-[calc(env(hud-inset-left)+1rem)]">
        <HintBar hints={[{ action: "confirm", label: t("screen.common.select") }, { action: "back", label: t("screen.common.back") }, { action: "tabNext", label: t("screen.settings.page") }]} />
      </View>
    </View>
  );
}

/** The fixtures' handler: a row is drawn only where the game passes one. */
const press = () => {};

export const fixtures: Record<string, Partial<SettingsScreenProps>> = {
  default: {"note":"Some changes apply after a restart.","pages":[{"id":"general","name":"General","icon":"settings"},{"id":"audio","name":"Audio","icon":"sound"},{"id":"video","name":"Video","icon":"video"},{"id":"controls","name":"Controls","icon":"gamepad"}],"page":"general","pending":true,"entries":[{"id":"subtitles","page":"general","name":"Subtitles","description":"Show dialogue captions.","kind":"toggle","value":true},{"id":"volume","page":"general","name":"Master volume","description":"Adjust all game audio.","kind":"range","value":75,"min":0,"max":100,"step":5},{"id":"quality","page":"general","name":"Quality","description":"Choose the rendering quality.","kind":"select","value":"High","options":["Low","High"]},{"id":"language","page":"general","name":"Language","description":"The language of menus and subtitles.","kind":"select","value":"English","options":["English","中文"]},{"id":"textSize","page":"general","name":"Text size","description":"Scale every word on screen.","kind":"range","value":100,"min":80,"max":150,"step":10},{"id":"vibration","page":"general","name":"Vibration","description":"Rumble on hits and rewards.","kind":"toggle","value":false},{"id":"interact","page":"general","name":"Interact","description":"Choose the interaction key.","kind":"binding","value":"E","action":"interact"}],"onBack":press,"onDefaults":press,"onRevert":press,"onApply":press},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Select","back":"Back","tabNext":"Next page","tabPrev":"Previous page"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"settings","layer":"menu","focus":"Tree","under":"dim","input":"menu","pause":false,"capture":true,"remember":true,"back":"back"} as const;
