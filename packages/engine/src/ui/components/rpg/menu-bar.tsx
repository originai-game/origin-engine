import { For, Icon, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./menu-bar.en.ogt";

export interface RpgMenuBarProps {
  /** The menu's screens, keyed rows of { id, label } (Character, Inventory, Skills, Quests, Map). */
  tabs?: ReadonlyArray<{ id: string; label: string }>;
  /** The screen shown. */
  tab: string;
  /** Choose a screen (its key). */
  onChoose?: (e: UiEvent, id: string) => void;
  /** Previous screen. */
  onTabPrev?: (e: UiEvent) => void;
  /** Next screen. */
  onTabNext?: (e: UiEvent) => void;
  /** Close (the button, Escape or B). */
  onClose?: (e: UiEvent) => void;
}

/**
 * The menu's top bar: its screens as tabs and a close button. Reports
 * choose (a screen's key), tabPrev, tabNext and close (the button, or the
 * tabPrev, tabNext and back input actions).
 */
export function RpgMenuBar(props: RpgMenuBarProps) {
  return (
    <View class={`flex-row items-center w-full text-fg gap-x-2 compact:gap-x-0`} role="navigation" label={t("rpg.menu.name")}
      onAction={{ tabPrev: (e) => props.onTabPrev?.(e), tabNext: (e) => props.onTabNext?.(e), back: (e) => props.onClose?.(e) }}>
      <View class="relative flex-1 min-w-0 shadow-[inset_0px_-1px_0px_#ffffff24]">
        <View class="flex-row w-full">
          <For each={props.tabs ?? []} key={(c) => c.id}>
            {(c) => (
            <View class={`group/tab flex-1 min-w-0 compact:flex-none compact:basis-auto flex-row items-center justify-center min-h-11 px-4 compact:px-3 whitespace-nowrap text-[1.143rem] compact:text-[0.929rem] font-bold tracking-[0.12em] compact:tracking-[0.04em] uppercase text-fg-muted hover:bg-[#ffffff0f] data-selected:text-fg data-selected:bg-linear-to-t data-selected:from-[#38e8ff33] data-selected:to-transparent data-selected:shadow-[inset_0px_-3px_0px_#38e8ff] focus-visible:shadow-[inset_0px_0px_0px_2px_#ffffff]`} role="tab" label={c().label} selected={c().id === props.tab} data-selected={c().id === props.tab} onPress={(e) => props.onChoose?.(e, c().id)}>
              <Text class="whitespace-nowrap overflow-hidden">
                {c().label}
              </Text>
            </View>
            )}
          </For>
        </View>
      </View>
      <View class={`shrink-0 size-11 flex-row items-center justify-center bg-linear-to-b from-[#2a3242ea] to-[#1b2230f2] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none shadow-[inset_0px_0px_0px_1px_#ffffff3d,inset_0px_1px_0px_#ffffff99] hover:from-[#3d4759f2] hover:to-[#2a3242f5]`} role="button" label={t("rpg.menu.close")} onPress={(e) => props.onClose?.(e)}>
        <Icon class="size-1/2 text-fg" name="close" />
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgMenuBarProps>> = {
  default: {"tabs":[{"id":"char","label":"Character"},{"id":"inv","label":"Inventory"}],"tab":"char"},
};
