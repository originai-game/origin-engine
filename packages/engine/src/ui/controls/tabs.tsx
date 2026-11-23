import { For, Scroll, Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface TabItem {
  readonly id: string;
  readonly label: string;
  readonly disabled?: boolean | undefined;
}

export interface TabsProps {
  /** Keyed tab definitions. */
  tabs: ReadonlyArray<TabItem>;
  /** Currently active tab `id`. */
  active: string;
  /** Accessible name of the tablist. */
  label?: string | undefined;
  class?: string | undefined;
  onChange?: ((e: UiEvent, id: string) => void) | undefined;
}

/**
 * A horizontal tab bar (`role="tablist"` + `role="tab"`) with an active accent
 * indicator and `tabPrev`/`tabNext` action support.
 */
export function Tabs(props: TabsProps) {
  return (
    <View
      role="tablist"
      label={props.label}
      focusGroup="wrap"
      class={`flex-row items-center gap-x-2 border-b border-rim ${packClass("Tab", "bar")} ${props.class ?? ""}`}
    >
      <Scroll class="grow shrink flex-row items-center gap-x-2 py-0.5 px-1">
        <For each={props.tabs} key={(t) => t.id}>
          {(tab) => {
            const isSelected = () => props.active === tab().id;
            return (
              <View
                role="tab"
                tabIndex={0}
                selected={isSelected()}
                data-selected={isSelected()}
                disabled={tab().disabled}
                label={tab().label}
                onPress={(e) => {
                  if (!tab().disabled) props.onChange?.(e, tab().id);
                }}
                class="relative flex-row items-center justify-center min-h-11 px-4 text-sm font-semibold text-fg-muted rounded-control transition duration-200 hover:bg-surface-hover data-selected:text-fg data-selected:bg-raised-hover-lo disabled:text-fg-subtle disabled:opacity-45"
              >
                <Text class="whitespace-nowrap">{tab().label}</Text>
                <Show when={isSelected()}>
                  <View
                    class={`absolute inset-x-2 bottom-0 h-0.75 rounded-capsule bg-accent pointer-events-none ${packClass("Tab", "indicator")}`}
                  />
                </Show>
              </View>
            );
          }}
        </For>
      </Scroll>
    </View>
  );
}

const noop = () => {};

export const fixtures: Record<string, Partial<TabsProps>> = {
  default: {
    active: "video",
    tabs: [
      { id: "general", label: "General" },
      { id: "video", label: "Video" },
      { id: "audio", label: "Audio" },
      { id: "controls", label: "Controls" },
    ],
    label: "Settings sections",
    onChange: noop,
  },
};
