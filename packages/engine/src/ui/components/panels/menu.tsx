import { For, Icon, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import "./menu.en.ogt";

/** One entry row: dim type on the pack's row, lit by the hover ground and its leading edge under the pointer, the keys or the focus; a disabled row faint and shadowless. */
const row =
  `group/row transition duration-317 w-full h-auto min-h-9.5 p-0 pointer-coarse:min-h-11 text-fg-muted hover:bg-surface-hover hover:text-fg hover:inset-shadow-bar hover:inset-shadow-hud-accent focus-visible:bg-surface-hover focus-visible:text-fg focus-visible:inset-shadow-bar focus-visible:inset-shadow-hud-accent active:bg-surface-hover active:text-fg active:inset-shadow-bar active:inset-shadow-hud-accent disabled:bg-transparent disabled:bg-none disabled:shadow-none disabled:text-fg-subtle data-danger:text-danger data-danger:hover:text-danger data-danger:focus-visible:text-danger disabled:data-danger:text-fg-subtle data-down:opacity-45 ${packClass("Row", "frame")}`;

export interface MenuProps {
  /** Its entries, submenus' too, a keyed collection of { id, label, kind, icon, shortcut, checked, danger, disabled, parent, more }: kind item, check, radio, separator or header; icon an icon name before the label (empty: none); shortcut shown at the end (Ctrl+S); checked a check or radio entry is on; danger a destructive command in the danger ink; disabled shown but inert; parent the id of the entry whose submenu it is in (empty: the top level); more it opens a submenu (a chevron at its end). */
  entries: ReadonlyArray<{ id: string; label: string; kind: string; icon: string; shortcut: string; checked: boolean; danger: boolean; disabled: boolean; parent: string; more: boolean }>;
  /** The id of the entry whose submenu shows, its rows in place of the menu's under a back row (set it to an entry with more when choose names that entry; back clears it); empty: the top level. */
  submenu: string;
  /** What a screen reader calls it; empty is Menu. */
  label: string;
  /** The back event, with the row's key where it comes from a row. */
  onBack?: (e: UiEvent, key?: string) => void;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
}

/**
 * A menu: a dense smoked plate with a lit top edge, its rows dim type that
 * wash in the accent from a lit leading edge under the pointer, the keys
 * or the focus, a shortcut on a dark key chip, a check drawn in the
 * accent, the chosen radio a lit diamond, a destructive command in the
 * danger ink and a disabled row faint. A submenu's rows show in place of
 * the menu's under a back row, and a header or a separator parts the
 * entries. Reports choose (an entry's key) and back.
 *
 * Dropped against the .svelte: the CJK heading size (a lang has no class
 * variant here; the engine's text scale carries it).
 */
export function Menu(props: MenuProps) {
  return (
    <View class={`pt-1 pb-2 bg-linear-to-b from-raised-pressed to-glass-lo shadow-overlay inset-shadow-lit inset-ring-1 inset-ring-rim text-fg min-w-60 max-w-full px-0 m-0 gap-0 ${packClass("Panel", "frame")}`} role="menu" label={props.label || t("component.menu.name")}>
      <Show when={props.submenu !== ""}>
        <View class={row} role="menuitem" label={t("component.menu.back")} onPress={(e) => props.onBack?.(e)}>
          <View class="flex-row items-center w-full gap-2">
            <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100" name="chevronLeft" />
            <Text class="grow shrink text-start text-base leading-snug font-semibold">
              {t("component.menu.back")}
            </Text>
          </View>
        </View>
        <View class="w-auto self-stretch h-px bg-rim my-1 mx-4" role="separator" />
      </Show>
      <For each={props.entries}>
        {(e) => (
          <Show when={e().parent === props.submenu}>
            <Show when={e().kind === "separator"} fallback={
              <Show when={e().kind === "header"} fallback={
                <View
                  class={row}
                  role={e().kind === "check" ? "menuitemcheckbox" : e().kind === "radio" ? "menuitemradio" : "menuitem"}
                  checked={e().kind === "check" || e().kind === "radio" ? e().checked : undefined}
                  label={e().label}
                  disabled={e().disabled}
                  onPress={(ev) => props.onChoose?.(ev, e().id)}
                  data-danger={e().danger}
                >
                  <View class="flex-row items-center w-full gap-2">
                    <View class="flex-row items-center justify-center gap-0 size-4.5">
                      <Show when={e().icon && e().kind !== "check" && e().kind !== "radio"}>
                        <Icon class="size-full" name={e().icon} />
                      </Show>
                      <Show when={e().kind === "check" && e().checked}>
                        <Icon name="check" class="text-hud-accent-ink size-3/4" />
                      </Show>
                      <Show when={e().kind === "radio" && e().checked}>
                        <View class="bg-hud-accent rotate-45 size-9/20" />
                      </Show>
                    </View>
                    <Text class="grow shrink text-start text-base leading-snug font-semibold break-words">
                      {e().label}
                    </Text>
                    <Show when={e().shortcut}>
                      <Text class={`font-bold text-xs leading-snug text-fg-muted tracking-sm whitespace-nowrap bg-well inset-ring-1 inset-ring-rim py-0.5 px-1 group-disabled/row:text-fg-subtle ${packClass("Chip", "frame")}`}>
                        {e().shortcut}
                      </Text>
                    </Show>
                    <Show when={e().more}>
                      <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100" name="chevronRight" />
                    </Show>
                  </View>
                </View>
              }>
                <Text class="tracking-label uppercase text-fg-subtle pt-2 pb-0.5 text-xs font-bold px-4">
                  {e().label}
                </Text>
              </Show>
            }>
              <View class="w-auto self-stretch h-px bg-rim my-1 mx-4" role="separator" />
            </Show>
          </Show>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<MenuProps>> = {
  default: {"submenu":"","label":"","entries":[{"id":"h","label":"Item","kind":"header","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"copy","label":"Copy","kind":"item","icon":"plus","shortcut":"Ctrl+C","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"compare","label":"Compare","kind":"check","icon":"","shortcut":"C","checked":true,"danger":false,"disabled":false,"parent":"","more":false},{"id":"sort","label":"Sort by","kind":"item","icon":"flag","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":true},{"id":"sell","label":"Sell","kind":"item","icon":"coin","shortcut":"","checked":false,"danger":false,"disabled":true,"parent":"","more":false},{"id":"s1","label":"","kind":"separator","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"destroy","label":"Destroy","kind":"item","icon":"bomb","shortcut":"Del","checked":false,"danger":true,"disabled":false,"parent":"","more":false},{"id":"name","label":"Name","kind":"radio","icon":"","shortcut":"","checked":true,"danger":false,"disabled":false,"parent":"sort","more":false},{"id":"value","label":"Value","kind":"radio","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"sort","more":false}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Choose","back":"Back"} as const;
