import {createSignal, For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Button } from "../controls/button";
import { Select } from "../controls/select";
import { Slider } from "../controls/slider";
import { Switch } from "../controls/switch";
import { Tab, Tabs } from "../controls/tabs";
import "./settings.en.ogt";

export interface SettingsScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
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
      class={`absolute inset-0 pt-[calc(env(hud-inset-top)+2rem)] pb-[calc(env(hud-inset-bottom)+1.5rem)] pl-[calc(env(hud-inset-left)+3rem)] pr-[calc(env(hud-inset-right)+3rem)] text-fg gap-y-4 overflow-hidden compact:pt-[calc(env(hud-inset-top)+1rem)] compact:pb-[calc(env(hud-inset-bottom)+0.75rem)] compact:pl-[calc(env(hud-inset-left)+1rem)] compact:pr-[calc(env(hud-inset-right)+1rem)] compact:gap-y-3 ${packClass("Screen", "frame")} ${props.class ?? ""}`}
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
      <View class={`absolute bg-glass backdrop-blur-regular inset-0`} />
      <View class={`flex-row min-h-12 items-baseline gap-x-4 py-1 px-4 compact:flex-wrap compact:gap-y-1`}>
        <Show when={props.onBack}>
          <Button class={`text-fg-muted compact:-ml-2 shrink`} autofocus onPress={(e) => props.onBack?.(e)}>
            <View class="flex-row items-center gap-x-1">
              <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100 shrink" name="chevronLeft" />
              <Text class="shrink">{t("screen.common.back")}</Text>
            </View>
          </Button>
        </Show>
        <View class="flex-row grow shrink basis-0 items-baseline flex-wrap gap-x-4 gap-y-0.5 compact:grow compact:shrink-0 compact:basis-full compact:items-start compact:flex-col">
          <Text class={`font-display text-display-lg font-bold leading-tight tracking-xs text-fg text-shadow-xs text-shadow-bg m-0 compact:text-3xl shrink`}>
            {t("screen.settings.title")}
          </Text>
          <Show when={props.note !== ""}>
            <Text class="mr-0 ml-auto text-base text-warn line-clamp-2 my-0 compact:ml-0 compact:text-caption shrink">
              {props.note}
            </Text>
          </Show>
        </View>
      </View>
      <View class="grow shrink basis-0 gap-y-4 min-h-0">
        <Show when={props.page}>
          <View class={`flex-row items-center gap-x-3`}>
            <Text class="flex-none pointer-coarse:hidden" glyph="tabPrev" />
            <Scroll class="grow shrink compact:-mx-4">
              <Tabs class="border-b-0 gap-x-2 px-2" label={t("screen.settings.title")}>
                <For each={props.pages}>
                  {(p) => (
                    <Tab class="shrink-0" selected={props.page === p().id} onPress={(e) => props.onPage?.(e, p().id)}>
                      <Text class="whitespace-nowrap">{p().name}</Text>
                    </Tab>
                  )}
                </For>
              </Tabs>
            </Scroll>
            <Text class="flex-none pointer-coarse:hidden" glyph="tabNext" />
          </View>
          <View class="grow shrink basis-0 min-h-0 grid grid-cols-[minmax(0,3fr)_minmax(0,2fr)] grid-rows-1 gap-x-12 compact:grid-cols-1">
            <Scroll class={`grow shrink basis-0 gap-y-0.5 p-0 m-0 ${packClass("List", "cascade")}`}>
              <For each={props.entries} key={(entry) => entry.id}>
                {(entry) => (
                  <Show when={entry().page === props.page}>
                    <View
                      class={`group/row text-fg transition duration-control hover:bg-surface-hover data-selected:text-fg disabled:text-fg-subtle ${packClass("Row", "frame")} ${packClass("List", "item")}`}
                      label={entry().description}
                      data-selected={inspected()?.id === entry().id}
                      onFocus={() => activeEntryId.set(entry().id)}
                      onPointerDown={() => activeEntryId.set(entry().id)}
                    >
                      <Show when={entry().kind === "toggle"}>
                        <Switch class="min-h-16 text-lg text-fg-muted py-1 px-4 compact:text-base compact:py-2 compact:px-3 group-hover/row:text-fg" checked={entry().value === true} onChange={(e, on) => { activeEntryId.set(entry().id); props.onChange?.(e, entry().id, on); }}>
                          <Text class="grow shrink">{entry().name}</Text>
                        </Switch>
                      </Show>
                      <Show when={entry().kind === "range"}>
                        <View class="grid grid-cols-2 items-center gap-x-6 min-h-16 text-lg py-1 px-4 compact:grid-cols-1 compact:items-stretch compact:text-base compact:gap-y-1 compact:py-2 compact:px-3">
                          <Text class="grow shrink text-fg-muted m-0 group-hover/row:text-fg">
                            {entry().name}
                          </Text>
                          <Slider
                            value={typeof entry().value === "number" ? (entry().value as number) : (entry().min ?? 0)}
                            min={entry().min}
                            max={entry().max}
                            step={entry().step}
                            label={entry().name}
                            onChange={(e, v) => { activeEntryId.set(entry().id); props.onChange?.(e, entry().id, v); }}
                          />
                        </View>
                      </Show>
                      <Show when={entry().kind === "select"}>
                        <View class="grid grid-cols-2 items-center gap-x-6 min-h-16 text-lg py-1 px-4 compact:grid-cols-1 compact:items-stretch compact:text-base compact:gap-y-1 compact:py-2 compact:px-3">
                          <Text class="grow shrink text-fg-muted m-0 group-hover/row:text-fg">
                            {entry().name}
                          </Text>
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
                        <View class={`grid grid-cols-2 items-center gap-x-6 min-h-16 text-lg py-1 px-4 compact:grid-cols-[minmax(0,1fr)_auto] compact:items-center compact:text-base compact:gap-y-1 compact:py-2 compact:px-3 group-hover/row:text-fg data-listening:bg-hud-accent data-listening:text-hud-accent-fg`} role="button" tabIndex={0} label={entry().name} data-listening={listening() === entry().id} onPress={() => { activeEntryId.set(entry().id); listening.set(listening() === entry().id ? "" : entry().id); }} onKey={(e) => {
                          if (listening() !== entry().id) return;
                          if (e.key === "Escape") {
                            listening.set("");
                            return;
                          }
                          props.onRebind?.(e, entry().id, e.key);
                          listening.set("");
                        }}>
                          <Text class="grow shrink text-fg-muted m-0 group-hover/row:text-fg group-data-listening/row:text-hud-accent-fg">
                            {entry().name}
                          </Text>
                          <Show when={listening() === entry().id} fallback={<Text class="justify-self-end" glyph={entry().action} />}>
                            <Text class="justify-self-end font-semibold text-hud-accent-fg whitespace-nowrap">{t("screen.settings.pressAKey")}</Text>
                          </Show>
                        </View>
                      </Show>
                    </View>
                  </Show>
                )}
              </For>
            </Scroll>
            <View class="flex-col justify-start gap-y-2 compact:hidden">
              <Show when={inspected()}>
                {(item) => (
                  <Show when={item().description}>
                    <View class={`flex-col gap-y-2 p-5 ${packClass("Plate", "frame")}`}>
                      <Text class="font-display text-xl font-bold leading-snug text-fg m-0">{item().name}</Text>
                      <Text class="text-base leading-normal text-fg-muted m-0">{item().description}</Text>
                    </View>
                  </Show>
                )}
              </Show>
            </View>
          </View>
          <View class="flex-row flex-wrap items-center justify-end gap-y-2 gap-x-3 compact:grid compact:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
            <Show when={props.onDefaults}>
              <Button class={`mr-auto compact:mr-0 compact:col-span-full compact:justify-self-start compact:-ml-2 shrink`} onPress={(e) => props.onDefaults?.(e)}>
                <Text>{t("screen.settings.defaults")}</Text>
              </Button>
            </Show>
            <Show when={props.onRevert}>
              <Button class="min-w-40 compact:min-w-0 shrink" onPress={(e) => props.onRevert?.(e)} disabled={!props.pending}>
                <Text>{t("screen.settings.revert")}</Text>
              </Button>
            </Show>
            <Show when={props.onApply}>
              <Button variant="primary" class="min-w-40 compact:min-w-0 shrink" disabled={!props.pending} onPress={(e) => props.onApply?.(e)}>
                <Text>{t("screen.settings.apply")}</Text>
              </Button>
            </Show>
          </View>
        </Show>
        {props.tree?.()}
      </View>
    </View>
  );
}

/** The fixtures' handler: a row is drawn only where the game passes one. */
const press = () => {};

export const fixtures: Record<string, Partial<SettingsScreenProps>> = {
  default: {"note":"Some changes apply after a restart.","pages":[{"id":"general","name":"General"}],"page":"general","pending":true,"entries":[{"id":"subtitles","page":"general","name":"Subtitles","description":"Show dialogue captions.","kind":"toggle","value":true},{"id":"volume","page":"general","name":"Master volume","description":"Adjust all game audio.","kind":"range","value":75,"min":0,"max":100,"step":5},{"id":"quality","page":"general","name":"Quality","description":"Choose the rendering quality.","kind":"select","value":"High","options":["Low","High"]},{"id":"interact","page":"general","name":"Interact","description":"Choose the interaction key.","kind":"binding","value":"E","action":"interact"}],"onBack":press,"onDefaults":press,"onRevert":press,"onApply":press},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Select","back":"Back","tabNext":"Next page","tabPrev":"Previous page"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"settings","layer":"menu","focus":"Tree","under":"dim","input":"menu","pause":false,"capture":true,"remember":true,"back":"back"} as const;
