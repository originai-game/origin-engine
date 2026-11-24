import { For, Icon, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import "./killfeed.en.ogt";

export interface KillfeedRow {
  id: string;
  /** The killer's name. */
  killer: string;
  /** The victim's name. */
  victim: string;
  /** The weapon's name. */
  weapon?: string;
  /** The weapon's class (rifle, smg, pistol, shotgun, sniper, launcher or melee); its glyph is drawn from it, empty for the weapon's name in words. */
  kind?: string;
  headshot?: boolean;
  /** The player took part (the row's edge is lit and the player's name takes the HUD's accent). */
  self?: boolean;
  /** Ally or enemy, the killer's side (the victim takes the other); empty for FFA neutral ink. */
  team?: string;
}

export interface KillfeedProps {
  /** The eliminations to show, newest first. The game trims old rows (a phone shows the two newest upright, three on its side); a new row slides in and an old one stays whole until it fades out over its last 300 ms. */
  rows?: ReadonlyArray<KillfeedRow>;
}

const KIND_ICON: Record<string, string> = {
  rifle: "crosshair",
  smg: "crosshair",
  pistol: "crosshair",
  shotgun: "crosshair",
  sniper: "crosshair",
  launcher: "bomb",
  melee: "sword",
};

/**
 * Eliminations at the screen's top corner, newest first: each row the
 * killer, the weapon icon and name, and the victim, a headshot tag, names
 * in their side's colour on frosted glass, an edge at the screen's side
 * lit for the player's own rows. Read only.
 */
export function Killfeed(props: KillfeedProps) {
  return (
    <View class={`flex-col items-end max-w-full gap-y-1.25 text-fg`} role="log" label={t("shooter.killfeed.name")}>
      <For each={props.rows ?? []} key={(r) => r.id}>
        {(r) => {
          const team = () => r().team ?? "";
          const kindIcon = () => KIND_ICON[r().kind ?? ""] ?? "";
          const killerTone = () =>
            team() === "ally"
              ? r().self ? "text-hud-accent-ink" : "text-ally-ink"
              : team() === "enemy"
                ? "text-foe-ink"
                : r().self ? "text-hud-accent-ink" : "text-fg";
          const victimTone = () =>
            team() === "ally"
              ? "text-foe-ink"
              : team() === "enemy"
                ? r().self ? "text-hud-accent-ink" : "text-ally-ink"
                : "text-fg-muted";
          return (
            <View
              class={`group/row relative flex-row items-center min-w-0 max-w-full overflow-hidden pl-3 compact:pl-2 ${packClass("Feed", "row")} ${r().self ? packClass("Feed", "row", "self") : ""}`}
              label={t("shooter.killfeed.row", { killer: r().killer, victim: r().victim, weapon: r().weapon ?? "" })}
              data-self={r().self}
            >
              <Text class={`shrink min-w-0 py-0.75 whitespace-nowrap truncate ${packClass("Feed", "name")} ${killerTone()}`}>
                {r().killer}
              </Text>
              <Show when={kindIcon() !== ""}>
                <Icon class="shrink-0 size-3.5 text-fg-muted" name={kindIcon()} />
              </Show>
              <Show when={(r().weapon ?? "") !== ""}>
                <Text class={`shrink-[4] min-w-0 whitespace-nowrap truncate ${packClass("Feed", "weapon")}`}>{r().weapon ?? ""}</Text>
              </Show>
              <Text class={`shrink min-w-0 py-0.75 whitespace-nowrap truncate ${packClass("Feed", "name")} ${victimTone()}`}>
                {r().victim}
              </Text>
              <Show when={r().headshot}>
                <Text class={`shrink-0 py-0 px-1.25 text-xs font-bold tracking-label uppercase whitespace-nowrap text-foe-ink text-shadow-none ${packClass("Chip", "frame")}`}>
                  {t("shooter.killfeed.hs")}
                </Text>
              </Show>
              <View class={`shrink-0 self-stretch w-0.75 ml-1 bg-sheen group-data-self/row:bg-hud-accent`} />
            </View>
          );
        }}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<KillfeedProps>> = {
  default: {"rows":[{"id":"k3","killer":"You","victim":"Vanta","weapon":"VK-7 Carbine","kind":"rifle","headshot":true,"self":true,"team":"ally"},{"id":"k2","killer":"Quill","victim":"Okonkwo-Reyes","weapon":"Hollow SMG","kind":"smg","headshot":false,"self":false,"team":"ally"},{"id":"k1","killer":"Vanta","victim":"Moraine","weapon":"Talon AR","kind":"rifle","headshot":false,"self":false,"team":"enemy"}]},
  words: {"rows":[{"id":"w2","killer":"Halden","victim":"Kestrel","weapon":"Brace-12","kind":"","headshot":true,"self":true,"team":"enemy"},{"id":"w1","killer":"Sable","victim":"Quill","weapon":"Pike .50","kind":"sniper","headshot":false,"self":false,"team":"ally"}]},
  long: {"rows":[{"id":"l1","killer":"AVeryLongPlayerNameThatOverflows","victim":"AnotherExtremelyLongName_42","weapon":"Lancet DMR Extended Marksman Rifle","kind":"","headshot":true,"self":false,"team":"enemy"}]},
};
