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
  /** Out of view, it stays pinned to the rim, dimmed, with a tick pointing its way (an objective, a party member); false drops it. */
  pinned?: boolean;
  /** above or below the player (another floor): a small chevron on the pip; empty is level. */
  elevation?: string;
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

const RIM = 10; // px kept between a pinned pip's centre and the glass's edge.

function Marker(props: { marker: () => MinimapViewMarker; cx: () => number; cy: () => number; zoom: () => number; round: () => boolean }) {
  const m = props.marker;
  const pip = () => (m().kind === "objective" ? PIP_BIG : m().kind === "loot" ? PIP * 0.85 : m().kind === "ping" ? PIP * 1.5 : PIP);
  const raw = () => ({
    x: (m().x - props.cx()) * (SIDE / props.zoom()),
    y: (m().y - props.cy()) * (SIDE / props.zoom()),
  });
  const limit = SIDE / 2 - RIM;
  // Where it is drawn, relative to the centre: as is inside the glass, on the rim when pinned
  // and out of view (round glass: along its ray; square: on the box).
  const placed = () => {
    const { x, y } = raw();
    if (props.round()) {
      const d = Math.hypot(x, y);
      if (d <= limit) return { x, y, edge: false };
      return { x: (x / d) * limit, y: (y / d) * limit, edge: true };
    }
    const k = Math.max(Math.abs(x), Math.abs(y));
    if (k <= limit) return { x, y, edge: false };
    return { x: (x / k) * limit, y: (y / k) * limit, edge: true };
  };
  const outside = () => {
    const { x, y } = raw();
    return props.round() ? Math.hypot(x, y) > SIDE / 2 : Math.max(Math.abs(x), Math.abs(y)) > SIDE / 2;
  };
  const visible = () => !outside() || m().pinned === true;
  const edge = () => m().pinned === true && placed().edge;
  const bearing = () => (Math.atan2(raw().x, -raw().y) * 180) / Math.PI;
  const left = () => `calc(${SIDE / 2 + placed().x}px - ${pip() / 2}rem)`;
  const top = () => `calc(${SIDE / 2 + placed().y}px - ${pip() / 2}rem)`;
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
  const ink = () => {
    if (m().kind === "ally" || m().kind === "poi") return "bg-ally-ink";
    if (m().kind === "enemy") return "bg-foe-ink";
    if (m().kind === "objective") return "bg-gold";
    if (m().kind === "loot") return "bg-epic";
    return "bg-warn";
  };
  const ring = () => (m().kind === "ping" ? "ring-heavy ring-fill inset-ring-regular inset-ring-warn" : "ring-regular ring-raised-lo");
  return (
    <Show when={visible()}>
      <View
        class="absolute"
        label={m().label}
        data-edge={edge()}
        style={{ width: `${pip()}rem`, height: `${pip()}rem`, left: left(), top: top() }}
      >
        <View class={`absolute inset-0 ${shape()} ${fill()} ${ring()} ${edge() ? "opacity-soft scale-press" : ""}`}>
          <Show when={m().kind === "ping"}>
            <View class="absolute inset-0 rounded-full ring-heavy ring-fill inset-ring-regular inset-ring-warn animate-ping" />
          </Show>
        </View>
        <Show when={edge()}>
          <View class="absolute inset-0 pointer-events-none" style={{ transform: `rotate(${bearing()}deg)` }}>
            <View class={`absolute left-1/2 -top-1.5 w-0.75 h-1.25 -translate-x-1/2 rounded-capsule ${ink()}`} />
          </View>
        </Show>
        <Show when={m().elevation === "above" || m().elevation === "below"}>
          <View class="absolute -right-1.5 -top-1 w-2 h-1.75">
            <Line
              class="size-full stroke-fg stroke-[1.5]"
              points={m().elevation === "above" ? [1, 5, 4, 2, 7, 5] : [1, 2, 4, 5, 7, 2]}
              viewBox={[8, 7]}
              label={m().elevation === "above" ? t("component.minimap-view.above") : t("component.minimap-view.below")}
            />
          </View>
        </Show>
      </View>
    </Show>
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
              {(m) => <Marker marker={m} cx={cx} cy={cy} zoom={zoom} round={() => round() || props.rotate === true} />}
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
            class="absolute left-1/2 top-1/2"
            style={{
              width: `${ARROW_BOX}px`,
              height: `${ARROW_BOX}px`,
              transform: `translate(-50%, -50%)${props.rotate ? "" : ` rotate(${heading()}deg)`}`,
            }}
          >
            <Line
              points={[ARROW_BOX / 2, 1.8, ARROW_BOX - 3.6, ARROW_BOX - 3.6, ARROW_BOX / 2, ARROW_BOX - 10.2, 3.6, ARROW_BOX - 3.6]}
              closed
              class="stroke-fg stroke-2"
            />
          </View>
          <View class="absolute inset-0 rounded-full pointer-events-none inset-shadow-sunk inset-ring-hairline inset-ring-border data-square:rounded-card" data-square={props.square === true} />
        </View>
        <View
          class="absolute inset-0 pointer-events-none"
          style={{
            transform: props.rotate ? `rotate(${heading() * -1}deg)` : undefined,
            transformOrigin: "50% 50%",
          }}
        >
          <Text
            class="absolute top-1.75 left-1/2 -translate-x-1/2 min-w-6 h-6 flex-row items-center justify-center rounded-full bg-foe shadow-glow shadow-foe ring-regular ring-key-fg text-caption font-bold leading-none text-foe-fg whitespace-nowrap"
            style={{ transform: props.rotate ? `rotate(${heading()}deg)` : undefined }}
          >
            {t("component.minimap-view.north")}
          </Text>
        </View>
      </View>
      <Show when={(props.zone ?? "") !== ""}>
        <Text class={`py-0.5 px-2 text-caption font-semibold tracking-label uppercase whitespace-nowrap ${packClass("Plate", "frame")}`}>
          {props.zone}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<MinimapViewProps>> = {
  pinned: {"centerX":0.5,"centerY":0.5,"zoom":0.2,"heading":0,"zone":"Drowned Abbey","markers":[{"id":"quest","kind":"objective","x":0.9,"y":0.2,"label":"Quest","pinned":true},{"id":"mira","kind":"ally","x":0.1,"y":0.7,"label":"Mira","pinned":true},{"id":"far","kind":"enemy","x":0.95,"y":0.95,"label":"Enemy"},{"id":"up","kind":"poi","x":0.53,"y":0.45,"label":"Stairs","elevation":"above"},{"id":"down","kind":"enemy","x":0.45,"y":0.55,"label":"Enemy","elevation":"below"}]},
  default: {"texture":0,"centerX":0.5,"centerY":0.5,"zoom":0.3,"heading":30,"rotate":false,"square":false,"zone":"Ashen Vale","markers":[{"id":"ally","kind":"ally","x":0.45,"y":0.42,"label":"Mira"},{"id":"foe","kind":"enemy","x":0.58,"y":0.55,"label":"Enemy"},{"id":"quest","kind":"objective","x":0.6,"y":0.38,"label":"Quest"},{"id":"ping","kind":"ping","x":0.42,"y":0.6,"label":"Ping"},{"id":"chest","kind":"loot","x":0.53,"y":0.47,"label":"Chest"}]},
};
