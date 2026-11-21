import { For, Icon, Image, Show, Text, View, formatNumber, t, type UiEvent } from "@origingame/engine/ui";
import { HudBar } from "../hud/bar";
import "./party.en.ogt";

export interface RpgPartyProps {
  /** Keyed rows of { id, name, portrait, level, health, healthMax, mana, manaMax, down }: portrait a texture (og.load.texture), 0 none; down greys the card and marks it. */
  members?: ReadonlyArray<{ id: string; name: string; portrait: number; level: number; health: number; healthMax: number; mana: number; manaMax: number; down: boolean }>;
  /** The member chosen; empty is none. */
  selected: string;
  /** A wide but short place (a laptop window): wider, shorter portraits. */
  dense: boolean;
  /** Choose (a member's key). */
  onChoose?: (e: UiEvent, id: string) => void;
}

/**
 * The party as a row of portrait cards: a level seal, the portrait, the
 * name, and a health and a mana HudBar with their figures, filled in the
 * page's health and mana colours. Reports choose (a member's key). The
 * cards share the width evenly.
 */

export function RpgParty(props: RpgPartyProps) {
  return (
    <View class={`group/party flex-col w-full text-fg text-shadow-xs text-shadow-bg/55`} role="group" label={t("rpg.party.name")} data-dense={props.dense}>
      <Show when={(props.members ?? []).length} fallback={
          <View class="text-fg-muted" role="status">
            <Text>{t("rpg.party.empty")}</Text>
          </View>
      }>
        <View class="flex-row w-full" role="listbox" label={t("rpg.party.name")}>
          <For each={props.members ?? []} key={(m) => m.id}>
            {(m) => (
            <View class="relative flex-1 min-w-0 flex-col gap-y-px p-1" role="option" label={m().name} selected={m().id === props.selected} data-selected={m().id === props.selected} data-down={m().down} onPress={(e) => props.onChoose?.(e, m().id)}>
              <View class="relative w-full aspect-[1.5] compact:aspect-[1.15] group-data-dense/party:aspect-[2] overflow-hidden bg-tile">
                <Show when={m().portrait}>
                  <Image class="absolute inset-0 size-full" src={String(m().portrait)} />
                </Show>
              </View>
              <View class="absolute left-1 top-1 w-8 min-h-8 flex-col items-center justify-center py-0 px-1 bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c] rounded-capsule" label={m().down ? t("rpg.party.down") : t("rpg.party.level", { level: m().level })}>
                <Show when={!m().down} fallback={
                    <Icon class="size-4 text-fg" name="skull" />
                }>
                  <Text class="text-[0.857rem] font-bold text-fg">{t("rpg.party.lv")}</Text>
                  <Text class="text-[0.857rem] font-bold tabular-nums text-fg">{formatNumber(m().level)}</Text>
                </Show>
              </View>
              <Text class="py-[0.143rem] px-1 text-center whitespace-nowrap text-ellipsis overflow-hidden text-fg">
                {m().name}
              </Text>
              <View class="mx-1">
                <HudBar value={m().health} max={m().healthMax} showValue />
              </View>
              <View class="mx-1">
                <HudBar value={m().mana} max={m().manaMax} kind="mana" showValue />
              </View>
            </View>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgPartyProps>> = {
  default: {"members":[{"id":"a","name":"Aerin","portrait":0,"level":8,"health":62,"healthMax":72,"mana":48,"manaMax":72,"down":false},{"id":"l","name":"Lys","portrait":0,"level":8,"health":0,"healthMax":60,"mana":0,"manaMax":80,"down":true}],"selected":"a"},
  empty: {"members":[],"selected":""},
};
