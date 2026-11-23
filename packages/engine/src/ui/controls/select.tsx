import { For, Icon, Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean | undefined;
}

export interface SelectProps {
  /** The chosen option's value. The game owns it: a pick reports `onChange`. */
  value: string;
  options: readonly SelectOption[];
  /** Its list is shown. The game owns it: a press on the trigger reports `onOpenChange`, a pick or `back` closes it. */
  open?: boolean | undefined;
  disabled?: boolean | undefined;
  /** What screen readers call it ("Quality"). */
  label?: string | undefined;
  /** Classes of the document that uses it, on its trigger. */
  class?: string | undefined;
  onChange?: ((e: UiEvent, value: string) => void) | undefined;
  onOpenChange?: ((e: UiEvent, open: boolean) => void) | undefined;
}

/**
 * One value picked from a list (NSH base's `Select`): a combobox for
 * screen readers whose arrows step the value while closed, and whose list
 * opens under the trigger at its width, as one focus
 * group the arrows move in. The pack draws the trigger (`Select.trigger`,
 * `aria-expanded:` while open), the list (`Select.list`) and each option
 * (`Select.option`, `selected:` for the chosen one).
 */
export function Select(props: SelectProps) {
  const chosen = () => props.options.find((o) => o.value === props.value);
  const step = (e: UiEvent, by: number) => {
    const live = props.options.filter((o) => !o.disabled);
    if (!live.length) return;
    const at = Math.max(0, live.findIndex((o) => o.value === props.value));
    props.onChange?.(e, live[(at + by + live.length) % live.length]!.value);
  };
  return (
    <View class="relative h-fit">
      <View
        role="combobox"
        tabIndex={0}
        class={`${packClass("Select", "trigger")} ${props.class ?? ""}`}
        label={props.label}
        valueText={chosen()?.label ?? ""}
        expanded={props.open === true}
        disabled={props.disabled}
        onPress={(e) => props.onOpenChange?.(e, props.open !== true)}
        onKey={(e) => {
          if (props.open || props.disabled) return;
          if (e.key === "ArrowDown" || e.key === "ArrowRight") step(e, 1);
          else if (e.key === "ArrowUp" || e.key === "ArrowLeft") step(e, -1);
        }}
      >
        <Text class="min-w-0 truncate">{chosen()?.label ?? ""}</Text>
        <Icon name="chevronDown" class="size-4 shrink-0" />
      </View>
      <Show when={props.open}>
        <View
          role="listbox"
          label={props.label}
          focusGroup="wrap"
          class={`absolute top-full inset-x-0 z-40 mt-1 ${packClass("Select", "list")}`}
          onAction={{ back: (e) => props.onOpenChange?.(e, false) }}
        >
          <For each={props.options} key={(o) => o.value}>
            {(o) => (
              <View
                role="option"
                tabIndex={0}
                class={packClass("Select", "option")}
                selected={o().value === props.value}
                disabled={o().disabled}
                autofocus={o().value === props.value}
                label={o().label}
                onPress={(e) => {
                  props.onChange?.(e, o().value);
                  props.onOpenChange?.(e, false);
                }}
              >
                <Text class="grow">{o().label}</Text>
                <Show when={o().value === props.value}>
                  <Icon name="check" class="size-4 shrink-0" />
                </Show>
              </View>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

const change = () => {};
const QUALITY = [
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
  { value: "ultra", label: "Ultra", disabled: true },
];

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<SelectProps>> = {
  closed: { value: "high", options: QUALITY, label: "Quality", onChange: change, onOpenChange: change },
  open: { value: "high", options: QUALITY, open: true, label: "Quality", onChange: change, onOpenChange: change },
};
