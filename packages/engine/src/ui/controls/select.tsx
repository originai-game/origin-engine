import { For, Icon, Show, Text, View, createSignal, packClass, type UiEvent } from "@origingame/engine/ui";

export interface SelectOption {
  readonly value: string;
  readonly label?: string | undefined;
}

export interface SelectProps {
  /** Currently selected option value. */
  value: string;
  /** Choices as strings or `{ value, label }` entries. */
  options: ReadonlyArray<string | SelectOption>;
  /** Accessible name (and placeholder when no option matches). */
  label?: string | undefined;
  placeholder?: string | undefined;
  disabled?: boolean | undefined;
  class?: string | undefined;
  tabIndex?: number | undefined;
  onChange?: ((e: UiEvent, value: string) => void) | undefined;
  [data: `data-${string}`]: unknown;
}

/**
 * A combobox / dropdown selector (`role="combobox"` + `role="listbox"`).
 * Supports click/press to open, Arrow keys to cycle, and Escape/`back` to close.
 */
export function Select(props: SelectProps) {
  const open = createSignal(false);
  const items = (): SelectOption[] =>
    props.options.map((o) => (typeof o === "string" ? { value: o, label: o } : { value: o.value, label: o.label ?? o.value }));
  const currentLabel = () => {
    const found = items().find((o) => o.value === props.value);
    return found ? (found.label ?? found.value) : (props.value || props.placeholder || "");
  };
  return (
    <View class={`flex-col gap-1 min-w-0 ${props.class ?? ""}`}>
      <View
        role="combobox"
        tabIndex={props.tabIndex ?? 0}
        disabled={props.disabled}
        label={props.label}
        expanded={open()}
        valueText={currentLabel()}
        data-open={open()}
        onPress={() => {
          if (!props.disabled) open.set(!open());
        }}
        onKey={(e) => {
          if (props.disabled) return;
          const list = items();
          if (list.length === 0) return;
          if (e.key === "Escape") {
            open.set(false);
            return;
          }
          const idx = Math.max(0, list.findIndex((o) => o.value === props.value));
          const wrap = (n: number) => list[((n % list.length) + list.length) % list.length]!;
          if (e.key === "ArrowDown" || e.key === "ArrowRight") props.onChange?.(e, wrap(idx + 1).value);
          else if (e.key === "ArrowUp" || e.key === "ArrowLeft") props.onChange?.(e, wrap(idx - 1).value);
        }}
        class={`flex-row items-center justify-between gap-2 min-h-11 py-0 px-3 rounded-control bg-tile text-fg inset-ring-1 inset-ring-rim hover:bg-surface-hover data-open:inset-ring-accent disabled:text-fg-subtle disabled:opacity-45 ${packClass("Select", "trigger")}`}
      >
        <Text class="text-sm font-semibold text-start min-w-0 truncate">{currentLabel()}</Text>
        <Icon class={`size-4 shrink-0 ${open() ? "rotate-180" : ""}`} name="chevronDown" />
      </View>
      <Show when={open()}>
        <View
          role="listbox"
          label={props.label}
          onAction={{ back: () => open.set(false) }}
          class={`flex-col rounded-control bg-tile inset-ring-1 inset-ring-rim shadow-panel overflow-hidden ${packClass("Select", "menu")}`}
        >
          <For each={items()} key={(o) => o.value}>
            {(opt) => (
              <View
                role="option"
                tabIndex={0}
                selected={opt().value === props.value}
                data-selected={opt().value === props.value}
                label={opt().label ?? opt().value}
                onPress={(e) => {
                  props.onChange?.(e, opt().value);
                  open.set(false);
                }}
                class="flex-row items-center justify-start gap-2 min-h-9 py-1 px-3 text-start text-sm font-semibold text-fg bg-raised-lo hover:bg-raised-hover-lo data-selected:text-accent data-selected:bg-surface-hover"
              >
                <Text class="whitespace-nowrap">{opt().label ?? opt().value}</Text>
              </View>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

const noop = () => {};

export const fixtures: Record<string, Partial<SelectProps>> = {
  default: { value: "medium", options: ["low", "medium", "high", "ultra"], label: "Quality", onChange: noop },
  labelled: {
    value: "en",
    options: [
      { value: "en", label: "English" },
      { value: "zh-CN", label: "简体中文" },
      { value: "ja", label: "日本語" },
    ],
    label: "Language",
    onChange: noop,
  },
  disabled: { value: "high", options: ["low", "high"], disabled: true, label: "Quality", onChange: noop },
};
