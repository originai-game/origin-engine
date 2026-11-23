import { Icon, Show, Text, View, createContext, packClass, provide, useContext, type UiEvent } from "@origingame/engine/ui";
import { Button } from "./button";

let made = 0;

/** What an item asks of the menu it is in: to close once it is chosen. */
const MenuContext = createContext<((e: UiEvent) => void) | null>(null);

export interface MenuProps {
  /** Shown or not. The game owns it: a press on the trigger reports `onOpenChange` with the next value; choosing an item, a press outside and `back` report false. */
  open: boolean;
  /** Where it opens by its trigger: `bottom` (the default) or `top`. */
  placement?: "bottom" | "top" | undefined;
  /** What screen readers call it ("Account"). */
  label?: string | undefined;
  /** Classes of the document that uses it, on its popup. */
  class?: string | undefined;
  onOpenChange?: ((e: UiEvent, open: boolean) => void) | undefined;
  /** What opens it: a control its presses go to (a `Button`, a hero's portrait). */
  trigger?: any;
  /** Its `MenuItem`s, `MenuSeparator`s and `MenuLabel`s, arranged freely. */
  children?: any;
}

/**
 * A list of actions its trigger opens (NSH base's `Menu`; a dropdown menu
 * on a button): the trigger says it is expanded, and the popup follows
 * the trigger's box, shifted inside the screen. It takes the focus while
 * open, so the arrows move among its items (one focus group that wraps)
 * and closing gives the focus back to the trigger; choosing an item, a
 * press outside it and `back` close it. The pack draws the popup
 * (`Menu.frame`), which pops in by its own `starting:`.
 */
export function Menu(props: MenuProps) {
  const name = `og-menu-${++made}`;
  const top = () => props.placement === "top";
  const close = (e: UiEvent) => props.onOpenChange?.(e, false);
  return (
    <View class="relative self-start">
      <View name={`${name}-trigger`} expanded={props.open} onPress={(e) => props.onOpenChange?.(e, !props.open)}>
        {props.trigger}
      </View>
      <MenuPopup name={name} open={props.open} top={top()} label={props.label} class={props.class} onClose={close}>
        {props.children}
      </MenuPopup>
    </View>
  );
}

export interface ContextMenuProps {
  /** Shown or not. The game owns it: a right click, a finger held, Shift+F10 or the Menu key on the trigger reports `onOpenChange` with true; choosing an item, a press outside and `back` report false. */
  open: boolean;
  label?: string | undefined;
  /** Classes of the document that uses it, on its popup. */
  class?: string | undefined;
  onOpenChange?: ((e: UiEvent, open: boolean) => void) | undefined;
  /** What it is the menu of (an item's slot, a unit's card); its own presses keep working. */
  trigger?: any;
  children?: any;
}

/**
 * The same menu opened by the secondary button (a right click), a finger
 * held on its trigger, or Shift+F10 and the Menu key while the focus is
 * in it (NSH base's `ContextMenu`), under the trigger at its start, the
 * focus on its first item; the trigger's own presses keep working. A
 * game that opens it otherwise sets `open` itself.
 */
export function ContextMenu(props: ContextMenuProps) {
  const name = `og-menu-${++made}`;
  const close = (e: UiEvent) => props.onOpenChange?.(e, false);
  return (
    <View class="relative self-start">
      <View name={`${name}-trigger`} expanded={props.open} onLongPress={(e) => props.onOpenChange?.(e, true)} onContextMenu={(e) => props.onOpenChange?.(e, true)}>
        {props.trigger}
      </View>
      <MenuPopup name={name} open={props.open} top={false} label={props.label} class={props.class} onClose={close}>
        {props.children}
      </MenuPopup>
    </View>
  );
}

/** The popup both menus share: a press shade outside it, the popup on its trigger. */
function MenuPopup(props: { name: string; open: boolean; top: boolean; label?: string | undefined; class?: string | undefined; onClose: (e: UiEvent) => void; children?: any }) {
  return (
    <Show when={props.open}>
      {/* Takes every press outside the popup, and draws nothing. */}
      <View class="absolute" spotlight={props.name} onPress={props.onClose} />
      <View
        role="menu"
        name={props.name}
        label={props.label}
        modal={true}
        focusGroup="wrap"
        anchor={`${props.name}-trigger`}
        lift={[0, props.top ? 0 : 1, 0]}
        clamp="edge"
        clampMargin={8}
        class={`z-40 ${props.top ? "-translate-y-full -mt-1" : "mt-1"} ${packClass("Menu", "frame")} ${props.class ?? ""}`}
        onAction={{ back: props.onClose }}
      >
        {provide(MenuContext, props.onClose, () => props.children)}
      </View>
    </Show>
  );
}

