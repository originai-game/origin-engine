import { For, Icon, Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import "./compass.en.ogt";

export interface CompassPoi {
  id: string;
  /** In radians clockwise from north (0 to 2π). */
  bearing: number;
  /** In m (negative: none). */
  distance?: number;
  /** objective, ping, enemy, ally, loot or poi. */
  kind?: string;
  /** An icon of the atlas; empty draws a diamond. */
  icon?: string;
  /** For a reader. */
  label?: string;
}

export interface CompassProps {
  /** The player's heading in radians clockwise from north, 0 to 2π. */
  heading?: number;
  /** Its points. */
  pois?: ReadonlyArray<CompassPoi>;
  /** The distance's unit (m). */
  units?: string;
}

/**
 * A strip of the horizon centred on the heading: the cardinal letters
 * large, the intercardinal ones dim, a tick each 15 degrees and a tall
 * one each 45, on a dark band that fades out at both ends. The heading
 * stands on a tag in the HUD's accent over it with a caret under the
 * ticks; each point of interest is a mark under the strip with its
 * distance. A lap (360 degrees) is three windows wide, so a window shows
 * 120 degrees; the strip is three laps and every mark repeats at its
 * bearing on each lap. The band's end fade and the caret's triangle have
 * no TSX surface: the fade is dropped and the caret is a small diamond.
 */
const K = 0.4774648;

const POINTS: ReadonlyArray<{ at: number; cardinal: boolean; key: string }> = [
  { at: 0, cardinal: true, key: "component.compass.n" },
  { at: 0.375, cardinal: false, key: "component.compass.ne" },
  { at: 0.75, cardinal: true, key: "component.compass.e" },
  { at: 1.125, cardinal: false, key: "component.compass.se" },
  { at: 1.5, cardinal: true, key: "component.compass.s" },
  { at: 1.875, cardinal: false, key: "component.compass.sw" },
  { at: 2.25, cardinal: true, key: "component.compass.w" },
  { at: 2.625, cardinal: false, key: "component.compass.nw" },
];

// Each kind's ink as a ground (the pip) and as text (its icon): theme roles
// paired per scheme by the pack.
const POI_INKS: Record<string, [string, string]> = {
  objective: ["bg-gold", "text-gold"],
  ping: ["bg-hud-accent", "text-hud-accent"],
  enemy: ["bg-foe-ink", "text-foe-ink"],
  ally: ["bg-ally-ink", "text-ally-ink"],
  loot: ["bg-epic", "text-epic"],
};

function poiInk(kind?: string): [string, string] {
  return POI_INKS[kind ?? ""] ?? ["bg-fg", "text-fg"];
}

function PoiCopy(props: { poi: CompassPoi; wrap: number; units: string }) {
  const ink = () => poiInk(props.poi.kind);
  return (
    <View
      class="absolute flex-col items-center gap-y-0.75 w-20 top-9.75"
      label={props.poi.label}
      style={{ left: `calc(${((props.poi.bearing * K + props.wrap) / 9) * 100}% - 2.857rem)` }}
    >
      <Show when={(props.poi.icon ?? "") !== ""} fallback={
        <View class={`size-2.5 m-0.25 rotate-45 ring-regular ring-entry-hi ${ink()[0]}`} />
      }>
        <Icon class={`size-3 ${ink()[1]}`} name={props.poi.icon} />
      </Show>
      <Show when={(props.poi.distance ?? -1) >= 0}>
        <Text class="text-caption leading-tight font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs">{`${formatNumber(props.poi.distance ?? 0)}${props.units}`}</Text>
      </Show>
    </View>
  );
}

/**
 * A compass strip: the cardinal and intercardinal points and ticks sliding
 * under the heading, the heading in degrees on a tab, and the points of
 * interest with their distances. Read only.
 */
export function Compass(props: CompassProps) {
  const heading = () => Math.min(Math.max(props.heading ?? 0, 0), Math.PI * 2);
  const degrees = () => (heading() * 57.29578);
  const pois = () => props.pois ?? [];
  return (
    <View class="flex-col items-center w-160 max-w-full" role="status" label={t("component.compass.name")} valueText={`${formatNumber(degrees())}°`}>
      <View class="flex-col items-center relative z-1 -mb-1">
        <Text class="py-0 px-1.25 text-xs leading-snug font-bold tabular-nums text-hud-accent-fg bg-hud-accent rounded-tl-control rounded-br-control text-shadow-none">
          {(degrees() < 9.5 ? "00" : degrees() < 99.5 ? "0" : "") + formatNumber(degrees()) + "°"}
        </Text>
      </View>
      <View class="relative w-full h-12.5 overflow-hidden">
        <View class="absolute inset-x-0 top-0 h-9 bg-glass" />
        <View
          class="absolute top-0 bottom-0 left-1/2 w-9/1"
          style={{ transform: `translateX(${(-(heading() * K + 3) / 9) * 100}%)` }}
        >
          <For each={POINTS.flatMap((p) => [0, 3, 6, 9].map((lap) => ({ at: p.at + lap, cardinal: p.cardinal, key: p.key, lap })))} key={(p) => `${p.key}-${p.lap}`}>
            {(p) => {
              const d = Math.abs(p().at - heading() * K - 3);
              const opacity = Math.min(Math.max((d - 0.05) * 25, 0), 1);
              return (
                <Text
                  class={`absolute top-0 -translate-x-1/2 whitespace-nowrap ${p().cardinal ? "text-xl leading-none text-fg" : "text-base leading-tight text-fg-muted"}`}
                  style={{ left: `${(p().at / 9) * 100}%`, opacity }}
                >
                  {t(p().key)}
                </Text>
              );
            }}
          </For>
          <For each={Array.from({ length: 72 }, (_, k) => k)} key={(k) => k}>
            {(k) => (
              <View
                class="absolute top-6.5 w-0.5 h-1.5 bg-rim-soft"
                style={{ left: `${((k() * 0.125) / 9) * 100}%` }}
              />
            )}
          </For>
          <For each={Array.from({ length: 24 }, (_, k) => k)} key={(k) => k}>
            {(k) => (
              <View
                class="absolute top-5.5 w-0.5 h-2.5 bg-rim"
                style={{ left: `${((k() * 0.375) / 9) * 100}%` }}
              />
            )}
          </For>
          <For each={pois()} key={(poi) => poi.id}>
            {(poi) => (
              <View class="absolute inset-0">
                <PoiCopy poi={poi()} wrap={0} units={props.units ?? ""} />
                <PoiCopy poi={poi()} wrap={3} units={props.units ?? ""} />
                <PoiCopy poi={poi()} wrap={6} units={props.units ?? ""} />
              </View>
            )}
          </For>
        </View>
        <View class="absolute top-8 left-1/2 -translate-x-1/2 size-2 rotate-45 bg-hud-accent" />
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CompassProps>> = {
  default: {"heading":0.733,"units":"m","pois":[{"id":"quest","bearing":0.9,"distance":240,"kind":"objective","icon":"star","label":"Quest"},{"id":"camp","bearing":0.15,"distance":85,"kind":"poi","icon":"flag","label":"Camp"},{"id":"foe","bearing":1.6,"distance":-1,"kind":"enemy","icon":"","label":"Enemy"}]},
};
