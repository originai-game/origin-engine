import { Show, View, packClass, t } from "@origingame/engine/ui";
import { Crosshair } from "../hud/crosshair";
import { DamageDirection, type DamageDirectionHit } from "../hud/damage-direction";
import { Killfeed, type KillfeedRow } from "../hud/killfeed";
import { StatBar } from "../hud/stat-bar";
import { AmmoCounter } from "../hud/ammo-counter";
import "./shooter.en.strings.ts";

export interface CombatHudProps {
  /** Health now. */
  health?: number;
  /** Most health. Default 1. */
  healthMax?: number;
  /** Shield now, a strip over the health bar. */
  shield?: number;
  /** Most shield; 0 hides the strip. */
  shieldMax?: number;
  /** Crosshair spread, 0 to 1 (Crosshair). */
  spread?: number;
  /** Crosshair hit mark: none, hit, headshot, kill or shieldBreak. Default none. */
  hit?: string;
  /** The hit's id; a new one replays the mark. */
  hitAt?: number;
  /** The aim rests on an enemy. */
  aimed?: boolean;
  /** Crosshair shape: cross, dot or circle. Default cross. */
  shape?: string;
  /** Weapon name (AmmoCounter). */
  weapon?: string;
  /** Fire mode: auto, burst or single. Default auto. */
  mode?: string;
  /** Rounds in the magazine. */
  magazine?: number;
  /** Magazine size. */
  capacity?: number;
  /** Rounds in reserve. */
  reserve?: number;
  /** Reload progress 0 to 1; negative when not reloading. Default -1. */
  reloading?: number;
  /** Incoming hits (DamageDirection). */
  hits?: ReadonlyArray<DamageDirectionHit>;
  /** Killfeed rows, newest first (Killfeed). */
  feed?: ReadonlyArray<KillfeedRow>;
}

/**
 * The shooter's whole HUD: the reticle and the hit directions centred, the
 * killfeed at the top's end, the vitals at the bottom's start and the
 * ammunition at the bottom's end. The edges sit inside a fixed inset, so a
 * phone's notch and home bar are the caller's (the svelte's safe-area
 * insets and its right-to-left mirror are dropped), and a phone's compact
 * reflow keeps the corners.
 */
export function CombatHud(props: CombatHudProps) {
  return (
    <View class="relative inset-0 w-full h-full pointer-events-none">
      <View class="absolute inset-0">
        <View class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 size-63 compact:size-42">
          <DamageDirection hits={props.hits ?? []} />
        </View>
        <View class="absolute inset-0 flex-row items-center justify-center">
          <Crosshair
            spread={props.spread ?? 0}
            hit={props.hit}
            hitAt={props.hitAt}
            aimed={props.aimed}
            shape={props.shape}
          />
        </View>
      </View>
      <View class="absolute inset-0 pt-[env(hud-inset-top)] pb-[env(hud-inset-bottom)] pl-[env(hud-inset-left)] pr-[env(hud-inset-right)]">
        <View class="relative w-full h-full">
          <View class="absolute top-4 right-4 max-w-full compact:top-3 compact:right-3">
            <Killfeed rows={props.feed ?? []} />
          </View>
          <View
            class={`absolute left-4 bottom-4 py-2 px-3 w-70 max-w-full compact:left-3 compact:right-3 compact:bottom-3 compact:w-auto ${packClass("Plate", "frame")}`}
          >
            <StatBar
              value={props.health ?? 0}
              max={props.healthMax ?? 1}
              kind="health"
              segment={25}
              shield={props.shield ?? 0}
              shieldMax={props.shieldMax ?? 0}
              heal={0}
              recoverable={0}
              label=""
              icon="heart"
              showValue={true}
              valueInside={false}
              valueText=""
              low={0.3}
              critical={0.15}
              bare={true}
              name={t("shooter.hud.health")}
            />
          </View>
          <View class="absolute right-4 bottom-4 compact:right-3 compact:bottom-16.75">
            <AmmoCounter
              weapon={props.weapon ?? ""}
              mode={props.mode ?? "auto"}
              magazine={props.magazine ?? 0}
              capacity={props.capacity ?? 0}
              reserve={props.reserve ?? 0}
              reloading={props.reloading ?? -1}
              low={0.25}
            />
          </View>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CombatHudProps>> = {
  default: {"health":72,"healthMax":100,"shield":50,"shieldMax":100,"spread":0.25,"hit":"headshot","hitAt":1,"aimed":true,"shape":"cross","weapon":"Halcyon Carbine","mode":"auto","magazine":24,"capacity":28,"reserve":112,"reloading":-1,"hits":[{"id":"h1","angle":0.9,"strength":0.9,"kind":"normal"}],"feed":[{"id":"k1","killer":"Kestrel","victim":"Vanta","weapon":"Halcyon","headshot":true,"self":true,"team":"ally"}]},
};
