import { Show, View, t } from "@origingame/engine/ui";
import "./crosshair.en.ogt";

export interface CrosshairProps {
  /** The weapon's current spread, 0 (settled) to 1 (widest): the arms move out from the centre. */
  spread: number;
  /** The last hit to confirm: none, hit, headshot, kill or shieldBreak. The game clears it to none when its mark should go. Default none. */
  hit?: string;
  /** The id of that hit (its tick): a new id plays the mark again. */
  hitAt?: number;
  /** The aim rests on an enemy: the reticle takes the target ink. */
  aimed?: boolean;
  /** cross (four arms and a dot), dot, or circle (a ring and a dot). Default cross. */
  shape?: string;
}

/**
 * The reticle: four arms and a dot (or a ring and a dot), opening with
 * the weapon's spread, the target's ink when the aim rests on an enemy,
 * and four diagonal ticks confirming the last hit (red with a glow for a
 * headshot and a kill, the shield's blue for a broken one). Geometry is
 * resolved into styles; the ring's mask is a border, and a new hitAt does
 * not replay the mark while it shows (no keyed surface): the ticks pop in
 * when the mark appears.
 */
export function Crosshair(props: CrosshairProps) {
  const gap = () => 0.571 + Math.min(Math.max(props.spread, 0), 1) * 2.286;
  const tick = () => (props.hit ?? "none") != "none";
  const tickClass = `absolute w-[1rem] h-[0.214rem] bg-white shadow-[0px_0px_0px_0.107rem_#000000b3] data-headshot:bg-foe data-headshot:shadow-[0px_0px_0.571rem_#ff4d5e] data-kill:h-[0.286rem] data-kill:bg-foe data-kill:shadow-[0px_0px_0.571rem_#ff4d5e] data-shieldbreak:bg-ally data-shieldbreak:shadow-[0px_0px_0.571rem_#4fd8ff] animate-pop`;
  const armClass = `absolute bg-white shadow-[0px_0px_0px_0.107rem_#000000b3] data-aimed:bg-foe data-dot:hidden data-circle:hidden`;
  return (
    <View class="relative size-24" role="image" label={t("shooter.crosshair.name")} data-aimed={props.aimed} data-dot={(props.shape ?? "cross") == "dot"} data-circle={(props.shape ?? "cross") == "circle"}>
      <View class={`${armClass} w-[0.143rem] h-[0.714rem]`} data-aimed={props.aimed} data-dot={(props.shape ?? "cross") == "dot"} data-circle={(props.shape ?? "cross") == "circle"} style={{ left: "calc(50% - 0.0715rem)", bottom: `calc(50% + ${gap()}rem)` }} />
      <View class={`${armClass} w-[0.143rem] h-[0.714rem]`} data-aimed={props.aimed} data-dot={(props.shape ?? "cross") == "dot"} data-circle={(props.shape ?? "cross") == "circle"} style={{ left: "calc(50% - 0.0715rem)", top: `calc(50% + ${gap()}rem)` }} />
      <View class={`${armClass} w-[0.714rem] h-[0.143rem]`} data-aimed={props.aimed} data-dot={(props.shape ?? "cross") == "dot"} data-circle={(props.shape ?? "cross") == "circle"} style={{ top: "calc(50% - 0.0715rem)", right: `calc(50% + ${gap()}rem)` }} />
      <View class={`${armClass} w-[0.714rem] h-[0.143rem]`} data-aimed={props.aimed} data-dot={(props.shape ?? "cross") == "dot"} data-circle={(props.shape ?? "cross") == "circle"} style={{ top: "calc(50% - 0.0715rem)", left: `calc(50% + ${gap()}rem)` }} />
      <View
        class="absolute rounded-full border-[0.143rem] border-white shadow-[0px_0px_0px_0.107rem_#000000b3] hidden data-circle:block data-aimed:border-foe"
        data-circle={(props.shape ?? "cross") == "circle"}
        data-aimed={props.aimed}
        style={{ left: `calc(50% - ${gap()}rem - 0.357rem)`, top: `calc(50% - ${gap()}rem - 0.357rem)`, width: `calc(${gap() * 2}rem + 0.714rem)`, height: `calc(${gap() * 2}rem + 0.714rem)` }}
      />
      <View
        class="absolute size-[0.286rem] rounded-full bg-white shadow-[0px_0px_0px_0.107rem_#000000b3] data-aimed:bg-foe"
        data-aimed={props.aimed}
        style={{ left: "calc(50% - 0.143rem)", top: "calc(50% - 0.143rem)" }}
      />
      <Show when={tick()}>
        <View class="absolute inset-0" data-headshot={(props.hit ?? "") == "headshot"} data-kill={(props.hit ?? "") == "kill"} data-shieldbreak={(props.hit ?? "") == "shieldBreak"}>
          <View class={tickClass} data-headshot={(props.hit ?? "") == "headshot"} data-kill={(props.hit ?? "") == "kill"} data-shieldbreak={(props.hit ?? "") == "shieldBreak"} style={{ left: `calc(50% + ${gap()}rem + 0.286rem)`, top: `calc(50% - ${gap()}rem - 0.286rem - 0.5rem)`, rotate: "-45deg" }} />
          <View class={tickClass} data-headshot={(props.hit ?? "") == "headshot"} data-kill={(props.hit ?? "") == "kill"} data-shieldbreak={(props.hit ?? "") == "shieldBreak"} style={{ left: `calc(50% + ${gap()}rem + 0.286rem)`, top: `calc(50% + ${gap()}rem + 0.286rem + 0.5rem)`, rotate: "45deg" }} />
          <View class={tickClass} data-headshot={(props.hit ?? "") == "headshot"} data-kill={(props.hit ?? "") == "kill"} data-shieldbreak={(props.hit ?? "") == "shieldBreak"} style={{ left: `calc(50% - ${gap()}rem - 0.286rem - 1rem)`, top: `calc(50% + ${gap()}rem + 0.286rem + 0.5rem)`, rotate: "-45deg" }} />
          <View class={tickClass} data-headshot={(props.hit ?? "") == "headshot"} data-kill={(props.hit ?? "") == "kill"} data-shieldbreak={(props.hit ?? "") == "shieldBreak"} style={{ left: `calc(50% - ${gap()}rem - 0.286rem - 1rem)`, top: `calc(50% - ${gap()}rem - 0.286rem - 0.5rem)`, rotate: "45deg" }} />
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<CrosshairProps>> = {
  default: {"spread":0.2,"hit":"none","hitAt":0,"aimed":false,"shape":"cross"},
  headshot: {"spread":0.45,"hit":"headshot","hitAt":12,"aimed":true,"shape":"cross"},
  kill: {"spread":0,"hit":"kill","hitAt":30,"aimed":false,"shape":"dot"},
  circle: {"spread":0.6,"hit":"shieldBreak","hitAt":4,"aimed":false,"shape":"circle"},
};
