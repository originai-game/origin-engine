import { For, Icon, Scroll, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import { Button } from "../controls/button";
import { TextField } from "../controls/field";
import { InputGlyph } from "../controls/input-glyph";
import "./controls.en.strings.ts";

export interface ControlsBinding {
  /** The action's stable id (jump, interact). */
  id: string;
  /** The action as a player reads it (Jump). */
  label: string;
  /** The group it is listed under (Movement, Combat); empty lists it under All only. */
  category?: string;
  /** The keyboard or mouse binding's words (Space, E, LMB); empty reads Unbound. */
  key?: string;
  /** The gamepad binding's words (A, RB); empty reads Unbound. */
  pad?: string;
  /** The binding is fixed (pause, the menu key): shown, not offered for rebinding. */
  locked?: boolean;
  /** Another action uses the same input: the row is marked, the game decides what that means. */
  conflict?: boolean;
}

export type ControlsDevice = "key" | "pad";

export interface ControlsScreenProps {
  /** Classes after the screen's own. */
  class?: string | undefined;
  /** The bindings, in the order the game lists them. */
  bindings?: ReadonlyArray<ControlsBinding> | undefined;
  /** The category shown; empty shows all. The game owns it (each `onCategory` writes it back). */
  category?: string | undefined;
  /** The search text; the screen lists the rows whose label contains it. */
  search?: string | undefined;
  /** The binding waiting for an input, as `id:key` or `id:pad`: that cell reads Press a key. */
  listening?: string | undefined;
  /** Hide the gamepad column (a keyboard-only game). */
  hidePad?: boolean | undefined;
  /** A cell was pressed: the game listens for the next input and writes the binding back. */
  onRebind?: ((e: UiEvent, id: string, device: ControlsDevice) => void) | undefined;
  /** Clear a binding (the cell's clear button). */
  onClear?: ((e: UiEvent, id: string, device: ControlsDevice) => void) | undefined;
  onCategory?: ((e: UiEvent, category: string) => void) | undefined;
  onSearch?: ((e: UiEvent, text: string) => void) | undefined;
  /** Restore the game's default bindings. */
  onReset?: ((e: UiEvent) => void) | undefined;
  onBack?: ((e: UiEvent) => void) | undefined;
}

/**
 * The control rebinding screen: every action with its keyboard and gamepad
 * binding, a category filter and a search. Pressing a cell asks the game to
 * listen (`onRebind`); the game captures the next input, resolves any
 * conflict as it chooses and writes the binding back. The screen never
 * decides a binding.
 */
export function ControlsScreen(props: ControlsScreenProps) {
  const categories = () => {
    const seen: string[] = [];
    for (const b of props.bindings ?? []) if (b.category && !seen.includes(b.category)) seen.push(b.category);
    return seen;
  };
  const rows = () => {
    const query = (props.search ?? "").trim().toLowerCase();
    return (props.bindings ?? []).filter(
      (b) => (!props.category || b.category === props.category) && (!query || b.label.toLowerCase().includes(query)),
    );
  };
  const cell = (b: ControlsBinding, device: ControlsDevice) => {
    const value = () => (device === "key" ? b.key : b.pad) ?? "";
    const listening = () => props.listening === `${b.id}:${device}`;
    return (
      <View class="flex-row items-center gap-x-1 w-40 compact:w-28">
        <Button
          class="grow shrink basis-0 min-w-0"
          size="sm"
          variant="ghost"
          disabled={b.locked === true}
          selected={listening()}
          label={t("screen.controls.rebind", { action: b.label, device: device === "key" ? t("screen.controls.keyboard") : t("screen.controls.gamepad") })}
          onPress={(e) => props.onRebind?.(e, b.id, device)}
        >
          <Show when={!listening()} fallback={<Text class="text-xs font-semibold text-hud-accent animate-pulse">{t("screen.controls.listening")}</Text>}>
            <Show when={value()} fallback={<Text class="text-xs text-fg-subtle">{t("screen.controls.unbound")}</Text>}>
              <InputGlyph keyLabel={value()} pad={device === "pad"} size="sm" disabled={b.locked === true} />
            </Show>
          </Show>
        </Button>
        <Show when={props.onClear && value() && !b.locked}>
          <Button size="sm" variant="ghost" label={t("screen.controls.clear", { action: b.label })} onPress={(e) => props.onClear?.(e, b.id, device)}>
            <Icon class="size-3.5 text-fg-muted" name="close" />
          </Button>
        </Show>
      </View>
    );
  };
  return (
    <View
      class={`absolute inset-0 pt-[calc(env(hud-inset-top)+2rem)] pb-[calc(env(hud-inset-bottom)+1.5rem)] px-12 text-fg gap-y-6 overflow-hidden compact:pt-[calc(env(hud-inset-top)+1rem)] compact:pb-[calc(env(hud-inset-bottom)+0.75rem)] compact:gap-y-3 compact:px-4 ${packClass("Screen", "frame")} ${props.class ?? ""}`}
      onAction={{ back: (e) => props.onBack?.(e), details: (e) => props.onReset?.(e) }}
    >
      <View class="absolute bg-glass inset-0" />
      <View class="flex-row min-h-12 items-center gap-x-4 py-1 px-4 narrow:flex-wrap narrow:gap-y-1">
        <Show when={props.onBack}>
          <Button class="text-fg-muted narrow:-ml-2 shrink" onPress={(e) => props.onBack?.(e)}>
            <View class="flex-row items-center gap-x-1">
              <Icon class="size-4 rtl:-scale-x-100 shrink" name="back" />
              <Text class="shrink">{t("screen.common.back")}</Text>
            </View>
          </Button>
        </Show>
        <Text class="font-display text-display-lg font-bold leading-tight tracking-xs text-fg text-shadow-xs text-shadow-bg shrink m-0 compact:text-3xl">
          {t("screen.controls.title")}
        </Text>
        <View class="grow shrink basis-0" />
        <Show when={props.onReset}>
          <Button size="sm" variant="ghost" onPress={(e) => props.onReset?.(e)}>
            <Text>{t("screen.controls.reset")}</Text>
          </Button>
        </Show>
      </View>
      <View class="flex-row flex-wrap items-end gap-x-4 gap-y-2">
        <View class="flex-row flex-wrap gap-1" role="tablist" focusGroup="stop" focusRoving>
          <Button size="sm" variant="ghost" selected={!props.category} onPress={(e) => props.onCategory?.(e, "")}>
            <Text>{t("screen.controls.all")}</Text>
          </Button>
          <For each={categories()}>
            {(c) => (
              <Button size="sm" variant="ghost" selected={props.category === c()} onPress={(e) => props.onCategory?.(e, c())}>
                <Text>{c()}</Text>
              </Button>
            )}
          </For>
        </View>
        <View class="grow" />
        <Show when={props.onSearch}>
          <TextField
            class="w-64 narrow:w-full"
            type="search"
            label={t("screen.controls.search")}
            placeholder={t("screen.controls.searchHint")}
            value={props.search ?? ""}
            onChange={(e, v) => props.onSearch?.(e, v)}
          />
        </Show>
      </View>
      <View class="flex-row items-center gap-x-4 px-3 text-xs font-semibold uppercase tracking-label text-fg-muted">
        <Text class="grow shrink basis-0 min-w-0">{t("screen.controls.action")}</Text>
        <Text class="w-40 compact:w-28">{t("screen.controls.keyboard")}</Text>
        <Show when={!props.hidePad}>
          <Text class="w-40 compact:w-28">{t("screen.controls.gamepad")}</Text>
        </Show>
      </View>
      <Scroll class="grow shrink basis-0 p-1 -m-1">
        <View class="gap-y-1" focusGroup="stop">
          <Show when={rows().length > 0} fallback={<Text class="text-base text-fg-muted text-center py-8">{t("screen.controls.none")}</Text>}>
            <For each={rows()} key={(b) => b.id}>
              {(b) => (
                <View class={`flex-row items-center gap-x-4 py-1.5 px-3 ${packClass("Row", "frame")}`} data-conflict={b().conflict === true}>
                  <View class="grow shrink basis-0 min-w-0 flex-row items-center gap-x-2">
                    <Text class="truncate text-base">{b().label}</Text>
                    <Show when={b().locked}>
                      <Icon class="shrink-0 size-3.5 text-fg-subtle" name="lock" />
                    </Show>
                    <Show when={b().conflict}>
                      <Text class="shrink-0 text-xs font-semibold text-warn" role="status">{t("screen.controls.conflict")}</Text>
                    </Show>
                  </View>
                  {cell(b(), "key")}
                  <Show when={!props.hidePad}>{cell(b(), "pad")}</Show>
                </View>
              )}
            </For>
          </Show>
        </View>
      </Scroll>
    </View>
  );
}

const press = () => {};

export const fixtures: Record<string, Partial<ControlsScreenProps>> = {
  default: {
    bindings: [
      { id: "forward", label: "Move forward", category: "Movement", key: "W", pad: "LS" },
      { id: "jump", label: "Jump", category: "Movement", key: "Space", pad: "A" },
      { id: "sprint", label: "Sprint", category: "Movement", key: "Shift", pad: "LS", conflict: true },
      { id: "attack", label: "Attack", category: "Combat", key: "LMB", pad: "RT" },
      { id: "block", label: "Block", category: "Combat", key: "RMB", pad: "LT" },
      { id: "interact", label: "Interact", category: "General", key: "E", pad: "X" },
      { id: "map", label: "Map", category: "General", key: "", pad: "" },
      { id: "pause", label: "Pause", category: "General", key: "Esc", pad: "Start", locked: true },
    ],
    listening: "block:key",
    onRebind: press,
    onClear: press,
    onCategory: press,
    onSearch: press,
    onReset: press,
    onBack: press,
  },
  keyboard: { hidePad: true, bindings: [{ id: "jump", label: "Jump", category: "Movement", key: "Space" }], onBack: press },
  empty: { bindings: [], search: "zzz", onSearch: press, onBack: press },
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = { confirm: "Rebind", back: "Back", details: "Reset" } as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = { name: "controls", layer: "menu", focus: "", under: "dim", input: "menu", pause: true, capture: true, remember: true, back: "pop" } as const;
