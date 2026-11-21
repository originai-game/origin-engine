import { For, Icon, Scroll, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import "./shooter.en.ogt";

export interface LoadoutCategory {
  id: string;
  /** An empty label takes the kit's string for the id (shooter.loadout.category.<id>). */
  label?: string;
}

export interface LoadoutSlot {
  id: string;
  /** A category's id. */
  category: string;
  /** What it is. */
  name: string;
  /** An icon of the style's atlas. */
  icon?: string;
  /** A short line under the name (ammunition, damage). */
  detail?: string;
  /** It is carried now. */
  equipped?: boolean;
  /** It cannot be chosen yet (shown with its cost). */
  locked?: boolean;
  /** Its cost text; empty is none. */
  cost?: string;
}

export interface LoadoutProps {
  /** The page's heading; empty takes the kit's (shooter.loadout.title, Loadout). */
  title?: string;
  /** The slot groups in order. */
  categories?: ReadonlyArray<LoadoutCategory>;
  /** What can be carried. */
  slots?: ReadonlyArray<LoadoutSlot>;
  /** The slot id selected now (empty: none). */
  selected?: string;
  /** Choose a slot. */
  onSelect?: (e: UiEvent, id: string) => void;
  /** Equip. */
  onEquip?: (e: UiEvent, id: string) => void;
  /** Back. */
  onBack?: (e: UiEvent) => void;
}

/**
 * A loadout screen over the scene: the title in large condensed capitals,
 * a category's name over a rule, each item a cut plate of smoked glass
 * (the chosen one ringed in the accent and washed in it), Equipped on a
 * green tag, the price in gold; names wrap on whole words. Approximations:
 * the chosen wash is flat and keeps no rim, the Equipped tag's sheen is
 * dropped, the scroll fade at the list's foot is dropped, and the
 * safe-area padding and short-screen reflow are the caller's.
 */
export function Loadout(props: LoadoutProps) {
  const heading = () => (props.title ?? "") != "" ? props.title : t("shooter.loadout.title");
  const selected = () => (props.slots ?? []).find((s) => s.id == props.selected);
  const categoryLabel = (c: LoadoutCategory) => (c.label ?? "") != "" ? c.label : t(`shooter.loadout.category.${c.id}`);
  const KEY = "shrink-0 size-6 min-w-6 text-center text-[0.929rem] leading-[1.6] font-bold text-[#262b38] bg-linear-to-b from-white to-[#d8dbe2] rounded-[0.286rem] shadow-[0px_0.143rem_0px_#0000008c,inset_0px_-0.143rem_0px_#0000002e] text-shadow-none pointer-coarse:hidden";
  const BUTTON = `flex-row items-center justify-center gap-x-1 min-h-11 px-4 min-w-[7.5rem] text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg nine-plate-cut bg-raised-lo hover:bg-raised-hover-lo disabled:opacity-45 compact:flex-1 ${packClass("Button", "plate")}`;
  return (
    <View class="flex-col w-full h-full max-w-full gap-y-4 text-fg" role="group" label={heading()}>
      <Text class="max-w-full text-[3.286rem] leading-none font-bold uppercase tracking-[0.04em] text-fg shadow-[0px_0.143rem_0.571rem_rgb(0_0_0/0.6)] compact:text-[2rem]">{heading()}</Text>
      <View class="flex-1 min-h-0 flex-col gap-y-4">
        <Scroll class="flex-1 min-h-0 w-full pr-4">
          <View class="flex-row flex-wrap gap-x-4 gap-y-4 content-start">
          <For each={props.categories ?? []} key={(c) => c.id}>
            {(c) => (
              <View class="group/cat basis-[18rem] grow min-w-0 flex-col" label={categoryLabel(c())}>
                <Text class="w-full pb-1 border-b border-white/30 text-[0.857rem] leading-[1.3] font-semibold tracking-[0.14em] uppercase text-fg">{categoryLabel(c())}</Text>
                <View class="flex-col gap-y-1 pt-2">
                  <For each={(props.slots ?? []).filter((s) => s.category == c().id)} key={(s) => s.id}>
                    {(s) => (
                      <View
                        class={`group/item flex-row items-center gap-x-3 w-full min-h-11 py-2 px-3 nine-plate-cut bg-entry-lo hover:bg-entry-hi data-selected:bg-[#38e8ff26] disabled:opacity-100 disabled:text-fg/66 transition duration-317`}
                        role="button"
                        label={s().name}
                        pressed={s().id == props.selected}
                        disabled={s().locked}
                        onPress={(e) => props.onSelect?.(e, s().id)}
                        data-selected={s().id == props.selected}
                        data-locked={s().locked === true}
                      >
                        <Icon class="shrink-0 size-6 text-fg group-data-locked/item:text-fg/66" name={s().locked ? "lock" : s().icon ?? ""} />
                        <View class="flex-1 min-w-0 flex-col items-start gap-y-[0.071rem]">
                          <Text class="w-full text-start text-[1.071rem] leading-[1.15] font-bold text-fg">{s().name}</Text>
                          <Show when={(s().detail ?? "") != "" || s().equipped}>
                            <View class="flex-row flex-wrap items-center gap-x-1 gap-y-[0.071rem]">
                              <Show when={(s().detail ?? "") != ""}>
                                <Text class="text-start text-[0.929rem] font-semibold whitespace-nowrap text-fg/66">{s().detail}</Text>
                              </Show>
                              <Show when={s().equipped}>
                                <Text class={`shrink py-0 px-1 text-[0.857rem] leading-[1.5] font-bold tracking-[0.14em] uppercase text-bg nine-chip-cut bg-success ${packClass("Chip", "frame")}`}>
                                  {t("shooter.loadout.equipped")}
                                </Text>
                              </Show>
                            </View>
                          </Show>
                        </View>
                        <Show when={(s().cost ?? "") != ""}>
                          <View class="shrink-0 flex-row items-center gap-x-1">
                            <Icon class="shrink-0 size-4 text-gold group-data-locked/item:text-fg/66" name="coin" />
                            <Text class="text-[1.143rem] font-bold tabular-nums whitespace-nowrap text-gold group-data-locked/item:text-fg/66">{s().cost}</Text>
                          </View>
                        </Show>
                      </View>
                    )}
                  </For>
                </View>
              </View>
            )}
          </For>
          <Show when={(props.slots ?? []).length == 0}>
            <Text class="text-[1.143rem] text-fg/66">{t("shooter.loadout.empty")}</Text>
          </Show>
          </View>
        </Scroll>
        <View class="flex-row flex-wrap justify-end gap-x-2 gap-y-2">
          <View class={BUTTON} role="button" label={t("shooter.loadout.back")} onPress={(e) => props.onBack?.(e)}>
            <Text class={KEY} glyph="back" />
            <Text class="text-fg">{t("shooter.loadout.back")}</Text>
          </View>
          <Show when={selected()}>
            <View
              class={`flex-row items-center justify-center gap-x-1 min-h-11 px-4 min-w-[7.5rem] text-[0.929rem] font-bold tracking-[0.14em] uppercase text-accent-fg nine-plate-cut bg-accent disabled:opacity-45 compact:flex-1 ${packClass("Button", "plate", "primary")}`}
              role="button"
              label={t("shooter.loadout.equip")}
              disabled={selected()?.locked || selected()?.equipped}
              onPress={(e) => props.onEquip?.(e, props.selected ?? "")}
            >
              <Text class={KEY} glyph="details" />
              <Text class="text-accent-fg">{t("shooter.loadout.equip")}</Text>
            </View>
          </Show>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<LoadoutProps>> = {
  default: {"title":"","selected":"strider","categories":[{"id":"primary","label":""},{"id":"secondary","label":""},{"id":"melee","label":""}],"slots":[{"id":"strider","category":"primary","name":"Strider AR","icon":"sword","detail":"25 / 75","equipped":true,"locked":false,"cost":"2,900"},{"id":"halcyon","category":"primary","name":"Halcyon","icon":"sword","detail":"30 / 90","equipped":false,"locked":false,"cost":"2,900"},{"id":"pike","category":"primary","name":"Pike .50","icon":"sword","detail":"5 / 10","equipped":false,"locked":true,"cost":"4,700"},{"id":"wren","category":"secondary","name":"Wren Pistol","icon":"bolt","detail":"15 / 45","equipped":true,"locked":false,"cost":"500"},{"id":"knife","category":"melee","name":"Field Knife","icon":"sword","detail":"","equipped":true,"locked":false,"cost":""}]},
};
