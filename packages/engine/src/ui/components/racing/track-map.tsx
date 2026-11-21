import { For, Line, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import "./racing.en.ogt";

export interface RacingTrackMapProps {
  /** The track's racing line as an SVG path (d) in the map's 260 by 220 box; empty draws no road. */
  path: string;
  /** The part of the lap the player has driven, as an SVG path in the same box, lit in the HUD's accent; empty draws none. */
  driven: string;
  /** The start line's place in the box, across. */
  startX: number;
  /** The start line's place in the box, down. */
  startY: number;
  /** The start line's turn in degrees. */
  startAngle: number;
  /** The cars on the map, keyed rows of { id, x, y, heading, color, player, name }: x and y in the map's box; color the car's colour as a CSS colour (empty: neutral); player marks the player's car, drawn as an arrow turned to heading (degrees clockwise from up) over the others' dots; name for a reader. */
  cars?: ReadonlyArray<{ id: string; x: number; y: number; heading: number; color: string; player: boolean; name: string }>;
  /** The track's name as the game words it, under the map; empty hides it. */
  name: string;
  /** The track's length as the game words it (4.8 km), on a tag beside the name; empty hides it. */
  length: string;
}

/**
 * Walks an SVG path's M/L/C commands (the racing line's forms), sampling
 * the cubics into a polyline of points in the path's 260 by 220 box. All
 * commands are read as absolute (the game's paths are).
 */
function pathPoints(d: string): { points: number[]; closed: boolean } {
  const toks = d.match(/[MLCZmlcz]|-?\d*\.?\d+(?:e[-+]?\d+)?/g) ?? [];
  const points: number[] = [];
  const push = (x: number, y: number) => points.push(Math.round(x * 1000) / 1000, Math.round(y * 1000) / 1000);
  let i = 0;
  let cmd = "";
  let cx = 0, cy = 0, sx = 0, sy = 0;
  let closed = false;
  while (i < toks.length) {
    const tok = toks[i]!;
    if (/[a-zA-Z]/.test(tok)) {
      if (cmd === "Z" || cmd === "z") {
        closed = true;
        push(sx, sy);
      }
      cmd = tok;
      i++;
      continue;
    }
    const n = () => Number(toks[i++]);
    if (cmd === "M" || cmd === "m") {
      cx = n(); cy = n();
      sx = cx; sy = cy;
      push(cx, cy);
      cmd = "L";
    } else if (cmd === "L" || cmd === "l") {
      cx = n(); cy = n();
      push(cx, cy);
    } else if (cmd === "C" || cmd === "c") {
      const x1 = n(), y1 = n(), x2 = n(), y2 = n(), x3 = n(), y3 = n();
      for (let k = 1; k <= 16; k++) {
        const u = k / 16, w = 1 - u;
        push(w * w * w * cx + 3 * w * w * u * x1 + 3 * w * u * u * x2 + u * u * u * x3, w * w * w * cy + 3 * w * w * u * y1 + 3 * w * u * u * y2 + u * u * u * y3);
      }
      cx = x3; cy = y3;
    } else {
      i++;
    }
  }
  if (cmd === "Z" || cmd === "z") {
    closed = true;
    push(sx, sy);
  }
  return { points, closed };
}

/**
 * The track map at the top right: the circuit as a light line over a dark
 * band, the lap driven so far lit in the HUD's accent, the start line, the
 * other cars as dots in their colours and the player's car as an arrow
 * over them; the track's name and length under it. The game gives the
 * shape and every position in the map's box; read only.
 */
export function RacingTrackMap(props: RacingTrackMapProps) {
  const track = () => pathPoints(props.path);
  const driven = () => pathPoints(props.driven);
  return (
    <View class={`flex-col gap-y-1 w-55 max-w-full text-fg compact:w-30`} role="group" label={props.name !== "" ? props.name : t("racing.map.name")}>
      <View class="relative w-full aspect-13/11" role="image" label={t("racing.map.name")}>
        <Show when={props.path !== ""}>
          {/* The road's band, then its line, from the same path data; the
              points stay in the path's 260 by 220 box and the tree maps
              them onto the plate, which already has the box's aspect. */}
          <Line class="absolute inset-0 size-full stroke-well stroke-14" points={track().points} closed={track().closed} viewBox={[260, 220]} fit="fill" />
          <Line class="absolute inset-0 size-full stroke-rim-strong stroke-4" points={track().points} closed={track().closed} viewBox={[260, 220]} fit="fill" />
        </Show>
        <Show when={props.driven !== ""}>
          <Line class="absolute inset-0 size-full stroke-hud-accent stroke-4" points={driven().points} viewBox={[260, 220]} fit="fill" />
        </Show>
        <Show when={props.path !== ""}>
          <View
            class="absolute w-3.5 h-1 bg-fg"
            style={{ left: `${props.startX / 260 * 100}%`, top: `${props.startY / 220 * 100}%`, transform: `translate(-50%,-50%) rotate(${props.startAngle}deg)` }}
          />
        </Show>
        <For each={props.cars ?? []} key={(c) => c.id}>
          {(c) => (
            <Show
              when={c().player}
              fallback={
                <View
                  class="absolute size-3 rounded-full border-2 border-well bg-sheen"
                  style={{ left: `${c().x / 260 * 100}%`, top: `${c().y / 220 * 100}%`, transform: "translate(-50%,-50%)", backgroundColor: c().color || undefined }}
                  role="image"
                  label={c().name}
                />
              }
            >
              {/* The arrow: a filled triangle from borders, turned to its heading. */}
              <View
                class="absolute size-0 border-x-7 border-b-11 border-x-transparent border-b-fg"
                style={{ left: `${c().x / 260 * 100}%`, top: `${c().y / 220 * 100}%`, transform: `translate(-50%,-55%) rotate(${c().heading}deg)` }}
                role="image"
                label={c().name}
              />
            </Show>
          )}
        </For>
      </View>
      <Show when={props.name !== "" || props.length !== ""}>
        <View class="flex-row items-center justify-between gap-x-2 min-w-0 compact:hidden">
          <Show when={props.name !== ""}>
            <Text class={`shrink min-w-0 text-reading font-semibold whitespace-nowrap overflow-hidden text-fg text-stroke-3 text-stroke-bg`}>{props.name}</Text>
          </Show>
          <Show when={props.length !== ""}>
            <Text
              class={`shrink-0 max-w-1/2 ml-auto py-0 px-1.5 text-xs font-bold tabular-nums whitespace-nowrap overflow-hidden bg-well nine-chip-cut text-fg ${packClass("Chip", "frame")}`}
              label={`${t("racing.map.length")} ${props.length}`}
            >
              {props.length}
            </Text>
          </Show>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RacingTrackMapProps>> = {
  default: { "path": "M40 170 C10 120 30 50 90 40 C140 30 150 80 190 70 C240 58 250 120 215 150 C185 176 150 140 120 165 C95 188 60 200 40 170 Z", "driven": "M40 170 C10 120 30 50 90 40 C140 30 150 80 190 70", "startX": 40, "startY": 170, "startAngle": -60, "name": "Costa Serpente", "length": "4.8 km", "cars": [{ "id": "okafor", "x": 212, "y": 88, "heading": 0, "color": "#ffd24a", "player": false, "name": "M. Okafor" }, { "id": "lindqvist", "x": 200, "y": 72, "heading": 0, "color": "#4f7bff", "player": false, "name": "S. Lindqvist" }, { "id": "castellanos", "x": 150, "y": 150, "heading": 0, "color": "#55e39a", "player": false, "name": "R. Castellanos" }, { "id": "you", "x": 190, "y": 70, "heading": 100, "color": "", "player": true, "name": "You" }] },
  start: { "path": "M40 170 C10 120 30 50 90 40 C140 30 150 80 190 70 C240 58 250 120 215 150 C185 176 150 140 120 165 C95 188 60 200 40 170 Z", "driven": "", "startX": 40, "startY": 170, "startAngle": -60, "name": "Costa Serpente", "length": "", "cars": [{ "id": "you", "x": 42, "y": 176, "heading": -20, "color": "", "player": true, "name": "You" }, { "id": "okafor", "x": 46, "y": 184, "heading": 0, "color": "#ffd24a", "player": false, "name": "M. Okafor" }] },
  overflow: { "path": "M30 110 L230 110", "driven": "M30 110 L130 110", "startX": 30, "startY": 110, "startAngle": 90, "name": "Autodromo Internazionale del Circuito delle Colline di Costa Serpente", "length": "20.832 km Nordschleife", "cars": [{ "id": "you", "x": 130, "y": 110, "heading": 90, "color": "", "player": true, "name": "You" }] },
};
