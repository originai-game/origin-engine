import { For, Show, Text, View, t } from "@origingame/engine/ui";
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
  /** Ally or enemy, the killer's side (the victim takes the other). */
  team?: string;
}

export interface KillfeedProps {
  /** The eliminations to show, newest first. The game trims old rows (a phone shows the two newest upright, three on its side); a new row slides in and an old one stays whole until it fades out over its last 300 ms. */
  rows?: ReadonlyArray<KillfeedRow>;
}

/**
 * Eliminations at the screen's top corner, newest first: each row the
 * killer, the weapon's name and the victim, a headshot tag, names in their
 * side's colour on a dark glass that fades toward the middle, an edge at
 * the screen's side lit for the player's own rows. Read only. The weapon
 * glyphs (svg silhouettes) have no TSX surface: every weapon shows in
 * words. The rows' slide and fade transitions have no TSX surface and are
 * dropped; a phone's compact trimming is the game's to do in the prop.
 */
export function Killfeed(props: KillfeedProps) {
  return (
    <View class={`flex-col items-end max-w-full gap-y-[0.357rem] text-fg`} role="log" label={t("shooter.killfeed.name")}>
      <For each={props.rows ?? []} key={(r) => r.id}>
        {(r) => (
          <View
            class={`group/row relative flex-row items-center min-w-0 max-w-full pl-3 bg-linear-to-l from-[#1a1e28db] to-[#1a1e2859] compact:pl-2`}
            label={t("shooter.killfeed.row", { killer: r().killer, victim: r().victim, weapon: r().weapon ?? "" })}
            data-self={r().self}
          >
            <Text class={`shrink min-w-0 py-[0.214rem] text-[1rem] font-semibold whitespace-nowrap truncate text-shadow-2xs text-[#4fd8ff] data-enemy:text-[#ff4d5e] data-self-ally:text-[#38e8ff]`} data-enemy={(r().team ?? "") !== "ally"} data-self-ally={(r().team ?? "") === "ally" && r().self}>
              {r().killer}
            </Text>
            <Text class={`shrink-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap truncate text-white/78 text-shadow-2xs`}>{r().weapon ?? ""}</Text>
            <Text class={`shrink min-w-0 py-[0.214rem] text-[1rem] font-semibold whitespace-nowrap truncate text-shadow-2xs text-[#ff4d5e] data-ally:text-[#4fd8ff] data-self-ally:text-[#38e8ff]`} data-ally={(r().team ?? "") !== "ally"} data-self-ally={(r().team ?? "") !== "ally" && r().self}>
              {r().victim}
            </Text>
            <Show when={r().headshot}>
              <Text class={`shrink-0 py-0 px-[0.357rem] text-[0.857rem] font-bold tracking-[0.14em] uppercase whitespace-nowrap text-[#ff4d5e] text-shadow-none bg-[#1b1f29e0] shadow-[inset_0px_1px_0px_0px_#ffffff38] rounded-tl-[0.286rem]`}>
                {t("shooter.killfeed.hs")}
              </Text>
            </Show>
            <View class={`shrink-0 self-stretch w-[0.214rem] ml-1 bg-white/60 group-data-self/row:bg-[#38e8ff]`} />
          </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<KillfeedProps>> = {
  default: {"rows":[{"id":"k3","killer":"You","victim":"Vanta","weapon":"VK-7 Carbine","kind":"rifle","headshot":true,"self":true,"team":"ally"},{"id":"k2","killer":"Quill","victim":"Okonkwo-Reyes","weapon":"Hollow SMG","kind":"smg","headshot":false,"self":false,"team":"ally"},{"id":"k1","killer":"Vanta","victim":"Moraine","weapon":"Talon AR","kind":"rifle","headshot":false,"self":false,"team":"enemy"}]},
  words: {"rows":[{"id":"w2","killer":"Halden","victim":"Kestrel","weapon":"Brace-12","kind":"","headshot":true,"self":true,"team":"enemy"},{"id":"w1","killer":"Sable","victim":"Quill","weapon":"Pike .50","kind":"sniper","headshot":false,"self":false,"team":"ally"}]},
  empty: {"rows":[]},
  long: {"rows":[{"id":"l1","killer":"AVeryLongPlayerNameThatOverflows","victim":"AnotherExtremelyLongName_42","weapon":"Lancet DMR Extended Marksman Rifle","kind":"","headshot":true,"self":false,"team":"enemy"}]},
};
