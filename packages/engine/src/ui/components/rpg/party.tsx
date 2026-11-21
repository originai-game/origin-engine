import { For, Icon, Image, Show, Text, View, createMemo, formatNumber, t, type UiEvent } from "@origingame/engine/ui";
import "./party.en.ogt";
import "../hud/bar.en.ogt";

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
 * name, and a health and a mana bar with their figures (the HudBar block,
 * inlined: hud/bar.svelte is not ported yet), filled in the page's health
 * and mana colours. Reports choose (a member's key). The cards share the
 * width evenly.
 */

/**
 * One HudBar block: a dark well with a lit rim, the fill in its kind's
 * colour and the figure upright over it. The skew stands in as the flat
 * bar; the gloss as the fill's top highlight.
 */
function PartyBar(props: { value: number; max: number; kind: string; name: string; height: number }) {
  const fill = (v: number) => (props.max <= 0 || v <= 0 ? 0 : v >= props.max ? 1 : v / props.max);
  return (
    <View class={`relative w-full overflow-hidden bg-[#04060cb8] shadow-[0px_0px_0px_1px_#ffffff38,inset_0px_1px_0px_#ffffff24,0px_2px_4px_#00000073]`} role="meter" label={props.name} valueMin={0} valueMax={props.max > 0 ? props.max : 1} valueNow={props.value <= 0 ? 0 : props.max > 0 && props.value >= props.max ? props.max : props.value} style={{ height: `${props.height}px` }}>
      <View class={`absolute inset-y-0 left-0 shadow-[inset_0px_2px_0px_#ffffff8c] data-health:bg-health data-mana:bg-mana`} data-health={props.kind === "health"} data-mana={props.kind === "mana"} style={{ width: `${fill(props.value) * 100}%` }} />
      <View class="absolute inset-0 flex-row items-center justify-center px-1">
        <Text class="text-[0.857rem] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs text-shadow-bg/55">
          {t("hud.bar.figure", { now: props.value, max: props.max })}
        </Text>
      </View>
    </View>
  );
}

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
                <PartyBar value={m().health} max={m().healthMax} kind="health" name={t("rpg.party.health")} height={24} />
              </View>
              <View class="mx-1">
                <PartyBar value={m().mana} max={m().manaMax} kind="mana" name={t("rpg.party.mana")} height={24} />
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