export interface MenuItemProps {
  /** It was chosen: a press, Enter or Space. The menu closes unless `closeOnSelect` is false. */
  onSelect?: ((e: UiEvent) => void) | undefined;
  disabled?: boolean | undefined;
  /** Checkable: a setting it shows (`menuitemcheckbox`), or with `radio` one choice of a group (`menuitemradio`). The game owns the value; `checked:` styles apply. */
  checked?: boolean | undefined;
  radio?: boolean | undefined;
  /** An action that destroys or cannot be undone: the pack's `Menu.item.danger`. */
  danger?: boolean | undefined;
  /** An icon name before its text. */
  icon?: string | undefined;
  /** A hint after its text: a key or a count. */
  shortcut?: string | undefined;
  /** Keeps the menu open once chosen, so a setting shows its new value at once. */
  closeOnSelect?: boolean | undefined;
  label?: string | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  children?: any;
}

/**
 * An item of a menu (NSH base's `MenuItem`): a focus stop of its menu
 * that reports `onSelect` and closes it. The pack draws it (`Menu.item`,
 * `Menu.item.danger`), the check before a checkable item's text
 * (`Menu.indicator`, shown by its `checked:` classes) and its hint
 * (`Menu.shortcut`).
 */
export function MenuItem(props: MenuItemProps) {
  const close = useContext(MenuContext);
  const checkable = () => props.checked !== undefined;
  return (
    <View
      role={!checkable() ? "menuitem" : props.radio ? "menuitemradio" : "menuitemcheckbox"}
      tabIndex={0}
      checked={checkable() ? props.checked : undefined}
      disabled={props.disabled}
      label={props.label}
      class={`${props.danger ? packClass("Menu", "item", "danger") : packClass("Menu", "item")} ${props.class ?? ""}`}
      onPress={(e) => {
        props.onSelect?.(e);
        if (props.closeOnSelect !== false) close?.(e);
      }}
    >
      <Show when={checkable()}>
        <View class={packClass("Menu", "indicator")} checked={props.checked}>
          <Show when={props.radio} fallback={<Icon name="check" class="size-full" />}>
            <View class="size-2 rounded-capsule bg-current" />
          </Show>
        </View>
      </Show>
      <Show when={props.icon}>
        <Icon name={props.icon!} class="size-4.5 shrink-0" />
      </Show>
      <View class="flex-row items-center grow min-w-0 gap-2">{props.children}</View>
      <Show when={props.shortcut}>
        <Text class={packClass("Menu", "shortcut")}>{props.shortcut}</Text>
      </Show>
    </View>
  );
}

/** A line between groups of a menu's items (`Menu.separator`), never a focus stop. */
export function MenuSeparator(props: { class?: string | undefined }) {
  return <View role="separator" class={`${packClass("Menu", "separator")} ${props.class ?? ""}`} />;
}

/** A group's heading in a menu (`Menu.label`), never a focus stop. */
export function MenuLabel(props: { class?: string | undefined; children?: any }) {
  return <Text class={`${packClass("Menu", "label")} ${props.class ?? ""}`}>{props.children}</Text>;
}

const press = () => {};

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

const items = () => [
  <MenuLabel>Hero</MenuLabel>,
  <MenuItem icon="star" onSelect={press} shortcut="E"><Text>Equip</Text></MenuItem>,
  <MenuItem icon="gem" onSelect={press}><Text>Upgrade</Text></MenuItem>,
  <MenuItem icon="lock" disabled onSelect={press}><Text>Lock</Text></MenuItem>,
  <MenuItem checked={true} closeOnSelect={false} onSelect={press}><Text>Show stats</Text></MenuItem>,
  <MenuSeparator />,
  <MenuItem icon="close" danger onSelect={press}><Text>Dismiss</Text></MenuItem>,
];

export const fixtures: Record<string, Partial<MenuProps>> = {
  default: {
    open: true,
    label: "Hero actions",
    onOpenChange: press,
    trigger: () => <Button variant="outline" size="sm"><Text>Actions</Text><Icon name="chevronDown" class="size-4" /></Button>,
    children: items,
  },
  closed: {
    open: false,
    label: "Hero actions",
    onOpenChange: press,
    trigger: () => <Button variant="outline" size="sm"><Text>Actions</Text><Icon name="chevronDown" class="size-4" /></Button>,
    children: items,
  },
};
