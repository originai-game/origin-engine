import { Icon, Show, Text, View, createContext, createSignal, packClass, provide, useContext, type Signal, type UiEvent } from "@origingame/engine/ui";
import { Button } from "./button";

/** The current entry's key, shared by the entries of one MenuList. */
const Current = createContext<Signal<string> | null>(null);

export interface MenuListProps {
  /** What screen readers call it ("Main menu"). */
  label?: string | undefined;
  /** The arrows wrap from the last entry to the first (a pause menu); the default stops at the ends. */
  wrap?: boolean | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  /** Its `MenuEntry`s, and spacers between groups. */
  children?: any;
  [data: `data-${string}`]: unknown;
}

/**
 * A front-end menu (a main menu's, a pause menu's): a column of
 * `MenuEntry`s, one focus group, whose entries enter one after another
 * when it shows (`List.cascade`). One bar stands behind the current entry,
 * the one hovered or focused, and glides to the next one as the pointer or
 * the focus moves (`MenuList.indicator`). Every look and timing is the pack's.
 */
export function MenuList(props: MenuListProps) {
  const current = createSignal("");
  const data = Object.fromEntries(Object.entries(props).filter(([k]) => k.startsWith("data-")));
  return (
    <View {...data} role="menu" label={props.label} focusGroup={props.wrap ? "wrap" : "stop"} focusMemory class={`gap-y-0.5 ${packClass("List", "cascade")} ${props.class ?? ""}`}>
      {provide(Current, current, () => props.children)}
    </View>
  );
}

export interface MenuEntryProps {
  /** Its key in the list, unique among its entries (newGame); also the node name the probe finds it by. */
  id: string;
  /** Its words. */
  label: string;
  /** A line under the words (where the last save stands); empty: none. */
  line?: string | undefined;
  /** A lesser entry (Credits, Quit to menu): no chevron. */
  minor?: boolean | undefined;
  /** It ends something (Quit game): the bar behind it is the danger's. */
  danger?: boolean | undefined;
  /** Its words' size: `lg` (a main menu's, the default) or `md` (a pause menu's). */
  size?: "md" | "lg" | undefined;
  autofocus?: boolean | undefined;
  disabled?: boolean | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  onPress?: ((e: UiEvent) => void) | undefined;
}

/**
 * One entry of a `MenuList`: a built-in `Button` in the `menu` (or
 * `menu-danger`) variant that enters in its turn (`List.item`) and takes
 * the list's bar while hovered or focused.
 */
export function MenuEntry(props: MenuEntryProps) {
  const current = useContext(Current);
  const lit = () => current !== null && current() === props.id && props.disabled !== true;
  const take = () => {
    if (current !== null && props.disabled !== true) current.set(props.id);
  };
  return (
    <Button
      variant={props.danger ? "menu-danger" : "menu"}
      name={props.id}
      class={`group/entry ${packClass("List", "item")} ${props.class ?? ""}`}
      data-current={lit()}
      autofocus={props.autofocus}
      disabled={props.disabled}
      onPress={(e) => props.onPress?.(e)}
      onFocus={take}
      onPointerMove={take}
    >
      <Show when={lit()}>
        <View class={packClass("MenuList", "indicator", props.danger ? "danger" : undefined)} />
      </Show>
      <View class="relative grow shrink">
        <Text class={`text-start leading-tight font-semibold text-shadow-xs text-shadow-bg ${props.size === "md" ? "text-title compact:text-xl" : "text-3xl"}`}>{props.label}</Text>
        <Show when={props.line}>
          <Text class="text-start text-base font-medium opacity-quiet whitespace-nowrap overflow-hidden">{props.line}</Text>
        </Show>
      </View>
      <Show when={!props.minor}>
        <Icon class="relative opacity-0 size-4 transition-opacity duration-control group-data-current/entry:opacity-100 rtl:-scale-x-100 rtl:scale-y-100" name="chevronRight" />
      </Show>
    </Button>
  );
}

const press = () => {};

/** The gallery mounts it over the key art, where a front end puts it. */
export const ground = "scene";

export const fixtures: Record<string, Partial<MenuListProps>> = {
  default: {
    label: "Main menu",
    children: () => [
      <MenuEntry id="continue" label="Continue" line="Chapter 2 · 1:24" autofocus onPress={press} />,
      <MenuEntry id="newGame" label="New game" onPress={press} />,
      <MenuEntry id="settings" label="Settings" onPress={press} />,
      <View class="h-4" />,
      <MenuEntry id="quit" label="Quit game" minor danger onPress={press} />,
    ],
  },
};
