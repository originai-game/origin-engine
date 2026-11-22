import { For, Image, Line, Show, Text, View, packClass, type UiEvent, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./minimap-view.en.ogt";

export interface MinimapViewMarker {
  id: string;
  /** ally, enemy, objective, ping, loot or poi. */
  kind?: string;
  /** On the map, 0 to 1 across. */
  x: number;
  /** On the map, 0 to 1 down. */
  y: number;
  /** For a reader. */
  label?: string;
}

export interface MinimapViewProps {
  /** The map texture (og.load.texture); 0 draws the well with a grid. */
  texture?: Texture | 0;
  /** The map point at its centre (the player's), 0 to 1 across. */
  centerX?: number;
  /** The map point at its centre, 0 to 1 down. */
  centerY?: number;
  /** How much of the map it shows across, 0 to 1 (0.3). Default 0.3. */
  zoom?: number;
  /** The player's heading in degrees clockwise from north. */
  heading?: number;
  /** Player-up: the map turns with the heading; false keeps north up and turns the player's arrow. */
  rotate?: boolean;
  /** A square frame; false is round. */
  square?: boolean;
  /** Its markers. */
  markers?: ReadonlyArray<MinimapViewMarker>;
  /** The zone's name under it; empty is none. */
  zone?: string;
  /** Pick: a tap on the glass. */
  onPick?: (e: UiEvent) => void;
}

/**
 * A minimap: a lit bezel round (or square) dark glass, bearing ticks
 * inside its rim, a red north badge pinned on the rim, the player's arrow
 * in the middle with a heading cone. Each marker kind keeps its own shape
 * at any size (an ally a disc, an enemy a diamond, an objective a larger
 * diamond, loot a square, a ping a ring), each in its kind's theme ink, so
 * a kind is never told by colour alone. The zone's name sits on a plate
 * under it. Reports pick (a tap on the glass). The style draws the player
 * as a filled SVG arrow and the cone and bezel with conic gradients: the
 * arrow is a stroked <Line> triangle here, the cone's wedge and the
 * bezel's sheen are approximated, the objective marker's glow is dropped,
 * the bearing ticks are a plain hairline ring and the zone plate keeps no
 * rim (a cut takes one fill).
 */
const SIDE = 192; // px, the glass (size-5xl); the map math is in px.
const PIP = 12 / 14; // rem, a marker pip (12 font steps).
const PIP_BIG = 17 / 14; // rem, an objective pip.

function Marker(props: { marker: () => MinimapViewMarker; cx: () => number; cy: () => number; zoom: () => number }) {
  const m = props.marker;
  const pip = () => (m().kind === "objective" ? PIP_BIG : m().kind === "loot" ? PIP * 0.85 : m().kind === "ping" ? PIP * 1.5 : PIP);
  const offset = () => (kind: "x" | "y") => SIDE / 2 + ((kind === "x" ? m().x : m().y) - (kind === "x" ? props.cx() : props.cy())) * (SIDE / props.zoom());
  const left = () => `calc(${offset()("x")}px - ${pip() / 2}rem)`;
  const top = () => `calc(${offset()("y")}px - ${pip() / 2}rem)`;
  const shape = () => {
    if (m().kind === "enemy" || m().kind === "objective") return "rotate-45";
    if (m().kind === "ally" || m().kind === "poi" || m().kind === "ping") return "rounded-full";
    return "";
  };
  // Each kind's fill is its theme ink, paired per scheme (bright on the dark map, deep on the
  // light one), and every pip stands in a halo of the map's own ground so the grid and the
  // cone never run through it.
  const fill = () => {
    if (m().kind === "ally" || m().kind === "poi") return "bg-ally-ink";
    if (m().kind === "enemy") return "bg-foe-ink";
    if (m().kind === "objective") return "bg-gold";
    if (m().kind === "loot") return "bg-epic";
    return "";
  };
  const ring = () => (m().kind === "ping" ? "ring-4 ring-fill inset-ring-2 inset-ring-warn" : "ring-2 ring-raised-lo");
  return (
    <View
      class={`absolute ${shape()} ${fill()} ${ring()}`}
      label={m().label}
      style={{ width: `${pip()}rem`, height: `${pip()}rem`, left: left(), top: top() }}
    >
      <Show when={m().kind === "ping"}>
        <View class="absolute inset-0 rounded-full ring-4 ring-fill inset-ring-2 inset-ring-warn animate-ping" />
      </Show>
    </View>
  );
}

const ARROW_BOX = 29; // px: the player arrow's box (an objective pip and a half).

/**
 * A minimap: the map texture zoomed round the player's point, player-up or
 * north-up, round or square, with its markers and the zone's name under it.
 * It reports onPick for a tap on the glass.
 */
export function MinimapView(props: MinimapViewProps) {
  const zoom = () => Math.min(Math.max(props.zoom ?? 0.3, 0.01), 1);
  const cx = () => props.centerX ?? 0.5;
  const cy = () => props.centerY ?? 0.5;
  const heading = () => props.heading ?? 0;
  const round = () => !(props.square ?? false);
  const mapSide = () => SIDE / zoom();
  return (
    <View class="flex-col items-center gap-y-1" role="image" label={props.zone || t("component.minimap-view.name")} data-square={props.square === true}>
      <View
        class={`relative rounded-full p-1.25 data-square:rounded-panel bg-conic from-rim-strong via-rim-soft to-rim-strong shadow-float`}
        data-square={props.square === true}
      >
        <View
          class={`flex-row items-center justify-center gap-1 relative size-48 overflow-hidden rounded-full data-square:rounded-card bg-raised-lo pointer-events-auto`}
          role="button"
          label={t("component.minimap-view.name")}
          onPress={(e) => props.onPick?.(e)}
          data-square={props.square === true}
        >
          <View
            class="absolute size-full top-0 left-0"
            style={{
              transform: props.rotate ? `rotate(${heading() * -1}deg)` : undefined,
              transformOrigin: "50% 50%",
            }}
          >
            <Show when={(props.texture ?? 0) > 0} fallback={
              <View class="absolute inset-0">
                <For each={Array.from({ length: 9 }, (_, k) => k)} key={(k) => `h${k}`}>
                  {(k) => <View class="absolute bg-rim" style={{ left: `${((k() + 1) / 10) * 100}%`, top: 0, width: "0.071rem", height: "100%" }} />}
                </For>
                <For each={Array.from({ length: 9 }, (_, k) => k)} key={(k) => `v${k}`}>
                  {(k) => <View class="absolute bg-rim" style={{ top: `${((k() + 1) / 10) * 100}%`, left: 0, height: "0.071rem", width: "100%" }} />}
                </For>
              </View>
            }>
              <Image
                class="absolute"
                src={props.texture ?? 0}
                style={{
                  width: `${mapSide()}px`,
                  height: `${mapSide()}px`,
                  left: `${SIDE / 2 - cx() * mapSide()}px`,
                  top: `${SIDE / 2 - cy() * mapSide()}px`,
                }}
              />
            </Show>
            <For each={props.markers ?? []} key={(m) => m.id}>
              {(m) => <Marker marker={m} cx={cx} cy={cy} zoom={zoom} />}
            </For>
          </View>
          <Show when={round()}>
            <View class="absolute inset-0 rounded-full border border-rim-soft pointer-events-none" />
          </Show>
          <View
            class="absolute left-1/2 top-1/2 w-20 h-40 -bg-conic-30 from-hud-accent to-transparent to-17%"
            style={{ transform: `translate(-50%, -50%) ${props.rotate ? "" : `rotate(${heading()}deg)`}` }}
          />
          <View
            class="absolute"
            style={{
              width: `${ARROW_BOX}px`,
              height: `${ARROW_BOX}px`,
              left: `calc(6rem - ${PIP_BIG * 0.75}rem)`,
              top: `calc(6rem - ${PIP_BIG * 0.75}rem)`,
              transform: props.rotate ? undefined : `rotate(${heading()}deg)`,
            }}
          >
            <Line
              points={[ARROW_BOX / 2, 1.8, ARROW_BOX - 3.6, ARROW_BOX - 3.6, ARROW_BOX / 2, ARROW_BOX - 10.2, 3.6, ARROW_BOX - 3.6]}
              closed
              class="stroke-fg stroke-2"
            />
          </View>
          <View class="absolute inset-0 rounded-full pointer-events-none inset-shadow-sunk inset-ring-1 inset-ring-rim data-square:rounded-card" data-square={props.square === true} />
        </View>
        <Text class="absolute top-1.75 left-1/2 -translate-x-1/2 min-w-6 h-6 flex-row items-center justify-center rounded-full bg-foe shadow-glow shadow-foe ring-2 ring-key-fg text-caption font-bold leading-none text-foe-fg whitespace-nowrap">
          {t("component.minimap-view.north")}
        </Text>
      </View>
      <Show when={(props.zone ?? "") !== ""}>
        <Text class={`py-0.5 px-2 text-caption font-semibold tracking-label uppercase whitespace-nowrap text-fg nine-plate-cut bg-plate-hi ${packClass("Plate", "frame")}`}>
          {props.zone}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<MinimapViewProps>> = {
  default: {"texture":0,"centerX":0.5,"centerY":0.5,"zoom":0.3,"heading":30,"rotate":false,"square":false,"zone":"Ashen Vale","markers":[{"id":"ally","kind":"ally","x":0.45,"y":0.42,"label":"Mira"},{"id":"foe","kind":"enemy","x":0.58,"y":0.55,"label":"Enemy"},{"id":"quest","kind":"objective","x":0.6,"y":0.38,"label":"Quest"},{"id":"ping","kind":"ping","x":0.42,"y":0.6,"label":"Ping"},{"id":"chest","kind":"loot","x":0.53,"y":0.47,"label":"Chest"}]},
};
