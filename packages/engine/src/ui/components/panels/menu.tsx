import { For, Icon, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./menu.en.ogt";

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

export function Menu(props: MenuProps) {
  return (
    <View class={`pt-1 pb-2 bg-linear-to-b from-[#171f30f5] to-[#080c16f7] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c,0px_20.576px_45.712px_#00000099] text-accent-hover min-w-[17.143rem] px-0 m-0 gap-0`} role="menu" label={props.label || t("component.menu.name")}>
      <Show when={props.submenu !== ""}>
        <View class={`text-accent-hover/74 transition duration-317 w-full h-auto min-h-[2.714rem] p-0 hover:bg-transparent hover:bg-linear-to-r hover:from-[#38e8ff3d] hover:via-[#38e8ff14] hover:via-55% hover:to-[#38e8ff08] hover:text-accent-hover hover:shadow-[inset_3.424px_0px_0px_#38e8ff,inset_0px_1px_0px_#ffffff2e] active:bg-transparent active:bg-linear-to-r active:from-[#38e8ff57] active:via-[#38e8ff1f] active:via-55% active:to-transparent active:text-accent-hover active:shadow-[inset_3.424px_0px_0px_#38e8ff,inset_0px_1px_0px_#ffffff2e] disabled:bg-transparent disabled:bg-none disabled:text-accent-hover/32 disabled:shadow-none data-down:opacity-45 pointer-coarse:min-h-11`} role="menuitem" onPress={(e) => props.onBack?.(e)}>
          <View class="flex-row items-center w-full gap-2">
            <Icon class={`size-[1.143rem] rtl:scale-x--100 rtl:scale-y-100`} name="chevronLeft" />
            <Text class="grow shrink text-start text-[1.143rem] leading-snug font-semibold">
              {t("component.menu.back")}
            </Text>
          </View>
        </View>
        <View class={`w-auto self-stretch text-sheen my-1 mx-4`} />
      </Show>
      <For each={props.entries}>
        {(e) => (
        <Show when={e().parent === props.submenu}>
          <View class="w-full gap-0">
            <Show when={e().kind === "separator"} fallback={
                <Show when={e().kind === "header"} fallback={
                    <View class={`group/row text-accent-hover/74 transition duration-317 w-full h-auto min-h-[2.714rem] p-0 hover:bg-transparent hover:bg-linear-to-r hover:from-[#38e8ff3d] hover:via-[#38e8ff14] hover:via-55% hover:to-[#38e8ff08] hover:text-accent-hover hover:shadow-[inset_3.424px_0px_0px_#38e8ff,inset_0px_1px_0px_#ffffff2e] active:bg-transparent active:bg-linear-to-r active:from-[#38e8ff57] active:via-[#38e8ff1f] active:via-55% active:to-transparent active:text-accent-hover active:shadow-[inset_3.424px_0px_0px_#38e8ff,inset_0px_1px_0px_#ffffff2e] disabled:bg-transparent disabled:bg-none disabled:text-accent-hover/32 disabled:shadow-none data-down:opacity-45 pointer-coarse:min-h-11 data-danger:hover:text-[#ff8076] data-danger:disabled:text-accent-hover/32`} role={e().kind === "check" ? "menuitemcheckbox" : e().kind === "radio" ? "menuitemradio" : "menuitem"} checked={e().kind === "check" || e().kind === "radio" ? e().checked : null} label={e().label} disabled={e().disabled} onPress={(ev) => props.onChoose?.(ev, e().id)} data-danger={e().danger} data-tone-danger={(e().danger && !e().disabled ? 'danger' : '') === "danger"}>
                      <View class="flex-row items-center w-full gap-2">
                        <View class="justify-center items-center gap-0 size-[1.286rem]">
                          <Show when={e().icon && e().kind !== "check" && e().kind !== "radio"}>
                            <Icon class={`size-full`} name={e().icon} />
                          </Show>
                          <Show when={e().kind === "check" && e().checked}>
                            <Icon name="check" class="text-[#38e8ff] size-3/4" />
                          </Show>
                          <Show when={e().kind === "radio" && e().checked}>
                            <Text class={`bg-[#38e8ff] text-fg rotate-45 size-[45%]`} />
                          </Show>
                        </View>
                        <Text class="grow shrink text-start text-[1.143rem] leading-snug font-semibold">
                          {e().label}
                        </Text>
                        <Show when={e().shortcut}>
                          <Text class={`font-bold text-[0.857rem] leading-snug text-accent-hover/74 tracking-sm whitespace-nowrap bg-[#060910e0] shadow-[inset_0px_0px_0px_1px_#ffffff29] py-0.5 px-1 group-disabled/row:text-accent-hover/32`}>
                            {e().shortcut}
                          </Text>
                        </Show>
                        <Show when={e().more}>
                          <Icon class={`size-[1.143rem] rtl:scale-x--100 rtl:scale-y-100`} name="chevronRight" />
                        </Show>
                      </View>
                    </View>
                }>
                  <Text class={`tracking-[0.14em] uppercase text-accent-hover/56 pt-2 pb-0.5 text-[0.857rem] font-bold px-4`}>
                    {e().label}
                  </Text>
                </Show>
            }>
              <View class={`w-auto self-stretch text-sheen my-1 mx-4`} />
            </Show>
          </View>
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
