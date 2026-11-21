import { Icon, Image, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import { Spinner } from "./spinner";
import "./states.en.ogt";

export interface ListItemProps {
  /** The row's primary text. */
  title?: string;
  /** Optional secondary text, wrapping below the title. */
  subtitle?: string;
  /** Optional leading icon, used when image is zero. */
  icon?: string;
  /** Optional loaded portrait or item texture handle. Default 0. */
  image?: number;
  /** The game's formatted trailing value. */
  value?: string;
  /** Optional trailing badge, such as New or 3. */
  badge?: string;
  /** Chosen by the game; marked with a check and aria-pressed. Default false. */
  selected?: boolean;
  /** Unavailable; cannot choose. Default false. */
  disabled?: boolean;
  /** Refreshing this row; prevents choosing and shows a spinner without hiding its content. Default false. */
  loading?: boolean;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
}

/**
 * One line of a list: what it is, its name over a supporting line, then
 * what the game says about it at the trailing end (a value over a
 * badge), then its state (chosen or refreshing). The trailing end stays
 * on the row while the words keep a measure beside it; a long value on a
 * narrow row drops under the words instead, with its state, so the name
 * is not squeezed into a column.
 */
export function ListItem(props: ListItemProps) {
  return (
    <View class="grow shrink basis-0 h-auto gap-2">
      <View class={`text-fg transition duration-317 w-full min-h-11 text-start py-2 px-3 rounded-control hover:bg-[#7379851d] hover:bg-none active:bg-[#73798532] active:bg-none data-selected:bg-rim-soft data-selected:bg-none data-selected:text-fg data-selected:shadow-[inset_3px_0px_0px_#e8e8ed] disabled:bg-transparent disabled:bg-none disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} role="button" label={props.title} description={props.subtitle} pressed={(props.selected ?? false)} busy={(props.loading ?? false)} disabled={(props.disabled ?? false) || (props.loading ?? false)} onPress={(e) => props.onChoose?.(e)}>
        <View class="flex-row flex-wrap items-center w-full gap-3">
          <Show when={(props.image ?? 0) || props.icon}>
            <View class={`bg-tile shadow-sunk justify-center items-center self-start gap-2 size-11 rounded-control`}>
              <Show when={(props.image ?? 0)} fallback={
                  <Icon class={`size-6`} name={props.icon ?? ""} />
              }>
                <Image class={`object-cover size-full data-muted:opacity-45`} src={String(props.image ?? 0)} />
              </Show>
            </View>
          </Show>
          <View class={`grow shrink basis-40 gap-0.5`}>
            <Text class={`font-semibold text-lg text-fg group-disabled/listitem:text-fg-subtle`}>
              {props.title}
            </Text>
            <Show when={props.subtitle}>
              <Text class={`text-caption text-fg-muted group-disabled/listitem:text-fg-subtle`}>
                {props.subtitle}
              </Text>
            </Show>
          </View>
          <Show when={props.value || props.badge || (props.loading ?? false) || (props.selected ?? false)}>
            <View class="flex-row items-center shrink ml-auto max-w-full gap-3">
              <Show when={props.value || props.badge}>
                <View class={`shrink items-end text-end gap-0.5`}>
                  <Show when={props.value}>
                    <Text class={`font-semibold tabular-nums text-fg`}>
                      {props.value}
                    </Text>
                  </Show>
                  <Show when={props.badge}>
                    <Text class={`text-caption tabular-nums bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c] text-fg transition duration-317 py-0 px-2 rounded-capsule`}>
                      {props.badge}
                    </Text>
                  </Show>
                </View>
              </Show>
              <Show when={(props.loading ?? false)} fallback={
                  <Show when={(props.selected ?? false)}>
                    <Icon class={`text-accent size-4 animate-pop`} name="check" />
                  </Show>
              }>
                <Spinner size={20} name={t("base.loading")} />
              </Show>
            </View>
          </Show>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ListItemProps>> = {
  default: {"title":"Ember Blade","subtitle":"Legendary sword · Equipped by Mira","icon":"sword","value":"1,250","badge":"New","selected":true},
  disabled: {"title":"Northern expedition","subtitle":"Reach level 12 to unlock this destination.","icon":"lock","disabled":true},
  loading: {"title":"Shared build","subtitle":"Retrieving equipment…","icon":"bag","loading":true},
  overflow: {"title":"The Ember Blade of the Last Watcher of the Ninefold Gate","subtitle":"A two-handed legendary sword with an unusually long description that must remain readable on a phone.","icon":"sword","value":"1,250,000 gold","badge":"Recommended","selected":true},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Choose this row (the enclosing collection's stable key)"} as const;
