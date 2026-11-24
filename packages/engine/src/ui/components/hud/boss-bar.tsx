import { For, Icon, Image, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import { Spinner } from "../base/spinner";
import "./boss-bar.en.ogt";

export interface BossBarPhase {
  id: string;
  label?: string;
  complete?: boolean;
}

export interface BossBarWeakness {
  id: string;
  /** For a reader. */
  name?: string;
  /** A texture (og.load.texture); 0 draws the glyph in its place. */
  icon?: Texture | 0;
  /** An icon name, in art's place. */
  glyph?: string;
}

export interface BossBarProps {
  /** The boss name; empty uses Boss. */
  name?: string;
  /** The boss's title as the game words it (Warden of the Ninefold Gate), beside the name; empty hides it. */
  epithet?: string;
  /** Health, clamped to the supplied range for display; its share of max stands in figures at the bar's end. */
  value?: number;
  /** Full health; a nonpositive capacity shows no health instead of dividing by zero. */
  max?: number;
  /** Health before the latest loss, held as long as the game wants it shown: a light ghost behind the fill, the health just lost. Absent, the ghost rests at the fill and lags it on each loss by itself. */
  ghost?: number;
  /** Keyed phases in the fight's order; the game owns phase progression. A diamond on the bar marks each boundary between phases (the first phase is the bar's right end), dimmed once health has passed it. */
  phases?: ReadonlyArray<BossBarPhase>;
  /** The current phase's stable id; read out with the health. */
  phase?: string;
  /** The stagger gauge's fill, 0 to 1 (the boss breaks at 1); negative hides the gauge. */
  stagger?: number;
  /** Cells the stagger gauge is cut into (10). */
  staggerCells?: number;
  /** The elements the boss is weak to, keyed rows. */
  weaknesses?: ReadonlyArray<BossBarWeakness>;
  /** Health remains visible, greyed, and a worded invulnerability state is shown. */
  invulnerable?: boolean;
  /** The game has confirmed defeat; this does not follow automatically from zero health. */
  defeated?: boolean;
  /** Waiting for boss data; hides stale health and phases. */
  loading?: boolean;
}

/**
 * A boss's bar across the top of the screen, on a HUD plate: a Boss tag,
 * the name and its epithet; a long health bar with a warm ghost behind the fill and a
 * diamond at each phase boundary; under it the stagger gauge cut into
 * cells, the elements the boss is weak to and its health's share. The
 * game owns every value; read only. The style's lean (skewX), the gauge's
 * cut mask (notch marks stand in) and the weakness tiles' cut corners
 * (rounded ones stand in) have no TSX surface; the invulnerable fill is
 * greyed in kind (a grey fill, since the greying filter is dropped).
 */
export function BossBar(props: BossBarProps) {
  const max = () => props.max ?? 1;
  const share = () => (max() <= 0 || (props.value ?? 0) <= 0 ? 0 : (props.value ?? 0) >= max() ? 1 : (props.value ?? 0) / max());
  // The ghost rests at the fill and lags it on a loss (its width eases after a delay), so the health just lost shows; a game's ghost holds it longer.
  const ghost = () => {
    if (max() <= 0) return 0;
    const g = Math.max(props.ghost ?? 0, props.value ?? 0);
    return g <= 0 ? 0 : g >= max() ? 1 : g / max();
  };
  const shown = () => Math.min(Math.max(props.value ?? 0, 0), Math.max(max(), 0));
  const critical = () => !(props.loading ?? false) && !(props.defeated ?? false) && max() > 0 && (props.value ?? 0) / max() <= 0.25;
  const invulnerable = () => (props.invulnerable ?? false) && !(props.defeated ?? false);
  const staggerCells = () => (props.staggerCells ?? 10) > 0 ? (props.staggerCells ?? 1) : 1;
  return (
    <View
      class={`flex-col items-center w-full max-w-150 gap-y-1.5 text-fg nine-panel-cut bg-glass py-2 px-4 compact:gap-y-1 compact:py-1 compact:px-2 ${packClass("Plate", "frame")}`}
      role="region"
      label={(props.name ?? "") !== "" ? props.name : t("hud.boss-bar.name")}
      busy={props.loading}
    >
      <View class={`flex-row items-baseline justify-center w-full min-w-0 gap-x-2.5`}>
        <View class={`flex-row items-center shrink-0 gap-x-1.25`}>
          <View class={`shrink-0 size-1.5 rotate-45 bg-foe`} />
          <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap text-foe-ink`}>{t("hud.boss-bar.tag")}</Text>
        </View>
        <Text class={`shrink min-w-0 text-2xl leading-tight font-bold tracking-label uppercase whitespace-nowrap truncate text-fg compact:text-reading`}>
          {(props.name ?? "") !== "" ? props.name : t("hud.boss-bar.name")}
        </Text>
        <Show when={(props.epithet ?? "") !== ""}>
          <Text class={`shrink min-w-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap truncate text-fg-muted compact:hidden`}>
            {props.epithet}
          </Text>
        </Show>
      </View>
      <Show when={props.loading} fallback={
        <>
          <View
            class={`group/health relative w-full h-3.5 overflow-hidden bg-track shadow-plate ring ring-rim data-critical:shadow-glow data-critical:shadow-danger data-critical:ring data-critical:ring-danger data-critical:animate-pulse compact:h-2.25`}
            role="meter"
            label={t("hud.boss-bar.health")}
            valueMin={0}
            valueMax={max() > 0 ? max() : 1}
            valueNow={shown()}
            valueText={t("hud.boss-bar.value", { value: shown(), max: max() > 0 ? max() : 0 })}
            data-inv={invulnerable()}
            data-critical={critical()}
          >
            <View class={`absolute left-0 top-0 h-full bg-fg ${packClass("Bar", "trail")}`} data-ghost style={{ width: `${ghost() * 100}%` }} />
            <View
              class={`absolute left-0 top-0 h-full bg-linear-to-r from-legendary to-legendary shadow-glow shadow-late ${packClass("Bar", "fill")} group-data-inv/health:bg-none group-data-inv/health:bg-fg-subtle group-data-inv/health:shadow-none`}
              style={{ width: `${share() * 100}%` }}
            >
              <View class="absolute inset-0 bg-linear-to-b from-sheen to-transparent to-42% group-data-inv/health:hidden" />
            </View>
            <For each={(props.phases ?? []).slice(1)} key={(p) => p.id}>
              {(p, i) => (
                <View
                  class={`absolute size-2.25 rotate-45 bg-sheen ring-regular ring-entry-hi compact:size-1.75 data-passed:opacity-50`}
                  data-passed={max() > 0 && (props.value ?? 0) / max() <= ((props.phases ?? []).length - (i() + 1)) / (props.phases ?? []).length}
                  style={{ left: `calc(${(((props.phases ?? []).length - (i() + 1)) / (props.phases ?? []).length) * 100}% - 0.321rem)`, top: "calc(50% - 0.321rem)" }}
                />
              )}
            </For>
          </View>
          <View class={`flex-row items-center justify-between w-full min-w-0 gap-x-3`}>
            <Show when={(props.stagger ?? -1) >= 0}>
              <View class={`shrink min-w-0 flex-row items-center gap-x-1`}>
                <Text class={`shrink-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted`}>{t("hud.boss-bar.stagger")}</Text>
                <View
                  class={`relative w-42.5 min-w-10 h-2 overflow-hidden bg-track shadow-plate ring ring-rim compact:h-1.25`}
                  role="meter"
                  label={t("hud.boss-bar.stagger")}
                  valueMin={0}
                  valueMax={100}
                  valueNow={(props.stagger ?? 0) >= 1 ? 100 : (props.stagger ?? 0) * 100}
                >
                  <View class="absolute left-0 top-0 h-full bg-linear-to-b from-sheen to-transparent to-42% bg-gold shadow-glow shadow-gold" style={{ width: `${Math.min(Math.max(props.stagger ?? 0, 0), 1) * 100}%` }} />
                  <For each={Array.from({ length: Math.max(staggerCells() - 1, 0) }, (_, n) => n)}>
                    {(n) => <View class="absolute top-0 bottom-0 w-0.75 bg-glass -translate-x-full" style={{ left: `${((n() + 1) / staggerCells()) * 100}%` }} />}
                  </For>
                </View>
              </View>
            </Show>
            <View class={`flex-row items-center ml-auto min-w-0 gap-x-1`}>
              <Show when={(props.defeated ?? false) || invulnerable()}>
                <Text class={`shrink-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted data-defeated:text-boon-ink`} role="status" data-defeated={props.defeated}>
                  {(props.defeated ?? false) ? t("hud.boss-bar.defeated") : t("hud.boss-bar.invulnerable")}
                </Text>
              </Show>
              <For each={props.weaknesses ?? []} key={(w) => w.id}>
                {(w) => (
                  <View class={`relative shrink-0 size-5.5 nine-chip-cut bg-foe compact:size-4`} role="image" label={t("hud.boss-bar.weak", { name: w().name ?? "" })}>
                    <Show when={w().icon} fallback={
                      <View class="absolute inset-0.5 nine-chip-cut flex-row items-center justify-center bg-raised-lo">
                        <Show when={w().glyph}>
                          <Icon class="size-16/25 text-fg" name={w().glyph} />
                        </Show>
                      </View>
                    }>
                      <Image class="absolute inset-0.5 nine-chip-cut" src={w().icon} />
                    </Show>
                  </View>
                )}
              </For>
              <Text class={`shrink-0 text-base leading-none font-bold tabular-nums whitespace-nowrap text-fg compact:text-caption`}>
                {`${formatNumber(share() * 100, { decimals: 0 })}%`}
              </Text>
            </View>
          </View>
        </>
      }>
        <View class={`flex-row items-center gap-x-1`}>
          <Spinner size={18} name={t("hud.boss-bar.loading")} />
          <Text class={`text-xs font-semibold tracking-label uppercase whitespace-nowrap text-fg-muted`} role="status">
            {t("hud.boss-bar.loading")}
          </Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<BossBarProps>> = {
  default: {"name":"Ashen Colossus","epithet":"Warden of the Ninefold Gate","value":5800,"max":10000,"ghost":7100,"phases":[{"id":"shell","label":"Shell","complete":true},{"id":"storm","label":"Storm","complete":false},{"id":"core","label":"Core","complete":false}],"phase":"storm","stagger":0.7,"staggerCells":10,"weaknesses":[{"id":"fire","name":"Fire","icon":0,"glyph":"fire"},{"id":"frost","name":"Frost","icon":0,"glyph":"drop"}],"invulnerable":false,"defeated":false,"loading":false},
  critical: {"name":"Ashen Colossus","epithet":"","value":900,"max":10000,"ghost":1600,"phases":[],"phase":"","stagger":0,"staggerCells":10,"weaknesses":[],"invulnerable":false,"defeated":false,"loading":false},
  invulnerable: {"name":"The Glass Warden","value":4000,"max":10000,"invulnerable":true,"defeated":false,"loading":false},
  defeated: {"name":"The Glass Warden","value":0,"max":10000,"defeated":true,"invulnerable":false,"loading":false},
  loading: {"name":"Unknown presence","loading":true,"defeated":false,"invulnerable":false},
  overflow: {"name":"The Warden of the Far Northern Observatory and the Drowned Abbey","epithet":"Keeper of the Seventh Bell beneath the Mere, Last of the Tidebound Choir","value":1,"max":10,"ghost":9,"phases":[{"id":"a","label":"A","complete":true},{"id":"b","label":"B","complete":true},{"id":"c","label":"C","complete":true},{"id":"last","label":"The final defence of the observatory","complete":false}],"phase":"last","stagger":1,"staggerCells":40,"weaknesses":[{"id":"a","name":"A","icon":0,"glyph":"fire"},{"id":"b","name":"B","icon":0,"glyph":"drop"},{"id":"c","name":"C","icon":0,"glyph":"bolt"},{"id":"d","name":"D","icon":0,"glyph":"star"},{"id":"e","name":"E","icon":0,"glyph":"skull"}],"invulnerable":false,"defeated":false,"loading":false},
};
