import { For, Icon, Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import type { Entity } from "@origingame/engine";
import "./world.en.ogt";

export interface NameplateProps {
  /** The unit it sits over, by its bits (0: use at). */
  entity?: number;
  /** A world point it sits over without an entity (x y z). */
  at?: string;
  /** World units above the point (its head). */
  height?: number;
  /** The name, in the reaction's ink; a long one scrolls on one line. */
  name: string;
  /** A line under it in small capitals (a role, a guild); empty is none. */
  title?: string;
  /** Its level on a chip as the game words it (32); empty is none, a boss's empty level draws a skull. */
  level?: string;
  /** Its colour: neutral (empty), friendly, hostile or player. */
  reaction?: string;
  /** Health 0 to 1 for a skewed bar cut in quarters under the name; negative draws none (a friendly NPC that cannot be hurt). When it drops, a cream ghost of what was lost holds 400 ms and drains in 300 ms. At 0.25 or less the rim turns danger and breathes, and the percent shows. */
  health?: number;
  /** An absorb after the health, 0 to 1, in cyan. */
  shield?: number;
  /** The spell it is casting; empty is nothing. A boss or a caster sits above the interaction prompt when plates overlap. */
  castName?: string;
  /** How far the cast is, 0 to 1 (the game's progress, read every tick). */
  castProgress?: number;
  /** The cast's time left as the game words it (1.2); empty is none. */
  castTime?: string;
  /** The cast: casting (empty), locked (it can't be interrupted: steel grey with a shield mark) or interrupted (a red hatch and the word; the game shows it 600 ms, then clears castName). */
  castState?: string;
  /** A class, quest or role icon before the name; empty is none. */
  icon?: string;
  /** An elite: a gold level chip, gold wings at the bar's ends and a gold rim. */
  elite?: boolean;
  /** A boss: as an elite, with its name in tracked capitals, a longer bar and the percent. */
  boss?: boolean;
  /** The player's target: 1.14 times larger, its name and rim white, white brackets that close in 120 ms; drawn over the rest. */
  targeted?: boolean;
  /** Dead: the name greys and the bars go. */
  dead?: boolean;
  /** Dead with loot: Lootable in gold under the name. */
  lootable?: boolean;
  /** Far or crowded: only the bar (no name, title or cast). */
  compact?: boolean;
  /** Hidden past this many world units (80). */
  maxDistance?: number;
}

/**
 * A unit's plate over its head: the level chip and the name in its
 * reaction's ink, its title in small capitals, its health as a bar cut in
 * quarters (a cream ghost of a hit, the shield after the fill) and the
 * cast row (the spell and its time over a thin bar). Read only; the game
 * owns every figure and state. The old style's lean (skewX), the wings'
 * and the target brackets' clip-path arrows, the interrupted cast's hatch
 * and the anchor's max distance have no TSX surface and are dropped; it
 * stacks over the plates it overlaps and fades behind drawn geometry
 * (the anchor's declutter and occlusion); the quarters are drawn as
 * three notch marks, the plate does not scale when targeted, and the
 * plate's room cap (a viewport measure) stands as max-w-full.
 */
export function Nameplate(props: NameplateProps) {
  const reaction = () => ((props.reaction ?? "") == "" ? "neutral" : (props.reaction ?? ""));
  const level = () => props.level ?? "";
  const title = () => props.title ?? "";
  const icon = () => props.icon ?? "";
  const castName = () => props.castName ?? "";
  const castState = () => ((props.castState ?? "") == "" ? "casting" : (props.castState ?? ""));
  const castTime = () => props.castTime ?? "";
  const health = () => props.health ?? -1;
  const hp = () => Math.min(Math.max(health(), 0), 1);
  const low = () => health() >= 0 && health() <= 0.25 && !props.dead;
  const elite = () => (props.elite || props.boss) === true;
  const point = () => ((props.at ?? "0 0 0").split(/\s+/).map(Number) as [number, number, number]);
  const castFill = () => (castState() === "interrupted" ? 1 : Math.min(Math.max(props.castProgress ?? 0, 0), 1));
  return (
    <View
      anchor={props.entity ? (props.entity as Entity) : point()}
      lift={[0, props.height ?? 0, 0]}
      declutter="stack"
      occlusion="fade"
      class="group -translate-x-1/2 -translate-y-full flex-col items-center gap-y-[0.143rem] w-max max-w-full text-fg leading-[1.12]"
      label={props.name}
      data-boss={props.boss}
      data-compact={props.compact}
      data-targeted={props.targeted}
      data-elite={elite()}
      data-dead={props.dead}
      data-low={low()}
      data-reaction-friendly={reaction() === "friendly"}
      data-reaction-hostile={reaction() === "hostile"}
      data-reaction-player={reaction() === "player"}
    >
      <Show when={!props.compact}>
        <View class="flex-row items-center justify-center gap-x-[0.286rem] max-w-full min-w-0">
          <Show when={level() !== ""} fallback={
            <Show when={props.boss}>
              <View class="shrink-0 flex-row items-center justify-center h-[1.143rem] w-[1.357rem] nine-chip-cut bg-raised-pressed/88" label={t("world.nameplate.boss")}>
                <Icon class="size-[0.786rem] text-fg" name="skull" />
              </View>
            </Show>
          }>
            <View class="shrink-0 flex-row items-center justify-center h-[1.143rem] min-w-[1.357rem] px-[0.286rem] nine-chip-cut bg-raised-pressed/88 group-data-elite:bg-gold group-data-elite:text-gold-fg group-data-dead:opacity-50" label={t("world.nameplate.level", { level: level() })}>
              <Text class="text-[0.786rem] leading-none font-bold tabular-nums">{level()}</Text>
            </View>
          </Show>
          <Show when={icon() !== ""}>
            <Icon class="shrink-0 size-4" name={icon()} />
          </Show>
          <Text class="shrink min-w-0 max-w-[10.714rem] text-reading font-semibold whitespace-nowrap truncate text-gold text-shadow-2xs text-stroke-3 text-stroke-bg group-data-reaction-hostile:text-danger group-data-reaction-friendly:text-success group-data-reaction-player:text-hud-accent group-data-boss:max-w-[14.286rem] group-data-boss:text-[1.214rem] group-data-boss:font-bold group-data-boss:tracking-[0.07em] group-data-boss:uppercase group-data-targeted:text-fg group-data-dead:text-fg/45">{props.name}</Text>
        </View>
        <Show when={title() !== ""}>
          <Text class="min-w-0 max-w-[13.571rem] text-[0.75rem] font-semibold tracking-[0.13em] uppercase whitespace-nowrap truncate text-fg/78 text-shadow-2xs group-data-dead:text-fg/45">{title()}</Text>
        </Show>
        <Show when={props.dead && props.lootable}>
          <Text role="status" class="text-[0.75rem] font-bold tracking-[0.13em] uppercase whitespace-nowrap text-gold text-shadow-2xs">{t("world.nameplate.lootable")}</Text>
        </Show>
      </Show>
      <Show when={health() >= 0 && !props.dead}>
        <View class="relative flex-row items-center gap-x-[0.357rem]">
          <View
            class={`relative h-2 w-[8.286rem] bg-track group-data-boss:h-[0.714rem] group-data-boss:w-[14.286rem] group-data-compact:h-[0.429rem] group-data-compact:w-[4.143rem] ${props.targeted ? (elite() ? "shadow-[0px_0px_0px_1.5px_#ffffff,0px_0px_0px_2.5px_#ffd23fb3]" : "shadow-[0px_0px_0px_1.5px_#ffffff,0px_0px_0.714rem_#ffffff59]") : elite() ? "shadow-[0px_0px_0px_1px_#ffd23f,0px_0.143rem_0.429rem_#00000080]" : "shadow-[0px_0px_0px_1px_#ffffff47,0px_0.143rem_0.429rem_#00000080]"}`}
            role="meter"
            label={t("world.nameplate.health")}
            valueMin={0}
            valueMax={1}
            valueNow={hp()}
            valueText={`${formatNumber(hp() * 100, { decimals: 0 })}%`}
          >
            <View class="absolute left-0 top-0 bottom-0 overflow-hidden">
              <View class="absolute left-0 top-0 bottom-0 bg-gold-hi/82 transition-layout duration-300 delay-400 ease-out" style={{ width: `${hp() * 100}%` }} />
              <View class="absolute left-0 top-0 bottom-0 bg-linear-to-b from-white/55 to-transparent to-50% bg-gold group-data-reaction-hostile:bg-danger group-data-reaction-friendly:bg-success group-data-reaction-player:bg-hud-accent" style={{ width: `${hp() * 100}%` }} />
              <Show when={(props.shield ?? 0) > 0}>
                <View class="absolute top-0 bottom-0 bg-linear-to-b from-white/55 to-transparent to-42% bg-info" style={{ left: `${hp() * 100}%`, width: `${Math.min(props.shield ?? 0, 1) * 100}%` }} />
              </Show>
              <For each={[1, 2, 3]} key={(i) => i}>
                {(i) => <View class="absolute top-0 bottom-0 w-[1px] bg-fg/75" style={{ left: `${i() * 25}%` }} />}
              </For>
            </View>
            <Show when={low()}>
              <View class="absolute inset-0 shadow-[0px_0px_0px_1px_#ff4d5e,0px_0px_0.857rem_#ff4d5ed9] animate-pulse" />
            </Show>
          </View>
          <Show when={(props.boss || low()) && !props.compact}>
            <Text class="shrink-0 text-xs leading-none font-bold tabular-nums whitespace-nowrap text-shadow-2xs group-data-low:text-danger">{`${formatNumber(hp() * 100, { decimals: 0 })}%`}</Text>
          </Show>
        </View>
      </Show>
      <Show when={castName() !== "" && !props.compact && !props.dead}>
        <View
          class="flex-col gap-y-[1px] min-w-36 max-w-[18.286rem] mt-[1px]"
          role="status"
          label={castState() === "locked" ? t("world.nameplate.uninterruptible", { spell: castName() }) : castState() === "interrupted" ? t("world.nameplate.interrupted") : t("world.nameplate.casting", { spell: castName() })}
          data-state-locked={castState() === "locked"}
          data-state-interrupted={castState() === "interrupted"}
        >
          <View class="group/cast flex-row items-center gap-x-[0.429rem] min-w-0 text-[0.821rem] leading-[1.1] font-bold tracking-[0.07em] uppercase whitespace-nowrap text-warn group-data-state-locked:text-fg-muted group-data-state-interrupted:text-fg">
            <Icon class="shrink-0 -mr-[0.214rem] size-[0.821rem]" name={castState() === "locked" ? "shield" : castState() === "interrupted" ? "close" : "bolt"} />
            <Text class="grow min-w-0 max-w-[12.143rem] truncate">{castState() === "interrupted" ? t("world.nameplate.interrupted") : castName()}</Text>
            <Show when={castTime() !== "" && castState() !== "interrupted"}>
              <Text class="shrink-0 tracking-normal tabular-nums text-fg">{castTime()}</Text>
            </Show>
          </View>
          <View class="relative h-[0.357rem] overflow-hidden bg-track shadow-[0px_0px_0px_1px_#ffb5478c] group-data-state-locked:shadow-[0px_0px_0px_1px_#c6c9d499] group-data-state-interrupted:shadow-[0px_0px_0px_1px_#ff4d5e,0px_0px_0.714rem_#ff4d5ecc]">
            <View class="absolute left-0 top-0 bottom-0 bg-linear-to-r from-warn/45 via-warn via-80% to-white shadow-[0px_0px_0.571rem_#ffb547] group-data-state-locked:bg-linear-to-r group-data-state-locked:from-fg-muted/35 group-data-state-locked:via-fg group-data-state-locked:via-85% group-data-state-locked:to-white group-data-state-locked:shadow-none group-data-state-interrupted:bg-danger group-data-state-interrupted:shadow-none" style={{ width: `${castFill() * 100}%` }} />
          </View>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<NameplateProps>> = {
  default: {"entity":0,"at":"0 0 0","height":2.1,"name":"Vorric the Hollow","title":"Ashbound Ritualist","level":"43","reaction":"hostile","health":0.91,"shield":0.18,"castName":"Shadow Volley","castProgress":0.64,"castTime":"1.2","castState":"","icon":"","elite":true,"boss":false,"targeted":false,"dead":false,"lootable":false,"compact":false,"maxDistance":80},
  friendly: {"name":"Mira Ashgrove","title":"Warden’s Scout","level":"","reaction":"friendly","health":-1,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false},
  neutral: {"name":"Old Tam","title":"Ferryman","level":"12","reaction":"","health":1,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false},
  hostile: {"name":"Ashbound Thrall","title":"","level":"41","reaction":"hostile","health":0.78,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false},
  player: {"name":"Sylvenne","title":"Ninefold Wardens","level":"42","reaction":"player","health":0.86,"shield":0.1,"castName":"","castProgress":0,"castTime":"","elite":false},
  boss: {"name":"Ashen Colossus","title":"Warden of the Ninefold Gate","level":"","reaction":"hostile","health":0.58,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false,"boss":true},
  targeted: {"name":"Ashbound Thrall","title":"","level":"41","reaction":"hostile","health":0.78,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false,"targeted":true},
  low: {"name":"Ashbound Thrall","title":"","level":"41","reaction":"hostile","health":0.22,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false},
  casting: {"castState":""},
  locked: {"name":"Ashen Colossus","title":"Warden of the Ninefold Gate","level":"","health":0.58,"shield":0,"castName":"Cataclysm","castProgress":0.62,"castTime":"2.4","castState":"locked","elite":false,"boss":true,"targeted":true},
  interrupted: {"castProgress":1,"castTime":"","castState":"interrupted"},
  dead: {"name":"Ashbound Thrall","title":"","level":"41","reaction":"hostile","health":0,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false,"dead":true,"lootable":true},
  compact: {"name":"Ashbound Thrall","title":"","level":"41","reaction":"hostile","health":0.62,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false,"compact":true},
  overflow: {"name":"Ysolde the Unbroken, Warden of the Ninefold Gate","title":"Guardian of the Eastern Marches and the Ninefold Gate","level":"120","reaction":"hostile","health":0.12,"shield":0.15,"castName":"Ritual of the Ninefold Gate Unending","castProgress":0.85,"castTime":"12.4","castState":"","elite":false,"boss":true,"targeted":true},
};
