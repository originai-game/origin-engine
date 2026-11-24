import { For, Scroll, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import "./combat-log.en.ogt";

export interface CombatLogLine {
  id: string;
  /** The line as the game words it (Sir Aldric uses Shield Bash on Cultist.). */
  text: string;
  /** damage, heal, status, crit or plain inks the line. */
  tone?: string;
  /** A quieter second line (→ 42 damage), empty for none. */
  detail?: string;
}

export interface CombatLogProps {
  /** The log, oldest first. */
  lines?: ReadonlyArray<CombatLogLine>;
}

/**
 * The combat log: a small band over a scroll of lines, the newest at the
 * foot, each inked by its tone (damage red, heal green, crit warn; status
 * reads plain, as the style's tone table words it). It scrolls inside the
 * height it is given and fades at its foot. Read only.
 */
export function RpgCombatLog(props: CombatLogProps) {
  const tone = (t?: string) => t === "damage" ? "text-danger" : t === "heal" ? "text-success" : t === "crit" ? "text-warn" : "text-fg";
  return (
    <View class={`flex-col w-full h-full min-h-0 overflow-hidden ${packClass("Plate", "frame")}`} role="log" label={t("rpg.log.name")}>
      <View class="flex-row shrink-0 h-8 items-center px-3">
        <Text class="text-sm font-semibold whitespace-nowrap text-fg text-shadow-xs">{t("rpg.log.name")}</Text>
      </View>
      <View class="relative flex-1 min-h-0">
        <Scroll class="absolute inset-0 pb-4">
          <For each={props.lines ?? []} key={(l) => l.id}>
            {(l) => (
              <View
                class={`group/line flex-col gap-y-0.5 py-1 px-2 ${tone(l().tone)}`}
                data-tone-damage={l().tone === "damage"}
                data-tone-heal={l().tone === "heal"}
                data-tone-crit={l().tone === "crit"}
              >
                <Text class={`text-caption group-data-tone-damage/line:text-danger group-data-tone-heal/line:text-success group-data-tone-crit/line:text-warn text-shadow-xs`}>{l().text}</Text>
                <Show when={l().detail}>
                  <Text class="pl-3 text-caption text-fg-muted text-shadow-xs">{l().detail}</Text>
                </Show>
              </View>
            )}
          </For>
        </Scroll>
        <View class="absolute inset-x-0 bottom-0 h-4 bg-linear-to-t from-plate-lo to-transparent pointer-events-none" />
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CombatLogProps>> = {
  default: {"lines":[{"id":"l1","text":"Sir Aldric uses Shield Bash on Cultist.","tone":"plain","detail":"→ 42 damage"},{"id":"l2","text":"Cultist is Stunned!","tone":"status","detail":""}]},
};
