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

const POI_INKS: Record<string, string> = {
  objective: "#ffcf5a",
  ping: "#38e8ff",
  enemy: "#ff4d5e",
  ally: "#4fd8ff",
  loot: "#c86ff5",
};

function poiInk(kind?: string): string {
  return POI_INKS[kind ?? ""] ?? "#ffffff";
}

function PoiCopy(props: { poi: CompassPoi; wrap: number; units: string }) {
  const ink = () => poiInk(props.poi.kind);
  return (
    <View
      class="absolute flex-col items-center gap-y-[0.214rem] w-[5.714rem] top-[2.786rem]"
      label={props.poi.label}
      style={{ left: `calc(${((props.poi.bearing * K + props.wrap) / 9) * 100}% - 2.857rem)` }}
    >
      <Show when={(props.poi.icon ?? "") !== ""} fallback={
        <View class="size-[0.714rem] m-[0.071rem] rotate-45 shadow-[0px_0px_0px_0.143rem_#1a1e28d9]" style={{ background: ink() }} />
      }>
        <Icon class="size-[0.857rem]" name={props.poi.icon} style={{ color: ink() }} />
      </Show>
      <Show when={(props.poi.distance ?? -1) >= 0}>
        <Text class="text-[0.929rem] leading-[1.1] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-xs">{`${formatNumber(props.poi.distance ?? 0)}${props.units}`}</Text>
      </Show>
    </View>
  );
}

export function Compass(props: CompassProps) {
  const heading = () => Math.min(Math.max(props.heading ?? 0, 0), Math.PI * 2);
  const degrees = () => (heading() * 57.29578);
  const pois = () => props.pois ?? [];
  return (
    <View class="flex-col items-center w-[40rem] max-w-full" role="status" label={t("component.compass.name")} valueText={`${formatNumber(degrees())}°`}>
      <View class="flex-col items-center relative z-[1] -mb-[0.286rem]">
        <Text class="py-0 px-[0.357rem] text-[0.857rem] leading-[1.35] font-bold tabular-nums text-[#262b38] bg-[#38e8ff] rounded-tl-[0.286rem] rounded-br-[0.286rem] text-shadow-none">
          {(degrees() < 9.5 ? "00" : degrees() < 99.5 ? "0" : "") + formatNumber(degrees()) + "°"}
        </Text>
      </View>
      <View class="relative w-full h-[3.571rem] overflow-hidden">
        <View class="absolute inset-x-0 top-0 h-[2.571rem] bg-linear-to-b from-[#171a2100] to-[#171a218c]" />
        <View
          class="absolute top-0 bottom-0 left-1/2 w-[900%]"
          style={{ transform: `translateX(${(-(heading() * K + 3) / 9) * 100}%)` }}
        >
          <For each={POINTS.flatMap((p) => [0, 3, 6, 9].map((lap) => ({ at: p.at + lap, cardinal: p.cardinal, key: p.key, lap })))} key={(p) => `${p.key}-${p.lap}`}>
            {(p) => {
              const d = Math.abs(p().at - heading() * K - 3);
              const opacity = Math.min(Math.max((d - 0.05) * 25, 0), 1);
              return (
                <Text
                  class={`absolute top-0 -translate-x-1/2 whitespace-nowrap ${p().cardinal ? "text-[1.429rem] leading-none text-fg" : "text-[1.143rem] leading-[1.2] text-fg/66"}`}
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
                class="absolute top-[1.857rem] w-[0.143rem] h-[0.429rem] bg-white/66"
                style={{ left: `${((k() * 0.125) / 9) * 100}%` }}
              />
            )}
          </For>
          <For each={Array.from({ length: 24 }, (_, k) => k)} key={(k) => k}>
            {(k) => (
              <View
                class="absolute top-[1.571rem] w-[0.143rem] h-[0.714rem] bg-white"
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
        <View class="absolute top-[2.286rem] left-1/2 -translate-x-1/2 size-[0.571rem] rotate-45 bg-[#38e8ff]" />
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CompassProps>> = {
  default: {"heading":0.733,"units":"m","pois":[{"id":"quest","bearing":0.9,"distance":240,"kind":"objective","icon":"star","label":"Quest"},{"id":"camp","bearing":0.15,"distance":85,"kind":"poi","icon":"flag","label":"Camp"},{"id":"foe","bearing":1.6,"distance":-1,"kind":"enemy","icon":"","label":"Enemy"}]},
};
