import { For, Icon, Image, Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./shooter.en.ogt";

export interface ShooterVitalsProps {
  /** The player's name as the game words it; empty hides it. */
  name?: string;
  /** The player's level beside the name; 0 hides it. */
  level?: number;
  /** The player's portrait texture (og.load.texture); 0 shows the person glyph in its place. */
  portrait?: Texture | 0;
  /** Health now. */
  health: number;
  /** Most health. Default 100. */
  healthMax?: number;
  /** Health before the latest loss: the bar keeps a warm ghost to it that drains; at or under health shows none. */
  ghost?: number;
  /** Shield now, a cut bar over the health bar. */
  shield?: number;
  /** Most shield; 0 hides the shield bar. */
  shieldMax?: number;
  /** Shield per cell of the shield bar (25: four cells for 100). Default 25. */
  segment?: number;
  /** The fraction of most health at or under which the health bar turns to danger (0.3). Default 0.3. */
  low?: number;
}

/**
 * The player's vitals at the screen's bottom corner: the portrait in an
 * octagon with a lit edge, the name and level, health as a figure over its
 * most, a shield bar cut into cells over a fat health bar that leaves a
 * warm ghost when it drops and turns to danger when low. The game owns
 * every value; read only. Approximations: the portrait's octagon cut is
 * rounded and the cells' mask is hairline dividers.
 */
export function ShooterVitals(props: ShooterVitalsProps) {
  const healthMax = () => props.healthMax ?? 100;
  const healthShare = () => (props.health <= 0 ? 0 : props.health >= healthMax() ? 1 : props.health / healthMax());
  const ghostShare = () => ((props.ghost ?? 0) <= props.health ? 0 : (props.ghost ?? 0) >= healthMax() ? 1 : (props.ghost ?? 0) / healthMax());
  const shieldMax = () => props.shieldMax ?? 0;
  const shieldShare = () => ((props.shield ?? 0) <= 0 ? 0 : (props.shield ?? 0) >= shieldMax() ? 1 : (props.shield ?? 0) / shieldMax());
  const cells = () => ((props.segment ?? 25) > 0 ? Math.floor(shieldMax() / (props.segment ?? 25)) : 1);
  const low = () => props.health <= healthMax() * (props.low ?? 0.3);
  const rim = "shadow-[0px_0px_0px_1px_rgb(255_255_255/0.22),inset_0px_1px_0px_0px_rgb(255_255_255/0.14),0px_0.143rem_0.286rem_rgb(0_0_0/0.45)]";
  return (
    <View
      class={`group/vitals flex-row items-end gap-x-3.5 w-82.5 max-w-full compact:w-full compact:gap-x-2 text-fg`}
      role="group"
      label={t("shooter.vitals.name")}
      data-low={low()}
    >
      <View class="shrink-0 size-16 compact:size-10 rounded-[1rem] bg-linear-to-b from-white/75 via-white/22 to-white/14">
        <View class="absolute inset-0.5 rounded-panel overflow-hidden bg-linear-to-b from-plate-hi to-plate-lo">
          <Show when={(props.portrait ?? 0) > 0} fallback={
            <View class="size-full flex-row items-center justify-center">
              <Icon class="size-1/2 text-fg/66" name="person" />
            </View>
          }>
            <Image class="size-full" src={props.portrait ?? 0} />
          </Show>
        </View>
      </View>
      <View class="flex-1 min-w-0 flex-col gap-y-1.25">
        <View class="flex-row flex-wrap items-baseline gap-x-2 min-w-0">
          <View class="shrink basis-auto flex-row items-baseline gap-x-2 min-w-0">
            <Show when={(props.name ?? "") != ""}>
              <Text class="min-w-0 text-base font-semibold whitespace-nowrap truncate text-fg text-shadow-2xs text-stroke-3 text-stroke-bg">{props.name}</Text>
            </Show>
            <Show when={(props.level ?? 0) > 0}>
              <Text class="shrink-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap text-hud-accent text-shadow-2xs text-stroke-3 text-stroke-bg">
                {t("shooter.vitals.level", { level: props.level ?? 0 })}
              </Text>
            </Show>
          </View>
          <View class="shrink-0 ml-auto flex-row items-baseline gap-x-1">
            <Text class="text-title leading-none font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs text-stroke-3 text-stroke-bg group-data-low/vitals:text-danger compact:text-base">
              {formatNumber(props.health)}
            </Text>
            <Text class="text-caption font-bold tabular-nums whitespace-nowrap text-fg/66 text-shadow-2xs text-stroke-3 text-stroke-bg">
              {t("shooter.vitals.max", { max: formatNumber(healthMax()) })}
            </Text>
          </View>
        </View>
        <Show when={shieldMax() > 0}>
          <View
            class={`relative w-full h-2.25 compact:h-1.5 overflow-hidden -skew-x-22 bg-track ${rim}`}
            role="meter"
            label={t("shooter.vitals.shield")}
            valueMin={0}
            valueMax={shieldMax()}
            valueNow={props.shield ?? 0}
          >
            <View class="absolute left-0 top-0 h-full bg-ally shadow-[0.143rem_0px_0.714rem_rgb(79_216_255/0.8)]" style={{ width: `${shieldShare() * 100}%` }}>
              <View class="absolute left-0 right-0 top-0 h-21/50 bg-linear-to-b from-white/55 to-transparent" />
            </View>
            <View class="absolute inset-0 pointer-events-none">
              <For each={Array.from({ length: Math.max(cells() - 1, 0) }, (_, k) => k)} key={(k) => `c${k}`}>
                {(k) => <View class="absolute top-0 bottom-0 w-0.75 bg-tile" style={{ left: `${((k() + 1) / cells()) * 100}%` }} />}
              </For>
            </View>
          </View>
        </Show>
        <View
          class={`relative w-full h-4 compact:h-2.75 overflow-hidden -skew-x-22 bg-track ${rim} group-data-low/vitals:shadow-[0px_0px_0px_1px_#ff4d5e,0px_0px_0.857rem_rgb(255_77_94/0.7)] group-data-low/vitals:animate-pulse`}
          role="meter"
          label={t("shooter.vitals.health")}
          valueMin={0}
          valueMax={healthMax()}
          valueNow={props.health}
        >
          <View class="absolute left-0 top-0 h-full bg-[linear-gradient(90deg,rgb(255_207_90/0.15),rgb(255_207_90/0.85))]" style={{ width: `${ghostShare() * 100}%` }} />
          <View
            class={`absolute left-0 top-0 h-full bg-[rgb(244_246_251)] shadow-[0.143rem_0px_0.714rem_rgb(244_246_251/0.55)] group-data-low/vitals:bg-danger group-data-low/vitals:shadow-[0.143rem_0px_0.714rem_rgb(255_77_94/0.8)]`}
            style={{ width: `${healthShare() * 100}%` }}
          >
            <View class="absolute left-0 right-0 top-0 h-21/50 bg-linear-to-b from-white/55 to-transparent" />
          </View>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShooterVitalsProps>> = {
  default: {"name":"Ysolde","level":24,"portrait":0,"health":64,"healthMax":100,"ghost":82,"shield":75,"shieldMax":100,"segment":25,"low":0.3},
  full: {"name":"Ysolde","level":24,"portrait":0,"health":100,"healthMax":100,"ghost":0,"shield":100,"shieldMax":100,"segment":25,"low":0.3},
  low: {"name":"Ysolde","level":24,"portrait":0,"health":18,"healthMax":100,"ghost":41,"shield":0,"shieldMax":100,"segment":25,"low":0.3},
  critical: {"name":"Ysolde","level":24,"portrait":0,"health":3,"healthMax":100,"ghost":12,"shield":0,"shieldMax":100,"segment":25,"low":0.3},
  noshield: {"name":"","level":0,"portrait":0,"health":150,"healthMax":200,"ghost":0,"shield":0,"shieldMax":0,"segment":25,"low":0.3},
  overflow: {"name":"Ysolde Varrenhallow-Okonkwo of the Ninth Relay Company","level":1234,"portrait":0,"health":12345,"healthMax":99999,"ghost":20000,"shield":250,"shieldMax":300,"segment":50,"low":0.3},
};
